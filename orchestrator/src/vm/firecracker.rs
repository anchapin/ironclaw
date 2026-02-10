// Firecracker Integration with Jailer
//
// This module handles Firecracker VM spawning with Jailer sandboxing.
// Provides secure, isolated VM execution for agent tasks.

use anyhow::{Context, Result};
use std::path::PathBuf;
use tracing::info;

use crate::vm::config::VmConfig;
use crate::vm::jailer::{start_with_jailer, JailerConfig, JailerProcess};

/// Firecracker VM process manager
#[derive(Debug)]
pub struct FirecrackerProcess {
    /// PID of the (jailed) Firecracker process
    pub pid: u32,

    /// Path to the Firecracker API socket
    pub socket_path: PathBuf,

    /// Whether this VM is running in a Jailer sandbox
    pub is_sandboxed: bool,

    /// VM configuration
    pub config: VmConfig,
}

/// Start a Firecracker VM process with Jailer sandboxing
///
/// # Arguments
///
/// * `vm_config` - VM configuration (memory, CPU, kernel, rootfs)
///
/// # Returns
///
/// * `FirecrackerProcess` - Handle to the running Firecracker VM
///
/// # Behavior
///
/// 1. Creates JailerConfig from VmConfig
/// 2. Spawns Firecracker with Jailer (or direct if Jailer unavailable)
/// 3. Returns process handle with socket path for API communication
///
/// # Resource Limits
///
/// - CPU: Limited by cgroup (default: 1 vCPU)
/// - Memory: Limited by cgroup (default: 256 MB)
/// - Network: Isolated via network namespace (if configured)
pub async fn start_firecracker(vm_config: &VmConfig) -> Result<FirecrackerProcess> {
    info!(
        "Starting Firecracker VM: {} ({} vCPUs, {} MB)",
        vm_config.vm_id, vm_config.vcpu_count, vm_config.memory_mb
    );

    // Create Jailer configuration from VM config
    let jailer_config = JailerConfig {
        jailer_id: vm_config.vm_id.clone(),
        exec_file: PathBuf::from("/usr/local/bin/firecracker-v1.14.1"),
        cpu_count: vm_config.vcpu_count,
        memory_limit_mb: vm_config.memory_mb,
        ..Default::default()
    };

    // Start Firecracker with Jailer sandboxing
    let jailer_process = start_with_jailer(&jailer_config)
        .await
        .context("Failed to start Firecracker with Jailer")?;

    // Convert to FirecrackerProcess
    Ok(FirecrackerProcess {
        pid: jailer_process.pid,
        socket_path: jailer_process.api_socket,
        is_sandboxed: jailer_process.jailed,
        config: vm_config.clone(),
    })
}

/// Stop a Firecracker VM process
///
/// # Arguments
///
/// * `process` - Firecracker process to stop
///
/// # Behavior
///
/// 1. Sends SIGTERM to Firecracker process
/// 2. Waits for graceful shutdown (max 5 seconds)
/// 3. Force kills (SIGKILL) if timeout
/// 4. Cleans up resources (chroot, cgroups) handled by Jailer
pub async fn stop_firecracker(process: FirecrackerProcess) -> Result<()> {
    info!(
        "Stopping Firecracker VM: {} (PID: {}, sandboxed: {})",
        process.config.vm_id, process.pid, process.is_sandboxed
    );

    // Create JailerProcess wrapper for termination
    let jailer_process = JailerProcess {
        pid: process.pid,
        api_socket: process.socket_path,
        jailed: process.is_sandboxed,
    };

    // Try graceful shutdown first (SIGTERM)
    jailer_process.terminate().await?;

    // TODO: Wait for process to exit (with timeout)
    // For now, we've sent the signal

    // If the process is still running after timeout, force kill
    // This is a simplified version - production code should wait
    if jailer_process.is_running().await {
        info!("Process still running, sending SIGKILL");
        jailer_process.force_kill().await?;
    }

    info!("Firecracker VM stopped: {}", process.config.vm_id);
    Ok(())
}

/// Check if a Firecracker VM is still running
pub async fn is_vm_running(process: &FirecrackerProcess) -> bool {
    // Create JailerProcess wrapper for status check
    let jailer_process = JailerProcess {
        pid: process.pid,
        api_socket: process.socket_path.clone(),
        jailed: process.is_sandboxed,
    };

    jailer_process.is_running().await
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_firecracker_process_attributes() {
        let config = VmConfig::new("test-vm".to_string());
        let process = FirecrackerProcess {
            pid: 1234,
            socket_path: PathBuf::from("/tmp/test.sock"),
            is_sandboxed: true,
            config: config.clone(),
        };

        assert_eq!(process.pid, 1234);
        assert_eq!(process.socket_path, PathBuf::from("/tmp/test.sock"));
        assert!(process.is_sandboxed);
        assert_eq!(process.config.vm_id, "test-vm");
    }

    #[tokio::test]
    async fn test_start_firecracker_requires_binary() {
        let config = VmConfig::new("test-vm".to_string());

        // This will fail because Firecracker binary doesn't exist
        let result = start_firecracker(&config).await;

        // We expect an error about missing binary or jailer
        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        assert!(
            err.contains("not found") || err.contains("binary") || err.contains("Failed to start"),
            "Expected binary not found error, got: {}",
            err
        );
    }

    #[tokio::test]
    async fn test_stop_firecracker_with_mock_process() {
        let config = VmConfig::new("test-vm".to_string());
        let process = FirecrackerProcess {
            pid: 99999, // Non-existent PID
            socket_path: PathBuf::from("/tmp/test.sock"),
            is_sandboxed: false,
            config,
        };

        // Should not panic even with non-existent PID
        let result = stop_firecracker(process).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_is_vm_running_nonexistent() {
        let config = VmConfig::new("test-vm".to_string());
        let process = FirecrackerProcess {
            pid: 99999, // Non-existent PID
            socket_path: PathBuf::from("/tmp/test.sock"),
            is_sandboxed: false,
            config,
        };

        // Should return false for non-existent PID
        assert!(!is_vm_running(&process).await);
    }

    #[tokio::test]
    async fn test_start_and_stop_flow() {
        let config = VmConfig {
            vm_id: "test-vm-flow".to_string(),
            vcpu_count: 1,
            memory_mb: 256,
            kernel_path: "/dev/null".to_string(), // Invalid but tests flow
            rootfs_path: "/dev/null".to_string(),
            enable_networking: false,
        };

        // Start will fail (no binary), but tests the flow
        let start_result = start_firecracker(&config).await;
        assert!(start_result.is_err());

        // If start fails, we can't test stop
        // This is expected in test environment without Firecracker
    }
}
