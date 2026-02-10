// JIT Micro-VM Configuration
//
// Firecracker VM configuration for secure agent execution

use anyhow::Result;
use serde::{Deserialize, Serialize};

/// VM configuration for Firecracker
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VmConfig {
    /// VM ID (unique identifier)
    pub vm_id: String,

    /// Number of vCPUs (default: 1)
    pub vcpu_count: u8,

    /// Memory size in MB (default: 512)
    pub memory_mb: u32,

    /// Kernel image path
    pub kernel_path: String,

    /// Root filesystem path
    pub rootfs_path: String,

    /// Enable networking (default: false for security)
    /// WARNING: When false, ALL network access is blocked including internet.
    /// Only vsock communication is allowed for host-guest interaction.
    pub enable_networking: bool,

    /// vsock socket path (automatically generated)
    #[serde(skip)]
    pub vsock_path: Option<String>,
}

impl Default for VmConfig {
    fn default() -> Self {
        Self {
            vm_id: "default".to_string(),
            vcpu_count: 1,
            memory_mb: 512,
            kernel_path: "/path/to/vmlinux.bin".to_string(),
            rootfs_path: "/path/to/rootfs.ext4".to_string(),
            enable_networking: false,
            vsock_path: None,
        }
    }
}

impl VmConfig {
    /// Create a new VM config with defaults
    pub fn new(vm_id: String) -> Self {
        let mut config = Self {
            vm_id,
            ..Default::default()
        };

        // Generate vsock path
        config.vsock_path = Some(format!("/tmp/ironclaw/vsock/{}.sock", config.vm_id));

        config
    }

    /// Validate configuration
    pub fn validate(&self) -> Result<(), String> {
        if self.vcpu_count == 0 {
            return Err("vCPU count must be > 0".to_string());
        }
        if self.memory_mb < 128 {
            return Err("Memory must be at least 128 MB".to_string());
        }

        // Security check: networking must be disabled
        if self.enable_networking {
            return Err(
                "Networking MUST be disabled for security. enable_networking must be false."
                    .to_string(),
            );
        }

        Ok(())
    }

    /// Validate configuration with anyhow::Result
    pub fn validate_anyhow(&self) -> Result<()> {
        self.validate()
            .map_err(|e| anyhow::anyhow!("Configuration validation failed: {}", e))
    }

    /// Convert to Firecracker JSON config
    pub fn to_firecracker_json(&self) -> String {
        // TODO: Implement actual Firecracker JSON format
        format!(
            r#"{{
  "boot-source": {{
    "kernel_image_path": "{}"
  }},
  "machine-config": {{
    "vcpu_count": {},
    "mem_size_mib": {},
    "ht_enabled": false
  }}
}}"#,
            self.kernel_path, self.vcpu_count, self.memory_mb
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = VmConfig::default();
        assert_eq!(config.vcpu_count, 1);
        assert_eq!(config.memory_mb, 512);
        assert!(!config.enable_networking);
        assert!(config.vsock_path.is_none()); // Default has no vsock path
    }

    #[test]
    fn test_new_config() {
        let config = VmConfig::new("test-vm".to_string());
        assert_eq!(config.vm_id, "test-vm");
        assert!(!config.enable_networking);
        assert!(config.vsock_path.is_some());
        assert!(config.vsock_path.as_ref().unwrap().contains("test-vm"));
    }

    #[test]
    fn test_config_validation() {
        let config = VmConfig::new("test-vm".to_string());
        assert!(config.validate().is_ok());
        assert!(config.validate_anyhow().is_ok());
    }

    #[test]
    fn test_config_validation_fails_vcpu() {
        let mut config = VmConfig::new("test-vm".to_string());
        config.vcpu_count = 0;
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_config_validation_fails_memory() {
        let mut config = VmConfig::new("test-vm".to_string());
        config.memory_mb = 64;
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_config_validation_fails_networking_enabled() {
        let mut config = VmConfig::new("test-vm".to_string());
        config.enable_networking = true;
        let result = config.validate();
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("MUST be disabled"));
    }

    #[test]
    fn test_to_json() {
        let config = VmConfig::new("test-vm".to_string());
        let json = config.to_firecracker_json();
        assert!(json.contains("boot-source"));
        assert!(json.contains("machine-config"));
    }

    #[test]
    fn test_vsock_path_generation() {
        let config = VmConfig::new("my-vm-123".to_string());
        assert_eq!(
            config.vsock_path,
            Some("/tmp/ironclaw/vsock/my-vm-123.sock".to_string())
        );
    }

    // Property-based test: all valid configs must have networking disabled
    #[test]
    fn test_networking_always_disabled() {
        let config = VmConfig::new("test-vm".to_string());
        assert!(!config.enable_networking);

        // Even if we try to enable it, validation should fail
        let mut config = config;
        config.enable_networking = true;
        assert!(config.validate().is_err());
    }
}
