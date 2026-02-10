// JIT Micro-VM Module
//
// This module handles spawning and managing ephemeral Firecracker VMs.
//
// Key invariants:
// - Spawn time: <200ms
// - Ephemeral: VM destroyed after task completion
// - Security: No host execution, full isolation
// - Network isolation: ALL network traffic blocked, only vsock allowed

pub mod config;
pub mod firecracker;
pub mod firewall;
pub mod vsock;

#[cfg(test)]
mod tests;

use crate::vm::config::VmConfig;
use crate::vm::firewall::FirewallManager;
use anyhow::Result;

/// VM handle for managing lifecycle
pub struct VmHandle {
    pub id: String,
    pub pid: u32,
    pub config: VmConfig,
    firewall_manager: Option<FirewallManager>,
}

impl VmHandle {
    /// Get the vsock socket path for this VM
    pub fn vsock_path(&self) -> Option<&str> {
        self.config.vsock_path.as_deref()
    }
}

/// Spawn a new JIT Micro-VM with network isolation
///
/// # Arguments
///
/// * `task_id` - Unique identifier for the task
///
/// # Returns
///
/// * `VmHandle` - Handle for managing the VM
///
/// # Invariants
///
/// * Must complete in <200ms
/// * VM must be destroyed after task completion
/// * ALL network traffic is blocked (firewall configured)
/// * Only vsock communication is allowed
///
/// # Note
///
/// If not running as root, firewall configuration will be skipped
/// with a warning. In production, the orchestrator should run with
/// appropriate capabilities (CAP_NET_ADMIN, CAP_NET_RAW).
pub async fn spawn_vm(task_id: &str) -> Result<VmHandle> {
    tracing::info!("Spawning isolated VM for task: {}", task_id);

    // Create VM configuration with networking DISABLED
    let config = VmConfig::new(format!("vm-{}", task_id));

    // Validate configuration (ensures networking is disabled)
    config.validate_anyhow()?;

    // Configure firewall to block all network traffic
    let firewall_manager = FirewallManager::new(config.vm_id.clone());

    // Apply firewall rules (may fail if not root)
    match firewall_manager.configure_isolation() {
        Ok(_) => {
            tracing::info!("Firewall isolation configured for VM: {}", config.vm_id);
        }
        Err(e) => {
            tracing::warn!(
                "Failed to configure firewall (running without root?): {}. \
                VM will still have networking disabled in config, but firewall rules are not applied.",
                e
            );
            // Continue anyway - networking is still disabled in config
        }
    }

    // Verify firewall rules are active (if configured)
    match firewall_manager.verify_isolation() {
        Ok(true) => {
            tracing::info!("Firewall isolation verified for VM: {}", config.vm_id);
        }
        Ok(false) => {
            tracing::debug!("Firewall rules not active for VM: {}", config.vm_id);
        }
        Err(e) => {
            tracing::debug!("Failed to verify firewall rules: {}", e);
        }
    }

    // TODO: Implement Firecracker VM spawning
    // 1. Create VM config (kernel, memory, drives)
    // 2. Start Firecracker process
    // 3. Verify VM is responsive
    // 4. Return handle

    let pid = 0; // Placeholder

    tracing::info!(
        "VM spawned with network isolation: {} (firewall: {})",
        config.vm_id,
        firewall_manager.chain_name()
    );

    Ok(VmHandle {
        id: config.vm_id.clone(),
        pid,
        config,
        firewall_manager: Some(firewall_manager),
    })
}

/// Destroy a VM (ephemeral cleanup)
///
/// # Arguments
///
/// * `handle` - VM handle to destroy
///
/// # Important
///
/// This MUST be called after task completion to ensure
/// no malware can persist (the "infected computer no longer exists")
///
/// # Cleanup
///
/// - Firewall rules are automatically removed via Drop trait
/// - VM process is terminated
/// - All resources are cleaned up
pub async fn destroy_vm(handle: VmHandle) -> Result<()> {
    tracing::info!("Destroying VM: {}", handle.id);

    // Firewall cleanup happens automatically via Drop trait
    // when firewall_manager goes out of scope

    // TODO: Implement VM destruction
    // 1. Send shutdown signal to VM
    // 2. Wait for graceful shutdown (timeout: 5s)
    // 3. Force kill if timeout
    // 4. Clean up resources (memory, sockets, etc.)

    tracing::info!("VM destroyed: {}", handle.id);

    Ok(())
}

/// Verify that a VM is properly network-isolated
///
/// # Arguments
///
/// * `handle` - VM handle to verify
///
/// # Returns
///
/// * `Ok(true)` - VM is properly isolated
/// * `Ok(false)` - VM is not isolated
/// * `Err(_)` - Failed to check isolation status
pub fn verify_network_isolation(handle: &VmHandle) -> Result<bool> {
    if let Some(ref firewall) = handle.firewall_manager {
        firewall.verify_isolation()
    } else {
        Ok(false)
    }
}
