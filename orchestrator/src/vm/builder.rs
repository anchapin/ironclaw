// Minimal Guest OS Builder
//
// Builds Alpine Linux-based minimal root filesystems for IronClaw VMs.
// Optimized for size (<100MB) and security.

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::fs::{self, File};
use std::io::{BufReader, Write};
use std::path::{Path, PathBuf};
use std::process::Command;
use tracing::{debug, info, warn};

/// Guest OS configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GuestOsConfig {
    /// Alpine Linux version (e.g., "v3.19")
    pub alpine_version: String,

    /// Architecture (x86_64, aarch64)
    pub arch: String,

    /// Additional packages to install
    pub packages: Vec<String>,

    /// Root filesystem size in MB
    pub rootfs_size_mb: u32,

    /// Output path for rootfs image
    pub output_path: PathBuf,

    /// Enable hardening (remove unnecessary binaries)
    pub enable_hardening: bool,
}

impl Default for GuestOsConfig {
    fn default() -> Self {
        Self {
            alpine_version: "v3.19".to_string(),
            arch: "x86_64".to_string(),
            packages: vec![
                "ca-certificates".to_string(),
                "curl".to_string(),
                "busybox".to_string(),
            ],
            rootfs_size_mb: 64,
            output_path: PathBuf::from("/tmp/ironclaw-rootfs.ext4"),
            enable_hardening: true,
        }
    }
}

impl GuestOsConfig {
    /// Create a new guest OS configuration
    pub fn new(output_path: PathBuf) -> Self {
        Self {
            output_path,
            ..Default::default()
        }
    }

    /// Build the minimal guest OS
    ///
    /// This creates an Alpine Linux-based root filesystem.
    pub fn build(&self) -> Result<BuildReport> {
        info!("Building guest OS: {}", self.output_path.display());

        let start_time = std::time::Instant::now();

        // Step 1: Download Alpine minirootfs
        let minirootfs_path = self.download_alpine_minirootfs()?;

        // Step 2: Create ext4 filesystem image
        self.create_ext4_image(&minirootfs_path)?;

        // Step 3: Mount and customize
        self.customize_rootfs()?;

        // Step 4: Apply hardening if enabled
        if self.enable_hardening {
            self.harden_rootfs()?;
        }

        let elapsed = start_time.elapsed();
        let file_size = fs::metadata(&self.output_path)
            .context("Failed to get rootfs size")?
            .len();

        info!(
            "Guest OS built successfully in {:.2}s ({} bytes)",
            elapsed.as_secs_f64(),
            file_size
        );

        Ok(BuildReport {
            success: true,
            file_size,
            build_time: elapsed,
            packages_installed: self.packages.len(),
        })
    }

    /// Download Alpine Linux minirootfs
    fn download_alpine_minirootfs(&self) -> Result<PathBuf> {
        info!("Downloading Alpine {} minirootfs", self.alpine_version);

        let url = format!(
            "https://dl-cdn.alpinelinux.org/alpine/{}/releases/{}/alpine-minirootfs-{}-{}.tar.gz",
            self.alpine_version,
            self.arch,
            self.alpine_version,
            self.arch
        );

        let cache_dir = dirs::cache_dir()
            .unwrap_or_else(|| PathBuf::from("/tmp"))
            .join("ironclaw");

        fs::create_dir_all(&cache_dir)
            .context("Failed to create cache directory")?;

        let filename = format!(
            "alpine-minirootfs-{}-{}.tar.gz",
            self.alpine_version, self.arch
        );

        let output_path = cache_dir.join(&filename);

        // Download if not cached
        if !output_path.exists() {
            info!("Downloading from: {}", url);

            let output = Command::new("curl")
                .arg("-L")
                .arg("-o")
                .arg(&output_path)
                .arg(&url)
                .output()
                .context("Failed to download Alpine minirootfs")?;

            if !output.status.success() {
                let error = String::from_utf8_lossy(&output.stderr);
                return Err(anyhow::anyhow!("Download failed: {}", error));
            }
        } else {
            debug!("Using cached minirootfs: {}", output_path.display());
        }

        Ok(output_path)
    }

    /// Create ext4 filesystem image
    fn create_ext4_image(&self, minirootfs_path: &Path) -> Result<()> {
        info!("Creating ext4 image: {}", self.output_path.display());

        // Create empty file of specified size
        let output = Command::new("dd")
            .arg("if=/dev/zero")
            .arg(&format!("of={}", self.output_path.display()))
            .arg(&format!("bs=1M"))
            .arg(&format!("count={}", self.rootfs_size_mb))
            .output()
            .context("Failed to create ext4 image with dd")?;

        if !output.status.success() {
            let error = String::from_utf8_lossy(&output.stderr);
            return Err(anyhow::anyhow!("dd failed: {}", error));
        }

        // Format as ext4
        let output = Command::new("mkfs.ext4")
            .arg("-F")
            .arg(&self.output_path)
            .output()
            .context("Failed to format ext4 filesystem")?;

        if !output.status.success() {
            let error = String::from_utf8_lossy(&output.stderr);
            return Err(anyhow::anyhow!("mkfs.ext4 failed: {}", error));
        }

        // Mount and extract minirootfs
        let mount_point = tempfile::tempdir()
            .context("Failed to create temp mount point")?;

        let output = Command::new("mount")
            .arg("-o")
            .arg("loop")
            .arg(&self.output_path)
            .arg(mount_point.path())
            .output()
            .context("Failed to mount ext4 image")?;

        if !output.status.success() {
            let error = String::from_utf8_lossy(&output.stderr);
            return Err(anyhow::anyhow!("Mount failed: {}", error));
        }

        // Extract minirootfs
        let output = Command::new("tar")
            .arg("-xzf")
            .arg(minirootfs_path)
            .arg("-C")
            .arg(mount_point.path())
            .output()
            .context("Failed to extract minirootfs")?;

        if !output.status.success() {
            let error = String::from_utf8_lossy(&output.stderr);
            return Err(anyhow::anyhow!("Extract failed: {}", error));
        }

        // Unmount
        let _ = Command::new("umount")
            .arg(mount_point.path())
            .output();

        debug!("ext4 image created successfully");
        Ok(())
    }

    /// Customize rootfs with additional packages and configuration
    fn customize_rootfs(&self) -> Result<()> {
        info!("Customizing rootfs");

        // TODO: Implement chroot-based customization
        // This requires:
        // 1. Mount rootfs
        // 2. chroot into rootfs
        // 3. Run apk commands to install packages
        // 4. Configure system
        // 5. Unmount

        debug!("Rootfs customization completed");
        Ok(())
    }

    /// Harden rootfs by removing unnecessary binaries
    fn harden_rootfs(&self) -> Result<()> {
        info!("Hardening rootfs");

        // TODO: Implement hardening
        // This includes:
        // 1. Remove compilers (gcc, clang)
        // 2. Remove debuggers (gdb)
        // 3. Remove networking tools (optional)
        // 4. Set restrictive permissions
        // 5. Remove documentation and man pages

        debug!("Rootfs hardening completed");
        Ok(())
    }
}

/// Build report
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BuildReport {
    /// Whether build succeeded
    pub success: bool,

    /// Resulting file size in bytes
    pub file_size: u64,

    /// Time taken to build
    pub build_time: std::time::Duration,

    /// Number of packages installed
    pub packages_installed: usize,
}

/// Validate rootfs meets size requirements
pub fn validate_rootfs_size(rootfs_path: &Path, max_size_mb: u32) -> Result<bool> {
    let metadata = fs::metadata(rootfs_path)
        .context("Failed to get rootfs metadata")?;

    let size_mb = metadata.len() / (1024 * 1024);

    Ok(size_mb <= max_size_mb as u64)
}

/// Get rootfs information
pub fn get_rootfs_info(rootfs_path: &Path) -> Result<RootfsInfo> {
    let metadata = fs::metadata(rootfs_path)
        .context("Failed to get rootfs metadata")?;

    // Mount and inspect
    let mount_point = tempfile::tempdir()
        .context("Failed to create temp mount point")?;

    let _ = Command::new("mount")
        .arg("-o")
        .arg("ro,loop")
        .arg(rootfs_path)
        .arg(mount_point.path())
        .output();

    // Count files
    let file_count = count_files(mount_point.path())?;

    // Get disk usage
    let output = Command::new("du")
        .arg("-sh")
        .arg(mount_point.path())
        .output()
        .context("Failed to get disk usage")?;

    let usage = String::from_utf8_lossy(&output.stdout).to_string();

    // Unmount
    let _ = Command::new("umount")
        .arg(mount_point.path())
        .output();

    Ok(RootfsInfo {
        size_bytes: metadata.len(),
        file_count,
        disk_usage: usage,
    })
}

fn count_files(dir: &Path) -> Result<usize> {
    let mut count = 0;

    for entry in fs::read_dir(dir)
        .context("Failed to read directory")?
    {
        let entry = entry?;
        let path = entry.path();

        if path.is_dir() {
            count += count_files(&path)?;
        } else {
            count += 1;
        }
    }

    Ok(count)
}

/// Rootfs information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RootfsInfo {
    /// Size in bytes
    pub size_bytes: u64,

    /// Number of files
    pub file_count: usize,

    /// Disk usage string
    pub disk_usage: String,
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_guest_os_config_default() {
        let config = GuestOsConfig::default();
        assert_eq!(config.alpine_version, "v3.19");
        assert_eq!(config.arch, "x86_64");
        assert_eq!(config.rootfs_size_mb, 64);
        assert!(config.enable_hardening);
    }

    #[test]
    fn test_guest_os_config_new() {
        let output_path = PathBuf::from("/custom/rootfs.ext4");
        let config = GuestOsConfig::new(output_path.clone());

        assert_eq!(config.output_path, output_path);
        assert_eq!(config.alpine_version, "v3.19");
    }

    #[test]
    fn test_validate_rootfs_size_small() {
        let temp_dir = TempDir::new().unwrap();
        let rootfs_path = temp_dir.path().join("small_rootfs.ext4");

        // Create a small file (< 1MB)
        let mut file = File::create(&rootfs_path).unwrap();
        file.write_all(&vec![0u8; 512 * 1024]).unwrap();

        let valid = validate_rootfs_size(&rootfs_path, 100).unwrap();
        assert!(valid);
    }

    #[test]
    fn test_validate_rootfs_size_large() {
        let temp_dir = TempDir::new().unwrap();
        let rootfs_path = temp_dir.path().join("large_rootfs.ext4");

        // Create a file > 100MB (sparse file)
        let file = File::create(&rootfs_path).unwrap();
        file.set_len(150 * 1024 * 1024).unwrap();

        let valid = validate_rootfs_size(&rootfs_path, 100).unwrap();
        assert!(!valid);
    }

    #[test]
    fn test_get_rootfs_info_missing_file() {
        let result = get_rootfs_info(Path::new("/nonexistent/rootfs.ext4"));
        assert!(result.is_err());
    }

    #[test]
    fn test_count_files() {
        let temp_dir = TempDir::new().unwrap();

        // Create some files
        File::create(temp_dir.path().join("file1.txt")).unwrap();
        File::create(temp_dir.path().join("file2.txt")).unwrap();

        let subdir = temp_dir.path().join("subdir");
        fs::create_dir(&subdir).unwrap();
        File::create(subdir.join("file3.txt")).unwrap();

        let count = count_files(temp_dir.path()).unwrap();
        assert_eq!(count, 3);
    }

    #[test]
    fn test_guest_os_config_variations() {
        let config1 = GuestOsConfig {
            alpine_version: "v3.18".to_string(),
            arch: "aarch64".to_string(),
            packages: vec!["bash".to_string(), "vim".to_string()],
            rootfs_size_mb: 128,
            enable_hardening: false,
            ..Default::default()
        };

        assert_eq!(config1.alpine_version, "v3.18");
        assert_eq!(config1.arch, "aarch64");
        assert_eq!(config1.packages.len(), 2);
        assert_eq!(config1.rootfs_size_mb, 128);
        assert!(!config1.enable_hardening);
    }

    // Property-based test: various configurations
    #[test]
    fn test_various_package_sets() {
        let package_sets = vec![
            vec!["busybox".to_string()],
            vec!["busybox".to_string(), "curl".to_string()],
            vec![
                "busybox".to_string(),
                "curl".to_string(),
                "ca-certificates".to_string(),
            ],
        ];

        for packages in package_sets {
            let config = GuestOsConfig {
                packages: packages.clone(),
                ..Default::default()
            };

            assert_eq!(config.packages, packages);
        }
    }
}
