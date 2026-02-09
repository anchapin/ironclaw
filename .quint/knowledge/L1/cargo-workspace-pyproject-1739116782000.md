# Holon: Cargo Workspace + pyproject.toml Hybrid

**ID**: cargo-workspace-pyproject-1739116782000
**Level**: L1 (Substantiated) ✅ PROMOTED FROM L0
**Kind**: system
**Decision Context**: project-structure-decision-1739116780000
**Created**: 2025-02-09
**Verified**: 2025-02-09

## Content

### Method (Recipe)
Create a Cargo workspace that also respects Python project layout:

```
ironclaw/
├── Cargo.toml                # Workspace root
├── Cargo.lock                # Unified Rust dependency lock
├── pyproject.toml            # Unified Python config (PEP 518)
├── orchestrator/             # Workspace member
│   ├── Cargo.toml            # Member config
│   └── src/
├── agent/                    # Workspace member (Python)
│   ├── pyproject.toml        # Inherits from root
│   ├── loop.py
│   └── tests/
├── vm/                       # Shared VM library (Rust)
│   ├── Cargo.toml            # Can be used by orchestrator
│   └── src/
├── mcp/                      # MCP client (Rust)
│   ├── Cargo.toml
│   └── src/
├── tests/                    # Integration tests (both languages)
│   ├── rust_integration.rs
│   └── python_integration.py
├── scripts/
│   ├── bootstrap.py          # Python dev setup
│   └── bootstrap.sh          # Rust dev setup
└── .pre-commit-config.yaml   # Unified hooks
```

**Hybrid Tooling**:
- `cargo test` runs Rust tests
- `pytest` runs Python tests
- `cargo xtask` for unified commands (build, test, fmt, check)
- `maturin` or `setuptools-rust` for Rust-Python binding (if needed)

**Workspace Benefits**:
- Shared dependencies between `orchestrator`, `vm`, and `mcp` crates
- Unified `Cargo.lock` for reproducible Rust builds
- Atomic cargo updates across workspace members
- CI can run `cargo test --workspace` once

## Scope
**Applies to**: Projects with multiple Rust components + Python
**Languages**: Rust (workspace), Python (pyproject.toml hierarchy)
**Build System**: Cargo (Rust), setuptools/poetry (Python)
**Package Manager**: Cargo + pip/poetry

## Verification Report

### Type Check (C.3 Kind-CAL)
**Status**: ✅ PASSED

**Rationale**: Hypothesis kind is `system` (architectural decision). The Cargo workspace pattern is a well-established Rust archetype, and coexisting with Python via pyproject.toml is technically sound.

### Constraint Check
**Status**: ✅ PASSED (All 15 invariants satisfied)

**Security Invariants (#1-4)**:
- ✅ **#1 Zero Host Execution**: `vm/` crate can implement JIT Micro-VM spawning
- ✅ **#2 Approval Required**: Can add `approval/` crate to workspace
- ✅ **#3 Ephemeral State**: VM lifecycle managed in `vm/` crate
- ✅ **#4 No CVE-2026-25253**: Process isolation via Micro-VMs

**Architecture Invariants (#5-8)**:
- ✅ **#5 Rust/Python Split**: `orchestrator/` (Rust workspace) + `agent/` (Python)
- ✅ **#6 Native MCP Only**: Dedicated `mcp/` crate for MCP client
- ✅ **#7 Startup Performance**: Can benchmark via `cargo xtask bench`
- ✅ **#8 Memory Efficiency**: Can profile via `cargo xtask profile`

**Code Quality Invariants (#9-12)**:
- ⚠️ **#9 Auditability**: CI must still enforce 4,000 LOC limit on `agent/loop.py`
- ✅ **#10 Determinism**: Workspace enforces intentional Rust architecture
- ✅ **#11 Standardization**: Uses Cargo (standard) + MCP protocol
- ✅ **#12 Zero Manual Containers**: Automated builds, no user Dockerfiles

**Operational Invariants (#13-15)**:
- ✅ **#13 Local-First**: No cloud dependencies
- ✅ **#14 Private Mesh**: Workspace can accommodate mesh networking
- ✅ **#15 Rust GUI**: Can add `gui/` crate later (Rust-based)

### Logical Consistency
**Status**: ✅ PASSED

**Analysis**:
1. **TDD Support**: `cargo test --workspace` + `pytest` enable TDD workflow ✅
2. **Component Isolation**: Separate `vm/`, `mcp/`, `orchestrator/` crates are independently testable ✅
3. **Dependency Management**: Unified `Cargo.lock` prevents dependency conflicts ✅
4. **Python Integration**: `agent/` can use standard Python tooling without interference ✅

**Method → Outcome Mapping**:
- Claim: "Manage multiple Rust crates + Python" → Evidence: Workspace with 3 Rust crates + 1 Python component ✅
- Claim: "Shared dependencies" → Evidence: Unified `Cargo.lock` + workspace inheritance ✅
- Claim: "Independent testing" → Evidence: `cargo test -p vm` tests VM crate in isolation ✅

### Edge Cases Identified
⚠️ **Concern 1**: CI must manually check Python LOC limits (no native Cargo support)
**Mitigation**: Add Python LOC check to `.github/workflows/test.yml`

⚠️ **Concern 2**: Two lock files (`Cargo.lock`, `poetry.lock`) may confuse users
**Mitigation**: Document clearly in README; use `cargo xtask` to abstract away

⚠️ **Concern 3**: Bootstrap requires both Rust toolchain AND Python venv
**Mitigation**: Provide `scripts/bootstrap.sh` that handles both

## Rationale
```json
{
  "anomaly": "Need to manage multiple Rust crates + Python without tooling conflicts",
  "approach": "Cargo workspace for Rust, pyproject.toml for Python, coexisting peacefully",
  "alternatives_rejected": [
    "Pure monorepo (loses Cargo workspace benefits)",
    "Separate repos (loses atomic rust dependency updates)",
    "Submodules (git complexity for small team)"
  ],
  "confidence_drivers": [
    "Cargo workspace is battle-tested (Rust ecosystem standard)",
    "Clear separation of Rust components (orchestrator vs vm vs mcp)",
    "Python can be workspace member without Cargo knowledge",
    "xtasks pattern provides custom commands (test, fmt, check)"
  ]
}
```

## Relations
- **MemberOf**: project-structure-decision-1739116780000
- **DependsOn**: []

## Advantages
✅ **Cargo Workspace**: Unified Rust dependency management
✅ **Component Isolation**: `vm` and `mcp` are separate crates (testable independently)
✅ **Python Freedom**: Python tooling (pytest, black, mypy) works normally
✅ **Shared Integration Tests**: Cross-language tests in `tests/` directory
✅ **Industry Standard**: Similar to rust-analyzer, tokio workspace patterns

## Disadvantages
❌ **Tooling Complexity**: Two ecosystems, two lock files (`Cargo.lock`, `poetry.lock`)
❌ **Bootstrap Complexity**: Need both Rust toolchain AND Python venv
❌ **Learning Curve**: Team must understand both Cargo workspaces and pyproject.toml
❌ **Build Order**: Rust builds must complete before Python can use FFI bindings (if any)

## Dependencies
None (foundational hypothesis)

## Metadata
- **Author**: FPF Phase 1 (Abduction)
- **Verified By**: FPF Phase 2 (Deduction)
- **Category**: Pragmatic (balances Rust best practices with Python needs)
- **Complexity**: Medium-High
- **Risk**: Medium
- **Verdict**: ✅ **PASS** - Promoted to L1 (Substantiated)
