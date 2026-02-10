# Agent Task: rootfs-hardening

## Context
- **Issue:** #21
- **Feature:** rootfs-hardening
- **Working Directory:** ../ironclaw-rootfs-hardening
- **Branch:** feature/21-rootfs-hardening

## Your Mission
Implement read-only root filesystem with integrity checks for IronClaw VMs.

## Key Tasks
1. Create read-only rootfs implementation
2. Implement separate /tmp overlay (writable)
3. Add dm-verity integrity check
4. Create minimal guest OS builder
5. Implement rootfs signing
6. Write security tests
7. Update documentation

## Dependencies
Depends on: Jailer integration (Issue #18)

## Working Instructions
Same as other agents - see AGENT_TASK.md in other worktrees for full instructions.

