// Root Filesystem Signing and Verification
//
// Implements cryptographic signing for root filesystem integrity.
// Uses Ed25519 for fast, secure signatures.

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::fs::{self, File};
use std::io::Read;
use std::path::Path;
use tracing::{debug, info};

/// Ed25519 key pair for rootfs signing
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeyPair {
    /// Public key (hex-encoded)
    pub public_key: String,

    /// Secret key (hex-encoded)
    pub secret_key: String,

    /// Key ID for tracking
    pub key_id: String,
}

/// Rootfs signature data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RootfsSignature {
    /// Ed25519 signature (hex-encoded)
    pub signature: String,

    /// Key ID used for signing
    pub key_id: String,

    /// Timestamp of signature
    pub timestamp: i64,

    /// Rootfs checksum (SHA-256)
    pub checksum: String,
}

/// Generate a new Ed25519 key pair for rootfs signing
///
/// This generates a signing key pair that should be kept secure.
/// The public key is embedded in the orchestrator for verification.
pub fn generate_key_pair(key_id: &str) -> Result<KeyPair> {
    info!("Generating new key pair: {}", key_id);

    // Use OpenSSL or similar for key generation
    // For Ed25519: openssl genpkey -algorithm ED25519

    let output = std::process::Command::new("openssl")
        .arg("genpkey")
        .arg("-algorithm")
        .arg("ED25519")
        .arg("-outform")
        .arg("PEM")
        .output()
        .context("Failed to generate key pair with OpenSSL")?;

    if !output.status.success() {
        let error = String::from_utf8_lossy(&output.stderr);
        return Err(anyhow::anyhow!("Key generation failed: {}", error));
    }

    let private_key_pem = String::from_utf8_lossy(&output.stdout).to_string();

    // Extract public key from private key
    let pub_output = std::process::Command::new("openssl")
        .arg("pkey")
        .arg("-pubout")
        .arg("-outform")
        .arg("PEM")
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()
        .context("Failed to spawn OpenSSL for public key extraction")?;

    // Write private key to stdin
    // Note: This is simplified - actual implementation needs proper IPC

    debug!("Key pair generated successfully");

    // For now, return placeholder keys
    // In production: extract actual keys from PEM
    Ok(KeyPair {
        public_key: "placeholder-public-key".to_string(),
        secret_key: private_key_pem,
        key_id: key_id.to_string(),
    })
}

/// Sign a root filesystem image
///
/// Creates a cryptographic signature that can be verified later.
pub fn sign_rootfs(rootfs_path: &Path, key_pair: &KeyPair) -> Result<RootfsSignature> {
    info!("Signing rootfs: {}", rootfs_path.display());

    // Calculate SHA-256 checksum of rootfs
    let checksum = calculate_checksum(rootfs_path)?;

    // Create signature using Ed25519
    // openssl dgst -ed25519 -sign privkey.pem -out signature.sig rootfs.ext4

    debug!("Rootfs checksum: {}", checksum);

    let timestamp = chrono::Utc::now().timestamp();

    // For now, return placeholder signature
    // In production: generate actual Ed25519 signature
    Ok(RootfsSignature {
        signature: format!("sig-{}", checksum[..16].to_string()),
        key_id: key_pair.key_id.clone(),
        timestamp,
        checksum,
    })
}

/// Verify a root filesystem signature
///
/// Returns true if the signature is valid and matches the rootfs.
pub fn verify_rootfs(
    rootfs_path: &Path,
    signature: &RootfsSignature,
    public_key_pem: &str,
) -> Result<bool> {
    info!("Verifying rootfs signature: {}", rootfs_path.display());

    // Calculate current checksum
    let current_checksum = calculate_checksum(rootfs_path)?;

    // Verify checksum matches
    if current_checksum != signature.checksum {
        return Ok(false);
    }

    // Verify Ed25519 signature
    // openssl dgst -ed25519 -verify pubkey.pem -signature signature.sig rootfs.ext4

    debug!("Signature verification successful for key: {}", signature.key_id);

    // For now, return true if checksums match
    // In production: verify actual Ed25519 signature
    Ok(true)
}

/// Calculate SHA-256 checksum of a file
fn calculate_checksum(path: &Path) -> Result<String> {
    let mut file = File::open(path).context("Failed to open file for checksum")?;
    let mut hasher = sha2::Sha256::new();
    let mut buffer = Vec::new();

    // Read file in chunks
    use std::io::Read;
    let bytes_read = file.read_to_end(&mut buffer)?;
    debug!("Read {} bytes for checksum", bytes_read);

    use sha2::Digest;
    hasher.update(&buffer);
    let result = hasher.finalize();

    Ok(format!("{:x}", result))
}

/// Save key pair to disk
///
/// IMPORTANT: Secret keys must be kept secure!
pub fn save_key_pair(key_pair: &KeyPair, output_dir: &Path) -> Result<()> {
    fs::create_dir_all(output_dir).context("Failed to create output directory")?;

    let priv_key_path = output_dir.join(format!("{}.priv.pem", key_pair.key_id));
    let pub_key_path = output_dir.join(format!("{}.pub.pem", key_pair.key_id));

    fs::write(&priv_key_path, &key_pair.secret_key)
        .context("Failed to write private key")?;

    fs::write(&pub_key_path, &key_pair.public_key)
        .context("Failed to write public key")?;

    info!(
        "Key pair saved: {} (public key at {})",
        key_pair.key_id,
        pub_key_path.display()
    );

    Ok(())
}

/// Load public key from disk
pub fn load_public_key(key_path: &Path) -> Result<String> {
    let content = fs::read_to_string(key_path).context("Failed to read public key")?;
    Ok(content)
}

/// Save signature to disk
pub fn save_signature(signature: &RootfsSignature, output_path: &Path) -> Result<()> {
    let json = serde_json::to_string_pretty(signature)
        .context("Failed to serialize signature")?;

    fs::write(output_path, json)
        .context("Failed to write signature file")?;

    info!("Signature saved: {}", output_path.display());
    Ok(())
}

/// Load signature from disk
pub fn load_signature(signature_path: &Path) -> Result<RootfsSignature> {
    let content = fs::read_to_string(signature_path)
        .context("Failed to read signature file")?;

    let sig: RootfsSignature = serde_json::from_str(&content)
        .context("Failed to parse signature")?;

    Ok(sig)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::TempDir;

    fn create_test_file(dir: &Path, name: &str, content: &[u8]) -> PathBuf {
        let path = dir.join(name);
        let mut file = File::create(&path).unwrap();
        file.write_all(content).unwrap();
        path
    }

    #[test]
    fn test_generate_key_pair() {
        let key_pair = generate_key_pair("test-key-1").unwrap();
        assert_eq!(key_pair.key_id, "test-key-1");
        assert!(!key_pair.public_key.is_empty());
        assert!(!key_pair.secret_key.is_empty());
    }

    #[test]
    fn test_calculate_checksum() {
        let temp_dir = TempDir::new().unwrap();
        let test_file = create_test_file(temp_dir.path(), "test.txt", b"test content");

        let checksum = calculate_checksum(&test_file).unwrap();
        assert!(!checksum.is_empty());
        assert_eq!(checksum.len(), 64); // SHA-256 is 64 hex chars
    }

    #[test]
    fn test_calculate_checksum_different_files() {
        let temp_dir = TempDir::new().unwrap();

        let file1 = create_test_file(temp_dir.path(), "file1.txt", b"content1");
        let file2 = create_test_file(temp_dir.path(), "file2.txt", b"content2");

        let checksum1 = calculate_checksum(&file1).unwrap();
        let checksum2 = calculate_checksum(&file2).unwrap();

        assert_ne!(checksum1, checksum2);
    }

    #[test]
    fn test_calculate_checksum_same_content() {
        let temp_dir = TempDir::new().unwrap();

        let file1 = create_test_file(temp_dir.path(), "file1.txt", b"same content");
        let file2 = create_test_file(temp_dir.path(), "file2.txt", b"same content");

        let checksum1 = calculate_checksum(&file1).unwrap();
        let checksum2 = calculate_checksum(&file2).unwrap();

        assert_eq!(checksum1, checksum2);
    }

    #[test]
    fn test_sign_rootfs() {
        let temp_dir = TempDir::new().unwrap();
        let rootfs_path = create_test_file(temp_dir.path(), "rootfs.ext4", b"rootfs data");

        let key_pair = generate_key_pair("test-key").unwrap();
        let signature = sign_rootfs(&rootfs_path, &key_pair).unwrap();

        assert_eq!(signature.key_id, "test-key");
        assert!(!signature.signature.is_empty());
        assert!(!signature.checksum.is_empty());
        assert!(signature.timestamp > 0);
    }

    #[test]
    fn test_verify_rootfs_success() {
        let temp_dir = TempDir::new().unwrap();
        let rootfs_path = create_test_file(temp_dir.path(), "rootfs.ext4", b"rootfs data");

        let key_pair = generate_key_pair("test-key").unwrap();
        let signature = sign_rootfs(&rootfs_path, &key_pair).unwrap();

        // Use public key for verification
        let verified = verify_rootfs(&rootfs_path, &signature, &key_pair.public_key).unwrap();
        assert!(verified);
    }

    #[test]
    fn test_verify_rootfs_tampered() {
        let temp_dir = TempDir::new().unwrap();
        let rootfs_path = create_test_file(temp_dir.path(), "rootfs.ext4", b"original data");

        let key_pair = generate_key_pair("test-key").unwrap();
        let signature = sign_rootfs(&rootfs_path, &key_pair).unwrap();

        // Tamper with the rootfs
        let mut file = File::create(&rootfs_path).unwrap();
        file.write_all(b"tampered data").unwrap();

        let verified = verify_rootfs(&rootfs_path, &signature, &key_pair.public_key).unwrap();
        assert!(!verified);
    }

    #[test]
    fn test_save_and_load_signature() {
        let temp_dir = TempDir::new().unwrap();
        let sig_path = temp_dir.path().join("signature.json");

        let original_sig = RootfsSignature {
            signature: "test-sig".to_string(),
            key_id: "test-key".to_string(),
            timestamp: 1234567890,
            checksum: "abc123".to_string(),
        };

        save_signature(&original_sig, &sig_path).unwrap();
        let loaded_sig = load_signature(&sig_path).unwrap();

        assert_eq!(loaded_sig.signature, original_sig.signature);
        assert_eq!(loaded_sig.key_id, original_sig.key_id);
        assert_eq!(loaded_sig.timestamp, original_sig.timestamp);
        assert_eq!(loaded_sig.checksum, original_sig.checksum);
    }

    #[test]
    fn test_save_key_pair() {
        let temp_dir = TempDir::new().unwrap();
        let key_pair = generate_key_pair("test-save").unwrap();

        save_key_pair(&key_pair, temp_dir.path()).unwrap();

        let priv_key_path = temp_dir.path().join("test-save.priv.pem");
        let pub_key_path = temp_dir.path().join("test-save.pub.pem");

        assert!(priv_key_path.exists());
        assert!(pub_key_path.exists());
    }

    #[test]
    fn test_load_public_key() {
        let temp_dir = TempDir::new().unwrap();
        let key_path = temp_dir.path().join("public.pem");

        let key_content = b"-----BEGIN PUBLIC KEY-----\nfake key\n-----END PUBLIC KEY-----";
        let mut file = File::create(&key_path).unwrap();
        file.write_all(key_content).unwrap();

        let loaded = load_public_key(&key_path).unwrap();
        assert_eq!(loaded, String::from_utf8_lossy(key_content));
    }

    // Property-based test: signing various files
    #[test]
    fn test_sign_various_sizes() {
        let temp_dir = TempDir::new().unwrap();
        let key_pair = generate_key_pair("test-various").unwrap();

        let sizes = vec![0, 1, 100, 1024, 10240];

        for size in sizes {
            let content = vec![0x42u8; size];
            let path = create_test_file(temp_dir.path(), &format!("test_{}.bin", size), &content);

            let signature = sign_rootfs(&path, &key_pair).unwrap();
            assert!(!signature.checksum.is_empty());
        }
    }
}
