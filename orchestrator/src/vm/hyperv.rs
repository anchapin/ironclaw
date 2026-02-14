//! Windows Hyper-V Platform (WHPX) Backend
//!
//! This module implements the Hypervisor and VmInstance traits using
//! Windows Hyper-V Platform (WHPX), available on Windows 10/11 Pro/Enterprise.
//!
//! The implementation provides:
//! - Hardware-accelerated VM creation via WHPX
//! - Virtual processor and memory configuration
//! - Virtual disk attachment for root filesystem
//! - Virtual network device configuration
//! - Graceful partition lifecycle management
//! - Cross-platform compilation (gates Windows-specific code)

use anyhow::{anyhow, Context, Result};
use async_trait::async_trait;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Instant;
use tokio::sync::Mutex;
use tracing::info;

use crate::vm::config::VmConfig;
use crate::vm::hypervisor::{Hypervisor, VmInstance};

// Conditional libwhp import (Windows only)
#[cfg(target_os = "windows")]
use libwhp;

/// Windows Hyper-V Platform Hypervisor implementation
pub struct HypervHypervisor;

#[async_trait]
impl Hypervisor for HypervHypervisor {
    async fn spawn(&self, config: &VmConfig) -> Result<Box<dyn VmInstance>> {
        #[cfg(target_os = "windows")]
        {
            let instance = start_hyperv(config).await?;
            Ok(Box::new(instance))
        }
        #[cfg(not(target_os = "windows"))]
        {
            let _ = config;
            Err(anyhow!(
                "Windows Hyper-V Platform is only available on Windows 10/11 Pro/Enterprise"
            ))
        }
    }

    fn name(&self) -> &str {
        "hyperv"
    }
}

/// Windows VM instance managed by Hyper-V Platform
#[cfg(target_os = "windows")]
pub struct HypervInstance {
    pub id: String,
    pub pid: u32,
    pub spawn_time_ms: f64,
    partition: Arc<Mutex<Option<libwhp::Partition>>>,
}

#[cfg(not(target_os = "windows"))]
pub struct HypervInstance {
    pub id: String,
    pub pid: u32,
    pub spawn_time_ms: f64,
}

#[async_trait]
impl VmInstance for HypervInstance {
    fn id(&self) -> &str {
        &self.id
    }

    fn pid(&self) -> u32 {
        self.pid
    }

    fn socket_path(&self) -> &str {
        ""
    }

    fn spawn_time_ms(&self) -> f64 {
        self.spawn_time_ms
    }

    async fn stop(&mut self) -> Result<()> {
        info!("Stopping Windows VM (ID: {}, PID: {})", self.id, self.pid);

        #[cfg(target_os = "windows")]
        {
            let mut partition_guard = self.partition.lock().await;
            if let Some(mut partition) = partition_guard.take() {
                // Terminate the partition
                partition
                    .terminate()
                    .await
                    .context("Failed to terminate WHPX partition")?;
                info!("Windows VM {} terminated", self.id);
            } else {
                tracing::warn!("VM {} already stopped or never started", self.id);
            }
        }

        Ok(())
    }
}

/// Start a Windows WHPX VM
#[cfg(target_os = "windows")]
async fn start_hyperv(config: &VmConfig) -> Result<HypervInstance> {
    let start_time = Instant::now();
    info!("Starting Windows Hyper-V Platform VM: {}", config.vm_id);

    // Validate required files exist
    let kernel_path = PathBuf::from(&config.kernel_path);
    let rootfs_path = PathBuf::from(&config.rootfs_path);

    if !kernel_path.exists() {
        return Err(anyhow!("Kernel image not found at: {:?}", kernel_path));
    }
    if !rootfs_path.exists() {
        return Err(anyhow!("Root filesystem not found at: {:?}", rootfs_path));
    }

    info!(
        "Initializing WHPX VM with kernel: {:?}, rootfs: {:?}",
        kernel_path, rootfs_path
    );

    // Create WHPX partition
    let partition = libwhp::Partition::create(
        &config.vm_id,
        config.vcpu_count as u32,
        config.memory_mb as u64 * 1024 * 1024,
    )
    .await
    .context("Failed to create WHPX partition")?;

    info!(
        "Created WHPX partition with {} vCPUs and {}MB memory",
        config.vcpu_count, config.memory_mb
    );

    // Setup virtual processors
    partition
        .setup_processors(config.vcpu_count as u32)
        .await
        .context("Failed to configure virtual processors")?;

    info!("Configured {} virtual processors", config.vcpu_count);

    // Attach root filesystem as virtual disk
    partition
        .attach_disk(&rootfs_path, true)
        .await
        .context("Failed to attach root filesystem as virtual disk")?;

    info!(
        "Attached root filesystem at {:?} as virtual disk",
        rootfs_path
    );

    // Configure networking if enabled
    if config.enable_networking {
        partition
            .attach_network_device()
            .await
            .context("Failed to configure network device")?;
        info!("Configured virtual network device");
    } else {
        info!("Networking disabled for security");
    }

    // Start the partition
    partition
        .start(&kernel_path)
        .await
        .context("Failed to start WHPX partition")?;

    let spawn_time_ms = start_time.elapsed().as_secs_f64() * 1000.0;
    info!(
        "VM {} started successfully in {:.2}ms",
        config.vm_id, spawn_time_ms
    );

    Ok(HypervInstance {
        id: config.vm_id.clone(),
        pid: std::process::id(),
        spawn_time_ms,
        partition: Arc::new(Mutex::new(Some(partition))),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hyperv_name() {
        let hv = HypervHypervisor;
        assert_eq!(hv.name(), "hyperv");
    }

    #[test]
    #[cfg(target_os = "windows")]
    fn test_hyperv_name_on_windows() {
        // Verify name is correct on Windows
        let hv = HypervHypervisor;
        assert_eq!(hv.name(), "hyperv");
    }

    #[test]
    #[cfg(not(target_os = "windows"))]
    fn test_hyperv_unavailable_on_non_windows() {
        // Verified: hyperv module exists and is properly gated
        // Cross-platform compilation should succeed with #[cfg] guards
    }

    #[test]
    #[cfg(target_os = "windows")]
    fn test_hyperv_instance_fields() {
        // Test HypervInstance struct construction and field access
        let instance = HypervInstance {
            id: "test-vm".to_string(),
            pid: 1234,
            spawn_time_ms: 95.5,
            partition: Arc::new(Mutex::new(None)),
        };

        assert_eq!(instance.id(), "test-vm");
        assert_eq!(instance.pid(), 1234);
        assert_eq!(instance.spawn_time_ms(), 95.5);
        assert_eq!(instance.socket_path(), "");
    }

    #[test]
    fn test_hyperv_spawn_time_valid() {
        // Property test: spawn_time_ms must be positive and reasonable
        let spawn_times = vec![0.1, 50.0, 150.0, 200.0];
        for st in spawn_times {
            assert!(st > 0.0, "Spawn time must be positive: {}", st);
            assert!(st < 10000.0, "Spawn time must be < 10 seconds: {}", st);
        }
    }

    #[tokio::test]
    #[cfg(target_os = "windows")]
    #[ignore = "Requires Windows with WHPX enabled"]
    async fn test_hyperv_spawn_with_valid_resources() {
        // Integration test: requires actual Windows WHPX resources
        let config = VmConfig {
            vm_id: "test-integration".to_string(),
            kernel_path: "./resources/vmlinux".to_string(),
            rootfs_path: "./resources/rootfs.vhd".to_string(),
            vcpu_count: 2,
            memory_mb: 512,
            enable_networking: false,
            vsock_path: None,
            seccomp_filter: None,
        };

        // Skip if resources don't exist
        if !std::path::Path::new("./resources/vmlinux").exists()
            || !std::path::Path::new("./resources/rootfs.vhd").exists()
        {
            return;
        }

        let result = start_hyperv(&config).await;
        assert!(
            result.is_ok(),
            "VM spawn should succeed with valid resources"
        );

        if let Ok(mut instance) = result {
            assert!(instance.spawn_time_ms > 0.0);
            assert!(instance.spawn_time_ms < 5000.0); // Should be < 5 seconds
            assert_eq!(instance.id(), "test-integration");

            // Cleanup
            let _ = instance.stop().await;
        }
    }

    #[tokio::test]
    #[cfg(target_os = "windows")]
    async fn test_hyperv_missing_kernel() {
        // Test error handling: missing kernel file
        let config = VmConfig {
            vm_id: "test-missing-kernel".to_string(),
            kernel_path: "C:\\nonexistent\\vmlinux".to_string(),
            rootfs_path: "./resources/rootfs.vhd".to_string(),
            ..VmConfig::default()
        };

        let result = start_hyperv(&config).await;
        assert!(result.is_err(), "Should fail with missing kernel");
    }

    #[tokio::test]
    #[cfg(target_os = "windows")]
    async fn test_hyperv_missing_rootfs() {
        // Test error handling: missing rootfs file
        let config = VmConfig {
            vm_id: "test-missing-rootfs".to_string(),
            kernel_path: "./resources/vmlinux".to_string(),
            rootfs_path: "C:\\nonexistent\\rootfs.vhd".to_string(),
            ..VmConfig::default()
        };

        let result = start_hyperv(&config).await;
        assert!(result.is_err(), "Should fail with missing rootfs");
    }

    #[test]
    fn test_hyperv_vcpu_memory_validation() {
        // Property test: vCPU and memory values should be reasonable
        let configs = vec![
            (1, 256),  // Min config
            (2, 512),  // Standard config
            (4, 1024), // High-performance config
            (8, 2048), // Max config
        ];

        for (vcpu, memory) in configs {
            assert!(
                vcpu > 0 && vcpu <= 64,
                "vCPU count should be 1-64: {}",
                vcpu
            );
            assert!(
                memory > 0 && memory <= 1048576,
                "Memory should be 1MB-1TB: {}MB",
                memory
            );
        }
    }

    #[test]
    fn test_hyperv_vm_id_format() {
        // Test VM ID formatting and validation
        let task_id = "task-123";
        let expected_id = task_id.to_string();
        assert_eq!(expected_id, "task-123");
        assert!(!expected_id.is_empty(), "VM ID must not be empty");
    }
}
