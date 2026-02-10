// JIT Micro-VM Configuration
//
// Firecracker VM configuration for secure agent execution

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
    pub enable_networking: bool,

    /// Mount rootfs as read-only (default: true for security)
    pub rootfs_readonly: bool,

    /// Enable rootfs integrity checking (default: true)
    pub enable_integrity_check: bool,

    /// Rootfs signature path (if signature verification enabled)
    pub rootfs_signature_path: Option<String>,

    /// Public key path for signature verification
    pub rootfs_pubkey_path: Option<String>,

    /// dm-verity hash tree path
    pub rootfs_hash_tree_path: Option<String>,
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
            rootfs_readonly: true,
            enable_integrity_check: true,
            rootfs_signature_path: None,
            rootfs_pubkey_path: None,
            rootfs_hash_tree_path: None,
        }
    }
}

impl VmConfig {
    /// Create a new VM config with defaults
    pub fn new(vm_id: String) -> Self {
        Self {
            vm_id,
            ..Default::default()
        }
    }

    /// Validate configuration
    pub fn validate(&self) -> Result<(), String> {
        if self.vcpu_count == 0 {
            return Err("vCPU count must be > 0".to_string());
        }
        if self.memory_mb < 128 {
            return Err("Memory must be at least 128 MB".to_string());
        }
        Ok(())
    }

    /// Convert to Firecracker JSON config
    pub fn to_firecracker_json(&self) -> String {
        // TODO: Implement actual Firecracker JSON format
        let readonly_flag = if self.rootfs_readonly { "true" } else { "false" };

        format!(
            r#"{{
  "boot-source": {{
    "kernel_image_path": "{}"
  }},
  "machine-config": {{
    "vcpu_count": {},
    "mem_size_mib": {},
    "ht_enabled": false
  }},
  "drives": [
    {{
      "drive_id": "rootfs",
      "path_on_host": "{}",
      "is_root_device": true,
      "is_read_only": {}
    }}
  ]
}}"#,
            self.kernel_path, self.vcpu_count, self.memory_mb, self.rootfs_path, readonly_flag
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
    }

    #[test]
    fn test_config_validation() {
        let config = VmConfig::default();
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_config_validation_fails() {
        let mut config = VmConfig::default();
        config.vcpu_count = 0;
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_to_json() {
        let config = VmConfig::new("test-vm".to_string());
        let json = config.to_firecracker_json();
        assert!(json.contains("boot-source"));
        assert!(json.contains("machine-config"));
    }
}
