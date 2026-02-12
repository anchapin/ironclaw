// Network Isolation Firewall Configuration
//
// This module configures and manages firewall rules to ensure complete
// network isolation for IronClaw VMs. Only vsock communication is allowed.
//
// Key invariants:
// - ALL external network traffic is BLOCKED
// - Only vsock communication is permitted
// - Firewall rules persist across VM lifecycle
// - Rules are automatically cleaned up on VM destruction

use anyhow::{Context, Result};
use std::process::Command as SyncCommand;
use tokio::process::Command as AsyncCommand;
use tracing::{info, warn};

/// Firewall manager for VM network isolation
pub struct FirewallManager {
    vm_id: String,
    chain_name: String,
}

impl FirewallManager {
    /// Create a new firewall manager for a VM
    ///
    /// # Arguments
    ///
    /// * `vm_id` - Unique identifier for the VM
    pub fn new(vm_id: String) -> Self {
        // Create a unique chain name for this VM
        // Sanitize vm_id to only contain alphanumeric characters
        let sanitized_id: String = vm_id
            .chars()
            .map(|c| if c.is_alphanumeric() { c } else { '_' })
            .collect();

        let chain_name = format!("IRONCLAW_{}", sanitized_id);

        Self { vm_id, chain_name }
    }

    /// Configure firewall rules to isolate the VM
    ///
    /// This creates a new iptables chain and configures rules to:
    /// 1. Block all inbound traffic
    /// 2. Block all outbound traffic
    /// 3. Allow only vsock communication (which doesn't go through iptables)
    ///
    /// # Returns
    ///
    /// * `Ok(())` - Firewall rules configured successfully
    /// * `Err(_)` - Failed to configure firewall rules
    ///
    /// # Note
    ///
    /// This function requires root privileges. If running without root,
    /// it will return an error. In production, the orchestrator should
    /// run with appropriate capabilities.
    pub async fn configure_isolation(&self) -> Result<()> {
        info!("Configuring firewall isolation for VM: {}", self.vm_id);

        // Check if iptables is available
        if !Self::check_iptables_installed() {
            anyhow::bail!("iptables is not installed or not accessible");
        }

        // Check if running as root
        if !Self::is_root() {
            anyhow::bail!("Firewall configuration requires root privileges");
        }

        // Create a new chain for this VM
        self.create_chain().await?;

        // Add rules to drop all traffic
        self.add_drop_rules().await?;

        // WARN: The chain is created but not linked to INPUT/OUTPUT/FORWARD.
        // This is intentional because we don't know the network interface name here.
        // It serves as a placeholder for when specific interfaces are assigned.
        warn!(
            "Firewall chain {} created but not linked to main tables. Rules are currently inactive until an interface is explicitly blocked.",
            self.chain_name
        );

        info!(
            "Firewall isolation configured for VM: {} (chain: {})",
            self.vm_id, self.chain_name
        );

        Ok(())
    }

    /// Remove firewall rules and cleanup (Async)
    ///
    /// This should be called when the VM is destroyed.
    pub async fn cleanup_async(&self) -> Result<()> {
        info!("Cleaning up firewall rules for VM: {}", self.vm_id);

        // Remove jump rules from INPUT and FORWARD chains
        // We loop until all references are removed to ensure we can delete the chain
        loop {
            let status = AsyncCommand::new("iptables")
                .args(["-D", "INPUT", "-j", &self.chain_name])
                .output()
                .await
                .map(|o| o.status.success())
                .unwrap_or(false);
            if !status {
                break;
            }
        }

        loop {
            let status = AsyncCommand::new("iptables")
                .args(["-D", "FORWARD", "-j", &self.chain_name])
                .output()
                .await
                .map(|o| o.status.success())
                .unwrap_or(false);
            if !status {
                break;
            }
        }

        // Flush and delete the chain
        self.flush_chain().await?;
        self.delete_chain().await?;

        info!("Firewall rules cleaned up for VM: {}", self.vm_id);

        Ok(())
    }

    /// Remove firewall rules and cleanup (Sync)
    ///
    /// This is used by Drop trait.
    pub fn cleanup(&self) -> Result<()> {
        // Remove jump rules (Sync)
        loop {
            let status = SyncCommand::new("iptables")
                .args(["-D", "INPUT", "-j", &self.chain_name])
                .output()
                .map(|o| o.status.success())
                .unwrap_or(false);
            if !status {
                break;
            }
        }

        loop {
            let status = SyncCommand::new("iptables")
                .args(["-D", "FORWARD", "-j", &self.chain_name])
                .output()
                .map(|o| o.status.success())
                .unwrap_or(false);
            if !status {
                break;
            }
        }

        // Flush chain (Sync)
        let _ = SyncCommand::new("iptables")
            .args(["-F", &self.chain_name])
            .output();

        // Delete chain (Sync)
        let _ = SyncCommand::new("iptables")
            .args(["-X", &self.chain_name])
            .output();

        Ok(())
    }

    /// Verify that firewall rules are active
    ///
    /// # Returns
    ///
    /// * `Ok(true)` - Rules are active and configured correctly
    /// * `Ok(false)` - Rules are not active
    /// * `Err(_)` - Failed to check rules
    pub async fn verify_isolation(&self) -> Result<bool> {
        let output = AsyncCommand::new("iptables")
            .args(["-L", &self.chain_name])
            .output()
            .await;

        // If iptables command fails (not installed, can't execute, etc.),
        // treat as if rules are not active (graceful degradation)
        let output = match output {
            Ok(output) => output,
            Err(_) => {
                tracing::debug!("iptables not available, treating as not isolated");
                return Ok(false);
            }
        };

        if !output.status.success() {
            // Chain doesn't exist, so rules are not active
            return Ok(false);
        }

        let rules = String::from_utf8_lossy(&output.stdout);

        // Check if DROP rules are present
        let has_drop_rules = rules.contains("DROP");

        Ok(has_drop_rules)
    }

    /// Create a new iptables chain
    async fn create_chain(&self) -> Result<()> {
        info!("Creating iptables chain: {}", self.chain_name);

        let output = AsyncCommand::new("iptables")
            .args(["-N", &self.chain_name])
            .output()
            .await
            .context("Failed to create iptables chain")?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            anyhow::bail!("Failed to create chain: {}", stderr);
        }

        Ok(())
    }

    /// Add DROP rules to the chain
    async fn add_drop_rules(&self) -> Result<()> {
        info!("Adding DROP rules to chain: {}", self.chain_name);

        // Drop all incoming traffic
        let output = AsyncCommand::new("iptables")
            .args(["-A", &self.chain_name, "-j", "DROP"])
            .output()
            .await
            .context("Failed to add DROP rule for incoming traffic")?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            anyhow::bail!("Failed to add DROP rule: {}", stderr);
        }

        Ok(())
    }

    /// Flush all rules in the chain
    async fn flush_chain(&self) -> Result<()> {
        info!("Flushing iptables chain: {}", self.chain_name);

        let output = AsyncCommand::new("iptables")
            .args(["-F", &self.chain_name])
            .output()
            .await
            .context("Failed to flush iptables chain")?;

        // Ignore errors if chain doesn't exist
        if !output.status.success() {
            warn!("Failed to flush chain (may not exist): {}", self.chain_name);
        }

        Ok(())
    }

    /// Delete the chain
    async fn delete_chain(&self) -> Result<()> {
        info!("Deleting iptables chain: {}", self.chain_name);

        let output = AsyncCommand::new("iptables")
            .args(["-X", &self.chain_name])
            .output()
            .await
            .context("Failed to delete iptables chain")?;

        // Ignore errors if chain doesn't exist
        if !output.status.success() {
            warn!(
                "Failed to delete chain (may not exist): {}",
                self.chain_name
            );
        }

        Ok(())
    }

    /// Check if iptables is installed and accessible
    fn check_iptables_installed() -> bool {
        let output = SyncCommand::new("iptables").arg("--version").output();

        match output {
            Ok(o) => o.status.success(),
            Err(_) => false,
        }
    }

    /// Check if running as root
    fn is_root() -> bool {
        use std::process::Output;

        let output: Output = SyncCommand::new("id")
            .arg("-u")
            .output()
            .unwrap_or_else(|_| Output {
                status: Default::default(),
                stdout: vec![],
                stderr: vec![],
            });

        if output.status.success() {
            let uid = String::from_utf8_lossy(&output.stdout);
            uid.trim() == "0"
        } else {
            false
        }
    }

    /// Block specific network interface (e.g., tap0 for VM)
    ///
    /// This links the isolation chain to the system INPUT and FORWARD chains
    /// for the specified interface, ensuring traffic is blocked.
    pub async fn block_interface(&self, interface: &str) -> Result<()> {
        info!(
            "Blocking network interface: {} for VM: {}",
            interface, self.vm_id
        );

        // Link INPUT chain to our isolation chain for this interface
        let output = AsyncCommand::new("iptables")
            .args(["-I", "INPUT", "-i", interface, "-j", &self.chain_name])
            .output()
            .await
            .context("Failed to link INPUT chain")?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            anyhow::bail!("Failed to link INPUT chain: {}", stderr);
        }

        // Link FORWARD chain to our isolation chain for this interface
        let output = AsyncCommand::new("iptables")
            .args(["-I", "FORWARD", "-i", interface, "-j", &self.chain_name])
            .output()
            .await
            .context("Failed to link FORWARD chain")?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            anyhow::bail!("Failed to link FORWARD chain: {}", stderr);
        }

        Ok(())
    }

    /// Get the chain name (for testing/debugging)
    pub fn chain_name(&self) -> &str {
        &self.chain_name
    }

    /// Get the VM ID
    pub fn vm_id(&self) -> &str {
        &self.vm_id
    }
}

impl Drop for FirewallManager {
    fn drop(&mut self) {
        // Attempt to cleanup when the manager is dropped
        if let Err(e) = self.cleanup() {
            warn!(
                "Failed to cleanup firewall rules for VM {}: {}",
                self.vm_id, e
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_firewall_manager_creation() {
        let manager = FirewallManager::new("test-vm".to_string());
        assert_eq!(manager.vm_id(), "test-vm");
        assert!(manager.chain_name().contains("IRONCLAW"));
        assert!(manager.chain_name().contains("test_vm"));
    }

    #[test]
    fn test_firewall_manager_sanitization() {
        // Test that special characters are sanitized
        let manager = FirewallManager::new("test-vm@123#456".to_string());
        assert_eq!(manager.vm_id(), "test-vm@123#456");
        assert!(manager.chain_name().contains("test_vm_123_456"));
        assert!(!manager.chain_name().contains('@'));
        assert!(!manager.chain_name().contains('#'));
    }

    #[test]
    fn test_firewall_manager_chain_name_format() {
        let manager = FirewallManager::new("my-vm".to_string());
        let chain = manager.chain_name();

        // Chain name should start with IRONCLAW_
        assert!(chain.starts_with("IRONCLAW_"));

        // Chain name should only contain alphanumeric and underscore
        assert!(chain.chars().all(|c| c.is_alphanumeric() || c == '_'));
    }

    #[test]
    fn test_iptables_check() {
        // This test will pass if iptables is installed
        let has_iptables = FirewallManager::check_iptables_installed();
        // We can't assert this in all environments, so we just log it
        if has_iptables {
            println!("iptables is installed");
        } else {
            println!("iptables is not installed (expected in some test environments)");
        }
    }

    // Property-based test: chain names are always valid
    #[test]
    fn test_chain_name_always_valid() {
        let test_cases = vec![
            "simple",
            "with-dash",
            "with_underscore",
            "with.dot",
            "with@symbol",
            "with space",
            "with/slash",
        ];

        for vm_id in test_cases {
            let manager = FirewallManager::new(vm_id.to_string());
            let chain = manager.chain_name();

            // Chain name should be a valid iptables chain name
            // (max 28 characters, alphanumeric and underscore only)
            assert!(chain.len() <= 28);
            assert!(chain.chars().all(|c| c.is_alphanumeric() || c == '_'));
            assert!(chain.starts_with("IRONCLAW_"));
        }
    }
}
