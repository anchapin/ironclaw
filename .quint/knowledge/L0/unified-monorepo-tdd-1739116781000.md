# Holon: Unified Monorepo with TDD Foundation

**ID**: unified-monorepo-tdd-1739116781000
**Level**: L0 (Hypothesis)
**Kind**: system
**Decision Context**: project-structure-decision-1739116780000
**Created**: 2025-02-09

## Content

### Method (Recipe)
Create a unified monorepo with the following structure:

```
ironclaw/
├── orchestrator/          # Rust Orchestrator
│   ├── Cargo.toml
│   ├── src/
│   │   ├── main.rs       # CLI entry point
│   │   ├── vm/           # JIT Micro-VM spawning
│   │   ├── mcp/          # MCP client
│   │   └── approval/     # Approval Cliff UI
│   └── tests/
│       ├── unit/
│       ├── integration/
│       └── property/     # QuickCheck-style tests
├── agent/                # Python Reasoning Loop
│   ├── pyproject.toml
│   ├── loop.py           # Forked from Nanobot
│   ├── tools/
│   └── tests/
│       ├── unit/
│       ├── integration/
│       └── properties/   # Hypothesis-based tests
├── docs/                 # Unified documentation
├── .github/
│   └── workflows/        # CI for both languages
├── scripts/
│   ├── dev.sh            # One-command dev setup
│   ├── test.sh           # Run all tests
│   └── fmt.sh            # Format all code
├── .gitignore            # Unified ignore rules
├── README.md             # Project overview
└── Makefile              # Unified build commands
```

**Tooling Setup**:
- `pre-commit` hooks for both rustfmt and black
- `pytest` + `hypothesis` for Python
- `cargo test` + `proptest` for Rust
- GitHub Actions matrix for Linux/Mac/Windows
- `tox` for Python version testing

**TDD Workflow**:
1. Write failing test first (Red)
2. Implement minimal code to pass (Green)
3. Refactor while keeping tests green (Refactor)
4. Commit only when all tests pass

### Invariant Enforcement
- Python LOC limit: CI job fails if `loop.py` exceeds 4,000 lines
- Security test suite: Must pass before any merge
- Startup time benchmark: CI runs performance regression tests
- Memory profiling: CI fails if baseline exceeds 100MB

## Scope
**Applies to**: Small teams (1-5 developers), single repository
**Languages**: Rust 1.75+, Python 3.11+
**Platforms**: Linux (primary), macOS (secondary), Windows (eventual)
**CI/CD**: GitHub Actions (recommended), GitLab CI (supported)

## Rationale
```json
{
  "anomaly": "Need unified structure for dual-language project with strict quality gates",
  "approach": "Monorepo with shared tooling and unified CI/CD pipeline",
  "alternatives_rejected": [
    "Polyrepo (overhead for small team, sync complexity)",
    "Submodule management (git submodule pain points)",
    "Separate repos for each language (fragile dependencies)"
  ],
  "confidence_drivers": [
    "Single source of truth for project state",
    "Atomic commits across Rust/Python boundary",
    "Unified testing workflow (one command to test everything)",
    "Simplified CI/CD (single pipeline matrix)"
  ]
}
```

## Relations
- **MemberOf**: project-structure-decision-1739116780000
- **DependsOn**: []

## Advantages
✅ **TDD Native**: Single test command runs both Rust and Python tests
✅ **Atomic Commits**: Changes across language boundary in one commit
✅ **Simplified Onboarding**: One clone, full development environment
✅ **Shared Tooling**: pre-commit, CI/CD configured once
✅ **Enforced Invariants**: CI can check cross-language properties (e.g., LOC limits)

## Disadvantages
❌ **Git Blame Noise**: Mixed languages can complicate history analysis
❌ **CI Complexity**: Matrix builds for both languages (mitigation: caching)
❌ **Release Coordination**: Must version both components together

## Dependencies
None (foundational hypothesis)

## Metadata
- **Author**: FPF Phase 1 (Abduction)
- **Category**: Conservative (proven pattern: similar to Cargo's own workspace structure)
- **Complexity**: Medium
- **Risk**: Low
