# Platform Support

## Current Status

**Supported Platforms:**
- ✅ **Linux** (Ubuntu 22.04+, Debian, Fedora, etc.) - **Primary development and production platform**
- ✅ **macOS** (Intel and Apple Silicon) - **Supported for development**

**Unsupported Platforms:**
- ❌ **Windows** - **Not supported at this time**

## Why Windows Is Not Supported

IronClaw relies heavily on Linux-specific technologies that have no direct equivalents on Windows:

### 1. Firecracker Micro-VMs
- Firecracker is built on Linux KVM (Kernel-based Virtual Machine)
- Requires `/dev/kvm` device access
- Windows has no KVM subsystem
- Alternative VMMs for Windows (Hyper-V, QEMU with HAXM) are not compatible with Firecracker's architecture

### 2. vsock (Virtual Socket)
- Used for secure communication between host and Micro-VM
- Linux kernel feature (`AF_VSOCK`)
- Windows has no vsock implementation
- No straightforward replacement for this secure, isolated IPC mechanism

### 3. iptables/nftables
- Used for VM network isolation (firewall)
- Linux netfilter framework
- Windows has Windows Defender Firewall, but different API and capabilities
- Seccomp filters also Linux-specific

### 4. seccomp (Secure Computing)
- Linux syscall filter for sandboxing
- No equivalent on Windows
- Core to IronClaw's security model (restrict to ~100 essential syscalls)

### 5. Unix Domain Sockets
- Used extensively for local IPC
- Windows has named pipes, but different semantics
- Not drop-in compatible

## Technical Debt Assessment

Adding Windows support would require:

1. **Replacing Firecracker** with a cross-platform VMM (possible alternatives):
   - QEMU (works on Windows but heavier, slower startup)
   - Hyper-V (Windows-only, defeats cross-platform goal)
   - Custom VMM (months of development)

2. **Replacing vsock** with platform-specific IPC:
   - Windows: Named pipes + security descriptors
   - Linux/Mac: Unix domain sockets
   - Significant abstraction layer needed

3. **Replacing iptables/seccomp** with:
   - Windows: Windows Defender Firewall + job objects + token filtering
   - macOS: PF firewall + seatbelt sandbox
   - Complex conditional compilation throughout codebase

4. **Testing Matrix**:
   - 3x platform maintenance burden
   - Platform-specific CI runners
   - Different installation procedures, dependencies

**Estimated Effort**: 2-3 months full-time for experienced Rust developer
**Risk Level**: High (introduces complexity, undermines simplicity)
**Priority**: Low (target audience: Linux developers, security-focused teams)

## Alternative Approaches

### Run IronClaw in WSL2
- Windows users can run IronClaw in **Windows Subsystem for Linux 2**
- Provides full Linux kernel compatibility
- Firecracker works natively in WSL2
- Recommended development workflow for Windows users

### Use a Linux VM
- Traditional approach: run Linux in VirtualBox/VMware
- IronClaw can run inside that VM
- Double-virtualization performance overhead but acceptable for development

### Remote Development
- Use IronClaw on a remote Linux server/machine
- Connect via SSH for development
- VS Code Remote SSH extension provides seamless experience

## Future Considerations

Windows support may be reconsidered if:

1. Firecracker (or a compatible drop-in replacement) adds Windows support
2. Community demand exceeds threshold (tracked via GitHub issues)
3. Cross-platform VMM solution emerges with similar performance characteristics

For now, the project remains **Linux-first** to maintain development velocity and codebase simplicity.

## See Also

- [CLAUDE.md](CLAUDE.md) - Architecture principles and development workflow
- [docs/architecture/architecture.md](docs/architecture/architecture.md) - System design
- [docs/testing/testing.md](docs/testing/testing.md) - Testing strategy (includes platform notes)
