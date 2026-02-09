# Holon: Cargo Workspace + pyproject.toml Hybrid

**ID**: cargo-workspace-pyproject-1739116782000
**Level**: L0 (Hypothesis)
**Kind**: system
**Decision Context**: project-structure-decision-1739116780000
**Created**: 2025-02-09

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
- **Category**: Pragmatic (balances Rust best practices with Python needs)
- **Complexity**: Medium-High
- **Risk**: Medium
