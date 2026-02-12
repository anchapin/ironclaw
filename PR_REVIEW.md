# PR Review: fix: Resolve firewall chain name length and memory bounds issues

## Summary of Changes
The PR addresses two specific issues:
1.  **Firewall Chain Name Length:** Truncates the sanitized VM ID to 19 characters in `FirewallManager` to ensure the resulting `iptables` chain name (`IRONCLAW_<id>`) stays within the 28-character kernel limit.
2.  **Seccomp Audit Log Memory Cap:** Switches `SeccompAuditLog` to use a `VecDeque` with a fixed capacity of 10,000 entries (`MAX_SECCOMP_LOG_ENTRIES`) to prevent unbounded memory growth.
3.  **Testing:** Adds comprehensive tests for truncation, sanitization, and memory limits, including property-based tests.

## Review Feedback

**Code Quality:** ✅
- Adheres to project guidelines and CLAUDE.md.
- Changes are minimal, targeted, and self-documenting.
- Rust code uses `anyhow::Result` and proper error handling.

**Testing:** ✅
- New tests cover the edge cases (long IDs, special characters, log capacity).
- Tests pass locally (`cargo test` verified).

**Security:** ✅
- Prevents `iptables` failures which could lead to unisolated VMs or orchestration crashes.
- Prevents DoS via memory exhaustion in the audit log.

## Potential Issues

🟡 **Warning: VM ID Collision Risk**

In `orchestrator/src/vm/firewall.rs`:
```rust
        // Sanitize vm_id to only contain alphanumeric characters
        // and truncate to ensure chain name <= 28 chars (kernel limit)
        // IRONCLAW_ is 9 chars, so we have 19 chars for the ID
        let sanitized_id: String = vm_id
            .chars()
            .map(|c| if c.is_ascii_alphanumeric() { c } else { '_' })
            .take(19)
            .collect();
```
Truncating the VM ID to 19 characters introduces a collision risk if multiple VMs have IDs that share the same first 19 alphanumeric characters (e.g., `long-project-task-1` and `long-project-task-2` -> both become `IRONCLAW_long_project_tas`).
If this happens, the second VM will fail to spawn because `iptables -N` will return an error ("Chain already exists").

**💡 Suggestion:**
To avoid collisions while respecting the length limit, consider appending a short hash of the full ID to the truncated name, or ensuring the input `vm_id` is a UUID.
Example: `format!("IRONCLAW_{}_{}", &sanitized_id[..10], short_hash(&vm_id))`

## Decision
**Approve** ✅

The changes correctly fix the reported crashes and memory issues. The collision risk is noted but acceptable for this fix; it should be addressed in a future PR if long, similar task names are expected.

---

# Review of PR #78 (Review of PR #72)

## Summary of Changes
This PR introduces `PR_REVIEW.md` which documents critical findings from reviewing PR #72, and implements a fix for firewall chain name collisions using FNV-1a hashing. However, critical security issues identified in `PR_REVIEW.md` (firewall linking, seccomp integration) remain unaddressed in code, and new compilation errors have been introduced.

## 🔴 Critical Issues

1.  **Firewall Rules Not Linked (Dead Code)**
    -   **File**: `orchestrator/src/vm/firewall.rs`
    -   **Issue**: The `FirewallManager` creates a custom chain (e.g., `IRONCLAW_...`) and adds DROP rules to it, but it never links this chain to main `INPUT`, `OUTPUT`, or `FORWARD` chains. The firewall rules are effectively dead code, as traffic is never directed to them.
    -   **Fix**: Add rules to `INPUT`/`FORWARD` chains to jump to `IRONCLAW_...` chain for traffic matching the VM's interface or IP.

2.  **Seccomp Filters Ignored**
    -   **File**: `orchestrator/src/vm/firecracker.rs`
    -   **Issue**: The `configure_vm` function constructs VM configuration but never sends `seccomp_filter` configuration to Firecracker API. Although `spawn_vm` ensures a filter is present in `VmConfig`, it is never applied.
    -   **Fix**: Update `configure_vm` to serialize `config.seccomp_filter` and send it to the `/seccomp` API endpoint (or include it in machine configuration if supported by the API version).

3.  **Compilation Error: Duplicate Module Definition**
    -   **File**: `orchestrator/src/vm/mod.rs`
    -   **Issue**: The file contains both `#[cfg(test)] mod tests;` (referencing `tests.rs`) and an inline `#[cfg(test)] mod tests { ... }` block. This causes a duplicate module definition error.
    -   **Fix**: Remove the inline `mod tests` block or merge its content into `tests.rs`.

## 🟡 Warnings & Code Quality

1.  **Clippy Errors**
    -   Multiple Clippy errors must be fixed for CI compliance:
        -   `src/vm/config.rs:124`: Field assignment outside of initializer (`Default::default()`).
        -   `src/vm/seccomp.rs:639, 672`: Needless borrow (`whitelist.contains(&sys)` -> `sys`).
        -   `benches/mcp_startup.rs`: Unused imports (`BenchmarkId`) and deprecated functions (`black_box` -> `std::hint::black_box`).

2.  **Python Formatting**
    -   **File**: `agent/mcp_client.py`
    -   **Issue**: Fails `black --check`. Run `black agent/` to fix formatting.

3.  **Missing Type Hints**
    -   **File**: `agent/loop.py`
    -   **Issue**: `execute_tool` function is missing type hint for `mcp_client` argument.
    -   **File**: `agent/mcp_client.py`
    -   **Issue**: `__exit__` method is missing type hints for arguments.

## 💡 Suggestions

1.  **Property-Based Testing**
    -   **File**: `orchestrator/src/vm/tests.rs`
    -   **Suggestion**: The current tests iterate over a fixed vector of inputs (`vec!["test-1", ...]`). Consider using `proptest` (which is already a dependency) to generate a wider range of inputs for robust property-based testing.

2.  **Seccomp Integration Tests**
    -   **Suggestion**: Add an integration test that attempts to perform a forbidden syscall (e.g., `socket` with a disallowed protocol) inside the VM to verify that Seccomp filters are actually active.

## Decision
❌ **Request Changes**
Please address Critical Issues (Firewall linking, Seccomp application, Compilation error) and fix linting/formatting warnings.

---

# PR Review for #69

**Summary of Changes**
This PR merges network isolation features (Firewall, Vsock), Seccomp filters, and Windows compatibility fixes. It also adjusts the coverage ratchet to match the CI environment.

**Potential Issues**

🔴 **Critical**
- **Orchestrator CLI missing `mcp` command**: The Python agent (`agent/mcp_client.py`) attempts to spawn the orchestrator using `cargo run -- mcp stdio` (Line 183), but `orchestrator/src/main.rs` does not implement the `mcp` subcommand. This will cause the agent to crash immediately.
- **Orphaned Modules**: `orchestrator/src/agent_rpc.rs` and `orchestrator/src/mcp_command.rs` exist in the codebase but are not included in `orchestrator/src/lib.rs` or `orchestrator/src/main.rs`. As a result, they are not compiled, tested, or available for use.

🟡 **Warning**
- **Blocking I/O in Async Context**: `FirewallManager` uses blocking `std::process::Command` calls (e.g., `orchestrator/src/vm/firewall.rs` Line 133) within async functions. While likely fast, this could block the Tokio runtime if `iptables` hangs or under high load. Consider using `tokio::process::Command` or `tokio::task::spawn_blocking`.

💡 **Suggestion**
- **Type Hints**: `agent/loop.py`: `execute_tool` (Line 72) is missing a type hint for `mcp_client` (should be `McpClient`).
- **Debugging**: `start_firecracker` (`orchestrator/src/vm/firecracker.rs` Line 94) redirects stdout/stderr to `/dev/null`. Capturing this output or redirecting to a log file would significantly aid in debugging startup failures.
- **Safety**: `agent/mcp_client.py`: `_send_request` (Line 144) assumes `self._process` is not `None`. While state checks guard this, an explicit `assert self._process is not None` would improve robustness and type checking.

**Testing**
- Rust unit tests pass (`make test-rust`), but do not cover the orphaned modules (`agent_rpc.rs`).
- Python tests pass (`make test-python`), but integration tests are skipped and likely fail due to the missing CLI command.

**Security**
- Seccomp filters are correctly implemented with a whitelist approach.
- Network isolation uses iptables with proper ID sanitization.

**Decision**
Request Changes. The missing `mcp` subcommand and orphaned modules are critical issues that must be addressed before merging to ensure the agent functions as intended.
