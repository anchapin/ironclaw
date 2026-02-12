# Review of PR #78 (Review of PR #72)

## Summary of Changes
This PR introduces `PR_REVIEW.md` which documents critical findings from reviewing PR #72, and implements a fix for firewall chain name collisions using FNV-1a hashing. However, the critical security issues identified in `PR_REVIEW.md` (firewall linking, seccomp integration) remain unaddressed in the code, and new compilation errors have been introduced.

## 🔴 Critical Issues

1.  **Firewall Rules Not Linked (Dead Code)**
    -   **File**: `orchestrator/src/vm/firewall.rs`
    -   **Issue**: The `FirewallManager` creates a custom chain (e.g., `IRONCLAW_...`) and adds DROP rules to it, but it never links this chain to the main `INPUT`, `OUTPUT`, or `FORWARD` chains. The firewall rules are effectively dead code, as traffic is never directed to them.
    -   **Fix**: Add rules to `INPUT`/`FORWARD` chains to jump to the `IRONCLAW_...` chain for traffic matching the VM's interface or IP.

2.  **Seccomp Filters Ignored**
    -   **File**: `orchestrator/src/vm/firecracker.rs`
    -   **Issue**: The `configure_vm` function constructs the VM configuration but never sends the `seccomp_filter` configuration to the Firecracker API. Although `spawn_vm` ensures a filter is present in `VmConfig`, it is never applied.
    -   **Fix**: Update `configure_vm` to serialize `config.seccomp_filter` and send it to the `/seccomp` API endpoint (or include it in the machine configuration if supported by the API version).

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
Please address the Critical Issues (Firewall linking, Seccomp application, Compilation error) and fix the linting/formatting warnings.
