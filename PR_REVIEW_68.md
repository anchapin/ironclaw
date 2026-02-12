# Review: PR 68 - Identify Critical Collision Vulnerability

## Summary
The PR successfully addresses the firewall chain name collision vulnerability by incorporating a hash of the VM ID into the chain name. This ensures that even if two VM IDs sanitize to the same string (e.g., `vm-1` and `vm_1`), their firewall chains will be unique.

However, there are critical functional gaps and code quality issues that must be addressed before merging.

## Findings

### 🔴 Critical: Missing Vsock Configuration
The PR claims to implement "Network Isolation" where vsock is the only allowed communication channel. However, the `start_firecracker` function in `orchestrator/src/vm/firecracker.rs` **does not configure the vsock device** for the VM.

- **File**: `orchestrator/src/vm/firecracker.rs`
- **Issue**: The `configure_vm` function sets up boot source, rootfs, and machine config, but ignores `config.vsock_path`. The Firecracker API requires a `PUT /vsock` call to enable the device.
- **Impact**: VMs will launch with network disabled (by firewall) and **no vsock device**, rendering them completely isolated and unable to communicate with the agent. The feature is non-functional.

### 🟡 Warning: Formatting Issues (Rust)
The Rust code in `orchestrator/src/vm/tests.rs` fails `cargo fmt --check`.
- **File**: `orchestrator/src/vm/tests.rs`
- **Lines**: 127-145 (indentation issues in `test_firewall_sanitizes_vm_ids`)

### 🟡 Warning: Formatting Issues (Python)
The Python code in `agent/mcp_client.py` fails `black --check`.
- **File**: `agent/mcp_client.py`
- **Action**: Please run `black agent/` to fix formatting.

### 💡 Suggestion: Sanitization Discrepancy
There is a minor discrepancy between the implementation and tests for sanitization.
- **File**: `orchestrator/src/vm/firewall.rs` uses `c.is_ascii_alphanumeric()`
- **File**: `orchestrator/src/vm/tests.rs` uses `c.is_alphanumeric()`
- **Impact**: Minimal, as the implementation is stricter (ASCII only), which is safer. However, tests should match implementation logic to avoid confusion.

### 💡 Suggestion: Vsock Path Security
The `VmConfig::new` method uses the raw `vm_id` to generate the vsock path:
```rust
config.vsock_path = Some(format!("/tmp/ironclaw/vsock/{}.sock", config.vm_id));
```
If `vm_id` contains path traversal characters (e.g., `../`), this could be exploited. While `FirewallManager` sanitizes the ID for iptables, `VmConfig` does not sanitize it for the socket path.
- **Recommendation**: Sanitize `vm_id` before using it in the path, or use a UUID.

## Conclusion
**Request Changes**

Please address the critical missing vsock configuration and formatting issues. The collision fix itself is solid and verified by tests.
