# Jailer Troubleshooting Guide

This guide helps you diagnose and fix common issues with Jailer integration.

## Quick Diagnosis Checklist

Before diving into specific issues, run this checklist:

```bash
# 1. Check if Jailer is installed
which jailer
# Expected: /usr/local/bin/jailer

# 2. Check Jailer version
jailer --version
# Expected: Jailer version x.y.z

# 3. Check cgroups v2
mount | grep cgroup
# Expected: cgroup2 on /sys/fs/cgroup type cgroup2

# 4. Check Firecracker binary
ls -la /usr/local/bin/firecracker-v1.14.1
# Expected: executable file

# 5. Check chroot directory
ls -la /var/jail
# Expected: directory exists with 755 permissions

# 6. Check running jailer processes
ps aux | grep jailer
# Expected: jailer processes if VMs are running
```

## Common Issues

### Issue 1: Jailer Binary Not Found

**Symptoms**:
- Error: `Jailer binary not found at /usr/local/bin/jailer`
- VM starts in unjailed mode (warning logged)
- Insecure for production

**Diagnosis**:
```bash
# Check if jailer is in PATH
which jailer

# Check if it exists in standard location
ls -la /usr/local/bin/jailer
```

**Solutions**:

**Option A: Install Jailer**
```bash
# Download Firecracker release
wget https://github.com/firecracker-microvm/firecracker/releases/download/v1.14.1/firecracker-v1.14.1-x86_64.tgz
tar -xzf firecracker-v1.14.1-x86_64.tgz

# Copy jailer binary
sudo cp release-v1.14.1-x86_64/jailer /usr/local/bin/jailer
sudo chmod +x /usr/local/bin/jailer

# Verify
jailer --version
```

**Option B: Update Configuration**
```rust
// If jailer is installed in a different location
let config = JailerConfig {
    jailer_binary: PathBuf::from("/custom/path/to/jailer"),
    ..Default::default()
};
```

**Option C: Build from Source**
```bash
# Clone Firecracker repository
git clone https://github.com/firecracker-microvm/firecracker.git
cd firecracker

# Build jailer
cargo build --release --bin jailer

# Install
sudo cp target/release/jailer /usr/local/bin/jailer
```

---

### Issue 2: cgroups v2 Not Available

**Symptoms**:
- Error: `Failed to set cgroup: Operation not permitted`
- Error: `Failed to create cgroup: No such file or directory`
- Jailer fails to start

**Diagnosis**:
```bash
# Check cgroups version
mount | grep cgroup
# If you see "cgroup" but not "cgroup2", you're using cgroups v1

# Check kernel version
uname -r
# cgroups v2 requires kernel 5.2+
```

**Solutions**:

**Option A: Enable cgroups v2 (Recommended)**

Edit `/etc/default/grub`:
```bash
sudo vim /etc/default/grub

# Add to GRUB_CMDLINE_LINUX
GRUB_CMDLINE_LINUX="systemd.unified_cgroup_hierarchy=1"

# Update grub and reboot
sudo update-grub
sudo reboot
```

**Option B: Use cgroups v1 (Not Recommended)**

Jailer for cgroups v1 requires different configuration:
```bash
# Jailer v1 arguments (unsupported by IronClaw)
jailer --id test --exec-file /usr/local/bin/firecracker-v1.14.1 ...
```

**Note**: IronClaw only supports cgroups v2. Use cgroups v2 for production.

---

### Issue 3: Permission Denied on chroot Directory

**Symptoms**:
- Error: `Failed to create chroot: Permission denied`
- Error: `Failed to chroot: Operation not permitted`

**Diagnosis**:
```bash
# Check chroot base directory
ls -la /var/jail
# Expected: drwxr-xr-x  root root

# Check current user
whoami
# Expected: root (or user with sudo privileges)
```

**Solutions**:

**Option A: Create Directory with Correct Permissions**
```bash
# Create base directory
sudo mkdir -p /var/jail
sudo chmod 755 /var/jail
sudo chown root:root /var/jail

# Verify
ls -la /var/jail
```

**Option B: Use Different Base Directory (Development Only)**
```rust
// Use a user-writable directory for development
let config = JailerConfig {
    chroot_base_dir: PathBuf::from("/tmp/jailer-test"),
    ..Default::default()
};
```

**Warning**: Only use non-standard paths for development. Production must use `/var/jail`.

---

### Issue 4: Firecracker Binary Not Found

**Symptoms**:
- Error: `Firecracker binary not found at /usr/local/bin/firecracker-v1.14.1`
- Jailer fails to start

**Diagnosis**:
```bash
# Check if Firecracker exists
ls -la /usr/local/bin/firecracker-v1.14.1

# Check version
/usr/local/bin/firecracker-v1.14.1 --version
```

**Solutions**:

**Option A: Install Firecracker**
```bash
# Download and extract
wget https://github.com/firecracker-microvm/firecracker/releases/download/v1.14.1/firecracker-v1.14.1-x86_64.tgz
tar -xzf firecracker-v1.14.1-x86_64.tgz

# Copy binary
sudo cp release-v1.14.1-x86_64/firecracker-v1.14.1 /usr/local/bin/firecracker-v1.14.1
sudo chmod +x /usr/local/bin/firecracker-v1.14.1

# Verify
/usr/local/bin/firecracker-v1.14.1 --version
```

**Option B: Create Symlink**
```bash
# If Firecracker is installed elsewhere
sudo ln -s /path/to/firecracker /usr/local/bin/firecracker-v1.14.1
```

**Option C: Update Configuration**
```rust
// Use a different Firecracker path
let config = JailerConfig {
    exec_file: PathBuf::from("/custom/path/to/firecracker"),
    ..Default::default()
};
```

---

### Issue 5: VM Exits Immediately After Start

**Symptoms**:
- VM starts but exits immediately
- Error: `VM exited with status 1`
- No useful error messages

**Diagnosis**:
```bash
# Check jailer logs
sudo journalctl -xe | grep -A 20 jailer

# Check cgroup limits
cat /sys/fs/cgroup/firecracker-v1.14.1/{vm_id}/memory.max
cat /sys/fs/cgroup/firecracker-v1.14.1/{vm_id}/cpu.max

# Check if process is running
ps aux | grep firecracker
```

**Solutions**:

**Option A: Check Resource Limits**
```bash
# Increase memory limit
echo 512M | sudo tee /sys/fs/cgroup/firecracker-v1.14.1/{vm_id}/memory.max

# Or in code:
let config = VmConfig {
    memory_mb: 512,  // Increase from 256
    ..Default::default()
};
```

**Option B: Check Kernel and Rootfs Paths**
```rust
// Ensure paths are correct
let config = VmConfig {
    kernel_path: "/absolute/path/to/vmlinux.bin".to_string(),
    rootfs_path: "/absolute/path/to/rootfs.ext4".to_string(),
    ..Default::default()
};

// Validate paths exist
assert!(Path::new(&config.kernel_path).exists());
assert!(Path::new(&config.rootfs_path).exists());
```

**Option C: Enable Debug Logging**
```bash
# Set RUST_LOG environment variable
export RUST_LOG=debug

# Run with verbose logging
cargo run --bin ironclaw
```

---

### Issue 6: API Socket Not Created

**Symptoms**:
- Error: `Failed to connect to API socket: No such file or directory`
- Cannot configure VM via API

**Diagnosis**:
```bash
# Check if socket exists
ls -la /var/jail/firecracker-v1.14.1/{vm_id}/run/firecracker.socket

# Check if jailer process is running
ps aux | grep jailer

# Check jailer logs
sudo journalctl -u firecracker -n 50
```

**Solutions**:

**Option A: Wait for Socket Creation**
```rust
use tokio::time::{sleep, Duration};

// Socket creation may take a few milliseconds
sleep(Duration::from_millis(100)).await;

// Then connect
let socket = config.api_socket_path();
```

**Option B: Check Socket Path**
```rust
// Get correct socket path
let socket_path = jailer_config.api_socket_path();
println!("API socket: {:?}", socket_path);

// Verify path format
assert!(socket_path.starts_with("/var/jail/"));
assert!(socket_path.ends_with("run/firecracker.socket"));
```

**Option C: Restart Jailer**
```bash
# Kill existing jailer process
sudo pkill -9 jailer

# Clean up chroot directory
sudo rm -rf /var/jail/firecracker-v1.14.1/{vm_id}

# Start again
```

---

### Issue 7: Resource Limits Not Enforced

**Symptoms**:
- VM uses more CPU/memory than configured
- cgroup limits not working

**Diagnosis**:
```bash
# Check cgroup version
cat /sys/fs/cgroup/cgroup.controllers
# Should include: cpu memory io

# Check specific VM cgroup
cat /sys/fs/cgroup/firecracker-v1.14.1/{vm_id}/memory.max
cat /sys/fs/cgroup/firecracker-v1.14.1/{vm_id}/cpu.max

# Check current usage
cat /sys/fs/cgroup/firecracker-v1.14.1/{vm_id}/memory.current
cat /sys/fs/cgroup/firecracker-v1.14.1/{vm_id}/cpu.stat
```

**Solutions**:

**Option A: Enable cgroup Controllers**
```bash
# Add controllers to subtree_control
sudo bash -c 'echo "+cpu +memory +io" > /sys/fs/cgroup/cgroup.subtree_control'
```

**Option B: Check Jailer Arguments**
```bash
# Verify Jailer is called with cgroup arguments
ps aux | grep jailer
# Should include: --cgroup cpu:1 --cgroup memory:256
```

**Option C: Verify Configuration**
```rust
// Ensure limits are set
let config = JailerConfig {
    cpu_count: 1,
    memory_limit_mb: 256,
    ..Default::default()
};

let args = config.build_args();
assert!(args.contains(&"--cgroup".to_string()));
assert!(args.contains(&"cpu:1".to_string()));
assert!(args.contains(&"memory:256".to_string()));
```

---

### Issue 8: Development Mode Warning

**Symptoms**:
- Warning: `Jailer not available, starting Firecracker without sandboxing (INSECURE for production!)`
- VM starts in unjailed mode

**Diagnosis**:
```bash
# Check if jailer binary exists
which jailer

# Check if it's executable
ls -la /usr/local/bin/jailer
```

**Solutions**:

**Option A: Install Jailer** (Recommended)
```bash
# See Issue 1 solutions
```

**Option B: Accept Development Mode** (Development Only)
```rust
// For development only, this is acceptable
let vm = start_firecracker(&config).await?;

if !vm.is_sandboxed {
    println!("WARNING: Running in development mode (no sandboxing)");
}
```

**Option C: Enforce Sandboxing in Production**
```rust
// Panic if not sandboxed (production code)
let vm = start_firecracker(&config).await?;
assert!(vm.is_sandboxed, "PRODUCTION ERROR: VM must be sandboxed!");
```

---

## Advanced Debugging

### Enable Detailed Logging

```bash
# Set environment variable
export RUST_LOG=ironclaw_orchestrator::vm::jailer=debug,ironclaw_orchestrator::vm::firecracker=debug

# Run with logging
cargo run --bin ironclaw
```

### Monitor cgroup Usage

```bash
# Real-time monitoring
watch -n 1 'cat /sys/fs/cgroup/firecracker-v1.14.1/{vm_id}/memory.current'

# CPU usage
watch -n 1 'cat /sys/fs/cgroup/firecracker-v1.14.1/{vm_id}/cpu.stat'

# I/O stats
watch -n 1 'cat /sys/fs/cgroup/firecracker-v1.14.1/{vm_id}/io.stat'
```

### Trace System Calls

```bash
# Trace jailer syscalls
sudo strace -f -p $(pgrep jailer)

# Trace firecracker syscalls
sudo strace -f -p $(pgrep firecracker)
```

### Inspect chroot Environment

```bash
# Enter chroot (for debugging)
sudo chroot /var/jail/firecracker-v1.14.1/{vm_id} /bin/bash

# List files in chroot
ls -la /var/jail/firecracker-v1.14.1/{vm_id}/

# Check socket permissions
ls -la /var/jail/firecracker-v1.14.1/{vm_id}/run/
```

## Getting Help

If you're still stuck after trying these solutions:

1. **Check the logs**: `sudo journalctl -xe | grep -E "(jailer|firecracker)"`

2. **Run in debug mode**: `RUST_LOG=debug cargo run`

3. **Check GitHub Issues**: https://github.com/firecracker-microvm/firecracker/issues

4. **File an Issue**: https://github.com/your-username/ironclaw/issues

When filing an issue, include:
- Jailer version: `jailer --version`
- cgroups version: `mount | grep cgroup`
- Kernel version: `uname -r`
- Full error message
- Diagnostic output from the checklist above

## Prevention

To avoid these issues in production:

1. **Use infrastructure-as-code**: Install Jailer via scripts or configuration management
2. **Enable cgroups v2 at boot**: Add to GRUB configuration
3. **Set up monitoring**: Alert on Jailer failures
4. **Test regularly**: Run integration tests before deployments
5. **Document deviations**: Keep track of any custom configurations

---

For more information, see:
- [Jailer Integration Guide](./jailer-integration-guide.md)
- [Firecracker Documentation](https://github.com/firecracker-microvm/firecracker/blob/main/docs/)
- [cgroups v2 Documentation](https://docs.kernel.org/admin-guide/cgroup-v2.html)
