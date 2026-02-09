# Holon: Unified Monorepo with TDD Foundation

**ID**: unified-monorepo-tdd-1739116781000
**Level**: L1 (Substantiated) ✅ PROMOTED FROM L0
**Kind**: system
**Decision Context**: project-structure-decision-1739116780000
**Created**: 2025-02-09
**Verified**: 2025-02-09

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

## Verification Report

### Type Check (C.3 Kind-CAL)
**Status**: ✅ PASSED

**Rationale**: Hypothesis kind is `system` (architectural decision), which is appropriate for project structure proposals. Input types (Rust + Python codebase) and output type (unified repository) are compatible with IronClaw's bounded context.

### Constraint Check
**Status**: ✅ PASSED (All 15 invariants satisfied)

**Security Invariants (#1-4)**:
- ✅ **#1 Zero Host Execution**: Structure supports JIT Micro-VM implementation in `orchestrator/vm/`
- ✅ **#2 Approval Required**: Structure includes `orchestrator/approval/` module for Approval Cliff
- ✅ **#3 Ephemeral State**: VM lifecycle management fits naturally in `orchestrator/vm/`
- ✅ **#4 No CVE-2026-25253**: Process isolation via Micro-VMs prevents host RCE

**Architecture Invariants (#5-8)**:
- ✅ **#5 Rust/Python Split**: `orchestrator/` (Rust) + `agent/` (Python) matches specification
- ✅ **#6 Native MCP Only**: `orchestrator/mcp/` explicitly for MCP client (no plugins)
- ✅ **#7 Startup Performance**: CI can benchmark <500ms target via performance tests
- ✅ **#8 Memory Efficiency**: CI can enforce <100MB baseline via profiling

**Code Quality Invariants (#9-12)**:
- ✅ **#9 Auditability**: CI enforces 4,000 LOC limit on `agent/loop.py`
- ✅ **#10 Determinism**: TDD workflow enforces intentional development
- ✅ **#11 Standardization**: Uses standard protocols (MCP), no reinvention
- ✅ **#12 Zero Manual Containers**: Automated CI/CD, no user-managed Dockerfiles

**Operational Invariants (#13-15)**:
- ✅ **#13 Local-First**: No cloud dependencies in structure
- ✅ **#14 Private Mesh**: Structure can accommodate mesh networking in future
- ✅ **#15 Rust GUI**: Structure supports Rust-based GUI (no Electron)

### Logical Consistency
**Status**: ✅ PASSED

**Analysis**:
1. **TDD Support**: Explicit test directories, pre-commit hooks, and unified test runner support TDD workflow ✅
2. **LOC Enforcement**: CI job checking `loop.py` line count directly addresses Invariant #9 ✅
3. **Performance Validation**: Startup time and memory benchmarks enable validation of Invariants #7-8 ✅
4. **Security Testing**: "Security test suite must pass before merge" enforces Invariants #1-4 ✅
5. **Tooling Integration**: Pre-commit hooks (rustfmt, black) prevent "vibe coding" (Invariant #10) ✅

**Method → Outcome Mapping**:
- Claim: "Supports TDD from day one" → Evidence: Explicit test structure + hooks ✅
- Claim: "Enforces invariants" → Evidence: CI gates for LOC, performance, security ✅
- Claim: "Prevents vibe coding" → Evidence: TDD workflow + automated formatting ✅

### Edge Cases Identified
⚠️ **Minor Concern**: CI matrix for both languages may slow feedback loop during development
**Mitigation**: Use GitHub Actions caching and parallel job execution

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
- **Verified By**: FPF Phase 2 (Deduction)
- **Category**: Conservative (proven pattern: similar to Cargo's own workspace structure)
- **Complexity**: Medium
- **Risk**: Low
- **Verdict**: ✅ **PASS** - Promoted to L1 (Substantiated)
