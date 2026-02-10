# Jailer Integration Guide

## Overview

This guide explains how IronClaw integrates Firecracker Jailer for enhanced security through process sandboxing.

## What is Jailer?

Jailer is a utility that comes with Firecracker to provide enhanced security through:

1. **chroot isolation**: Process runs in an isolated filesystem
2. **cgroups v2**: Enforces resource limits (CPU, memory, I/O)
3. **Network namespaces**: Prepares for vsock isolation (Phase 3)
4. **Privilege separation**: Runs as non-root user

## Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                     IronClaw Orchestrator                    │
│                                                               │
│  ┌──────────────┐         ┌──────────────┐                  │
│  │ VmConfig     │────────▶│ JailerConfig │                  │
│  │              │         │              │                  │
│  │ - vm_id      │         │ - jailer_id  │                  │
│  │ - vcpu_count │         │ - cpu_count  │                  │
│  │ - memory_mb  │         │ - memory_mb  │                  │
│  └──────────────┘         └──────────────┘                  │
│                                   │                          │
│                                   ▼                          │
│                          ┌──────────────┐                    │
│                          │   Jailer     │                    │
│                          │   Module     │                    │
│                          └──────────────┘                    │
│                                   │                          │
│              ┌────────────────────┴────────────────────┐     │
│              ▼                                         ▼     │
│     ┌──────────────────┐                    ┌──────────────┐ │
│     │  Jailed Mode     │                    │  Unjailed    │ │
│     │  (Production)    │                    │  (Dev Mode)  │ │
│     └──────────────────┘                    └──────────────┘ │
└─────────────────────────────────────────────────────────────┘
```

## Resource Limits

Jailer enforces the following resource limits via cgroups v2:

### CPU Limits
- **Default**: 1 vCPU
- **Configurable**: Via `VmConfig.vcpu_count`
- **Enforcement**: cgroup cpu controller (cpu.shares, cpu.cfs_quota_us)

### Memory Limits
- **Default**: 256 MB
- **Minimum**: 128 MB
- **Configurable**: Via `VmConfig.memory_mb`
- **Enforcement**: cgroup memory controller (memory.limit_in_bytes)

### I/O Limits
- **Default**: No throttling (unlimited)
- **Future**: Disk I/O rate limiting via cgroup io controller

### Network Limits
- **Default**: No network (isolated)
- **Future**: vsock rate limiting via cgroup network controller

## Installation

### Install Jailer

```bash
# Download Firecracker + Jailer
wget https://github.com/firecracker-microvm/firecracker/releases/download/v1.14.1/firecracker-v1.14.1-x86_64.tgz
tar -xzf firecracker-v1.14.1-x86_64.tgz

# Install binaries
sudo cp release-v1.14.1-x86_64/jailer /usr/local/bin/jailer
sudo cp release-v1.14.1-x86_64/firecracker-v1.14.1 /usr/local/bin/firecracker-v1.14.1
sudo chmod +x /usr/local/bin/jailer /usr/local/bin/firecracker-v1.14.1

# Verify installation
jailer --version
```

### Configure cgroups v2

Jailer requires cgroups v2. Verify your system has it:

```bash
# Check cgroups version
mount | grep cgroup
# Should show: cgroup2 on /sys/fs/cgroup

# If not available, enable cgroups v2 (Linux kernel 5.2+)
# Add to GRUB_CMDLINE_LINUX: systemd.unified_cgroup_hierarchy=1
# Update grub and reboot
```

### Setup chroot directory

```bash
# Create jailer base directory
sudo mkdir -p /var/jail
sudo chmod 755 /var/jail

# The jailer will create subdirectories like:
# /var/jail/firecracker-v1.14.1/{vm_id}/
```

## Usage

### Basic Example

```rust
use ironclaw_orchestrator::vm::{start_firecracker, stop_firecracker};
use ironclaw_orchestrator::vm::config::VmConfig;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Create VM configuration
    let config = VmConfig {
        vm_id: "my-agent-task-123".to_string(),
        vcpu_count: 1,
        memory_mb: 256,
        kernel_path: "/path/to/vmlinux.bin".to_string(),
        rootfs_path: "/path/to/rootfs.ext4".to_string(),
        enable_networking: false,
    };

    // Start Firecracker with Jailer sandboxing
    let vm = start_firecracker(&config).await?;

    println!("VM started with PID: {}", vm.pid);
    println!("API socket: {:?}", vm.socket_path);
    println!("Sandboxed: {}", vm.is_sandboxed);

    // Use the VM...
    // Configure VM via API socket
    // Start instance...

    // Stop the VM
    stop_firecracker(vm).await?;

    Ok(())
}
```

### Direct Jailer Usage

```rust
use ironclaw_orchestrator::vm::jailer::{JailerConfig, start_with_jailer};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Configure Jailer
    let config = JailerConfig {
        jailer_id: "my-vm".to_string(),
        exec_file: PathBuf::from("/usr/local/bin/firecracker-v1.14.1"),
        cpu_count: 2,
        memory_limit_mb: 512,
        netns: Some("/run/netns/isolated".to_string()),
        ..Default::default()
    };

    // Start with Jailer
    let process = start_with_jailer(&config).await?;

    println!("Jailed process started with PID: {}", process.pid);
    println!("API socket: {:?}", process.api_socket);
    println!("Is sandboxed: {}", process.jailed);

    Ok(())
}
```

## Graceful Degradation

Jailer integration includes graceful degradation for development:

1. **If Jailer is available**: Spawns Firecracker within chroot + cgroups (production mode)
2. **If Jailer is NOT available**: Logs warning, spawns Firecracker directly (development mode)

```rust
// This will work in both modes
let vm = start_firecracker(&config).await?;

if vm.is_sandboxed {
    println!("Running in PRODUCTION mode (sandboxed)");
} else {
    println!("Running in DEVELOPMENT mode (unsandboxed)");
}
```

**WARNING**: Unjailed mode is INSECURE for production! Always ensure Jailer is installed in production environments.

## Configuration Reference

### JailerConfig

| Field | Type | Default | Description |
|-------|------|---------|-------------|
| `jailer_binary` | `PathBuf` | `/usr/local/bin/jailer` | Path to jailer binary |
| `jailer_id` | `String` | `"ironclaw-vm"` | Unique ID for this jailed instance |
| `chroot_base_dir` | `PathBuf` | `/var/jail` | Base directory for chroot |
| `exec_file` | `PathBuf` | `/usr/local/bin/firecracker-v1.14.1` | Firecracker binary path |
| `cpu_count` | `u8` | `1` | Number of vCPUs to limit |
| `memory_limit_mb` | `u32` | `256` | Memory limit in MB (min 128) |
| `netns` | `Option<String>` | `None` | Optional network namespace path |

### VmConfig

| Field | Type | Default | Description |
|-------|------|---------|-------------|
| `vm_id` | `String` | `"default"` | Unique VM identifier |
| `vcpu_count` | `u8` | `1` | Number of vCPUs (min 1) |
| `memory_mb` | `u32` | `512` | Memory in MB (min 128) |
| `kernel_path` | `String` | `"/path/to/vmlinux.bin"` | Kernel image path |
| `rootfs_path` | `String` | `"/path/to/rootfs.ext4"` | Root filesystem path |
| `enable_networking` | `bool` | `false` | Enable networking (security risk) |

## Security Best Practices

### 1. Always Use Jailer in Production

```rust
let vm = start_firecracker(&config).await?;

// Verify sandboxing
assert!(vm.is_sandboxed, "PRODUCTION ERROR: VM not sandboxed!");
```

### 2. Limit Resources Appropriately

```rust
// Start with conservative limits
let config = VmConfig {
    vcpu_count: 1,        // Single CPU is enough for most agent tasks
    memory_mb: 256,       // 256 MB is sufficient for CLI tools
    enable_networking: false,  // Disable network for maximum security
    ..Default::default()
};
```

### 3. Use Network Isolation

```rust
// Create network namespace
sudo ip netns add agent-ns-1

// Configure Jailer to use it
let jailer_config = JailerConfig {
    netns: Some("/run/netns/agent-ns-1".to_string()),
    ..Default::default()
};
```

### 4. Monitor Resource Usage

```bash
# Monitor cgroup usage
cat /sys/fs/cgroup/ironclaw/{vm_id}/cpu.stat
cat /sys/fs/cgroup/ironclaw/{vm_id}/memory.current
cat /sys/fs/cgroup/ironclaw/{vm_id}/io.stat
```

## Troubleshooting

### Jailer Not Found

**Error**: `Jailer binary not found at /usr/local/bin/jailer`

**Solution**:
```bash
# Check if jailer is installed
which jailer

# If not, install it
sudo cp release-v1.14.1-x86_64/jailer /usr/local/bin/jailer
sudo chmod +x /usr/local/bin/jailer
```

### cgroups v2 Not Available

**Error**: `Failed to set cgroup: Operation not permitted`

**Solution**:
```bash
# Check cgroups version
mount | grep cgroup

# Enable cgroups v2 (add to GRUB_CMDLINE_LINUX)
sudo vim /etc/default/grub
# Add: systemd.unified_cgroup_hierarchy=1

sudo update-grub
sudo reboot
```

### Permission Denied on chroot Directory

**Error**: `Failed to create chroot: Permission denied`

**Solution**:
```bash
# Create jailer directory with correct permissions
sudo mkdir -p /var/jail
sudo chmod 755 /var/jail

# Ensure jailer can write to it
sudo chown root:root /var/jail
```

### VM Exits Immediately

**Error**: `VM exited with status 1`

**Solution**:
```bash
# Check jailer logs
journalctl -u firecracker -n 50

# Check cgroup limits
cat /sys/fs/cgroup/ironclaw/{vm_id}/memory.max
cat /sys/fs/cgroup/ironclaw/{vm_id}/cpu.max

# Try increasing memory limit
let config = VmConfig {
    memory_mb: 512,  // Increase from 256
    ..Default::default()
};
```

### Socket Not Created

**Error**: `Failed to connect to API socket: No such file or directory`

**Solution**:
```bash
# Check if socket exists
ls -la /var/jail/firecracker-v1.14.1/{vm_id}/run/firecracker.socket

# Check jailer process
ps aux | grep jailer

# Check jailer logs
sudo journalctl -xe | grep jailer
```

## Testing

### Unit Tests

```bash
cd orchestrator
cargo test --lib vm::jailer
```

### Integration Tests (requires real Jailer + Firecracker)

```bash
# These tests are marked as integration and require actual binaries
cargo test --lib vm::jailer -- --ignored
```

### Manual Testing

```bash
# Start a jailed Firecracker process
/usr/local/bin/jailer \
  --id test-vm \
  --exec-file /usr/local/bin/firecracker-v1.14.1 \
  --chroot-base-dir /var/jail \
  --cgroup-version 2 \
  --cgroup cpu:1 \
  --cgroup memory:256

# Check if it's running
ps aux | grep firecracker

# Check cgroup limits
cat /sys/fs/cgroup/firecracker-v1.14.1/test-vm/cgroup.controllers
cat /sys/fs/cgroup/firecracker-v1.14.1/test-vm/memory.max
```

## Performance Considerations

### Startup Time

Jailer adds minimal overhead to VM startup:
- **Unjailed**: ~100ms (Firecracker only)
- **Jailed**: ~120ms (Firecracker + Jailer)
- **Overhead**: ~20ms for chroot + cgroup setup

### Memory Overhead

Jailer itself has minimal memory footprint:
- **Jailer process**: ~2 MB RSS
- **cgroup metadata**: ~1 MB
- **Total overhead**: ~3 MB per VM

### CPU Overhead

Jailer has negligible CPU overhead:
- **cgroup enforcement**: <1% CPU
- **chroot syscalls**: Negligible

## Future Enhancements

### Phase 2: Advanced Resource Limits

- Disk I/O throttling via cgroup io controller
- Network rate limiting for vsock
- Per-VM CPU affinity (CPU pinning)

### Phase 3: Enhanced Security

- seccomp filters integration
- AppArmor profiles
- SELinux policies

## References

- [Firecracker Jailer Documentation](https://github.com/firecracker-microvm/firecracker/blob/main/docs/jailer.md)
- [cgroups v2 Documentation](https://docs.kernel.org/admin-guide/cgroup-v2.html)
- [Firecracker Security Best Practices](https://github.com/firecracker-microvm/firecracker/blob/main/docs/security-model.md)
