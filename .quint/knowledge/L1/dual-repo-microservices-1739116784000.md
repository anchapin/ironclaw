# Holon: Dual Repository Microservices Pattern

**ID**: dual-repo-microservices-1739116784000
**Level**: L0 (Hypothesis) ❌ NOT PROMOTED
**Kind**: system
**Decision Context**: project-structure-decision-1739116780000
**Created**: 2025-02-09
**Verified**: 2025-02-09

## Content

### Method (Recipe)
Separate repositories for Orchestrator and Agent with strict API boundaries:

```
ironclaw-orchestrator/          # Repo 1: Rust-only
├── Cargo.toml
├── src/
│   ├── main.rs
│   ├── vm/
│   ├── mcp/
│   └── ipc/                   # Inter-process communication
├── tests/
└── README.md

ironclaw-agent/                 # Repo 2: Python-only
├── pyproject.toml
├── loop.py
├── tools/
├── tests/
└── README.md

ironclaw-specs/                 # Repo 3: Shared API contracts
├── protocols/
│   ├── agent-orchestrator-api.md
│   └── mcp-connector-spec.md
└── tests/                      # Contract tests
```

**Communication Pattern**:
- Orchestrator spawns Agent as separate process
- Communication via stdin/stdout JSON messages (or gRPC)
- API versioning via `ironclaw-specs` repo
- Each repo has independent CI/CD pipeline

**Release Strategy**:
- Orchestrator v1.0 works with Agent v1.x
- Agent v2.0 requires Orchestrator v1.1+
- Semantic versioning enforced across repos
- Compatibility matrix documented in specs repo

## Scope
**Applies to**: Multi-team projects, independent release cycles
**Languages**: Repo 1: Rust-only, Repo 2: Python-only
**Communication**: IPC, gRPC, or message queue
**Teams**: Separate teams for each component (future-proofing)

## Verification Report

### Type Check (C.3 Kind-CAL)
**Status**: ✅ PASSED

**Rationale**: Hypothesis kind is `system` (architectural decision). Microservices pattern is technically valid.

### Constraint Check
**Status**: ❌ FAILED (Violates Project Scope)

**Security Invariants (#1-4)**:
- ✅ **#1 Zero Host Execution**: Process isolation actually ENHANCES this invariant ✅
- ✅ **#2-4**: All security invariants are supported

**Architecture Invariants (#5-8)**:
- ✅ **#5 Rust/Python Split**: Supported via separate repos
- ✅ **#6 Native MCP Only**: MCP client in `orchestrator` repo
- ⚠️ **#7 Startup Performance**: IPC adds overhead (may endanger <500ms target)
- ✅ **#8 Memory Efficiency**: Process isolation increases memory usage

**Code Quality Invariants (#9-12)**:
- ✅ **#9 Auditability**: Can enforce 4,000 LOC in `agent` repo
- ✅ **#10 Determinism**: Separate repos enforce boundaries
- ✅ **#11 Standardization**: Uses standard protocols
- ✅ **#12 Zero Manual Containers**: No containers used

**Operational Invariants (#13-15)**:
- ✅ **#13 Local-First**: No cloud dependencies
- ✅ **#14 Private Mesh**: Process communication aligns with mesh philosophy
- ✅ **#15 Rust GUI**: Can add separate repo or include in orchestrator

### Logical Consistency
**Status**: ❌ FAILED (Wrong Scale for Project)

**Analysis**:
1. **Project Scope Mismatch**: PRD states "Phase 1: Foundation (Months 1-2)" with "solo developer" → Microservices is over-engineering ❌
2. **Coordination Overhead**: For 1-2 developers, managing 3 repos is MORE complex than monorepo ❌
3. **Testing Complexity**: Integration testing across repos is harder (must clone multiple repos, set up IPC) ❌
4. **Atomic Commits**: Cannot fix bug that requires both Rust and Python changes in single commit ❌

**Method → Outcome Mapping**:
- Claim: "Independent versioning" → Evidence: True, but IronClaw doesn't NEED independent versioning yet (single product) ⚠️
- Claim: "Team autonomy" → Evidence: True, but there are no multiple teams (current state: solo project) ❌
- Claim: "Clear API boundaries" → Evidence: True, but process boundary is premature optimization ⚠️

### Critical Failure Points
1. **Wrong Scale**: Microservices pattern is designed for 10+ developers, not Phase 1 solo project
2. **IPC Overhead**: May violate <500ms startup target (process spawning + IPC setup)
3. **Integration Testing**: Cross-repo contract tests are complex to set up and maintain
4. **Onboarding Friction**: New contributors must clone 3 repos instead of 1

### When This Approach Might Work
- Phase 3+ when project has 5+ developers
- If Orchestrator becomes a standalone product (separate from Agent)
- If external teams want to build custom Agents for IronClaw Orchestrator

### Comparison to PRD Requirements
**PRD States**:
> "Phase 1: Foundation (Months 1-2) - Fork Nanobot, build Rust Orchestrator"

**This Hypothesis**: Requires 3 repos, IPC layer, versioning strategy from day 1

**Verdict**: PREMATURE OPTIMIZATION

## Rationale
```json
{
  "anomaly": "Coupling Rust and Python in same repo forces coordinated releases",
  "approach": "Separate repos with versioned API contracts enable independent evolution",
  "alternatives_rejected": [
    "Monorepo (forces coupling, adds git blame noise)",
    "Workspace (still requires coordinated builds)",
    "Single process (FFI complexity, memory safety concerns)"
  ],
  "confidence_drivers": [
    "Clear API boundaries (enforced by process isolation)",
    "Independent versioning (Orchestrator v2 while Agent v1)",
    "Separate CI/CD pipelines (Rust changes don't trigger Python tests)",
    "Team autonomy (different teams can work independently)",
    "Zero build-time coupling (no Maturin/setuptools-rust complexity)"
  ]
}
```

## Relations
- **MemberOf**: project-structure-decision-1739116780000
- **DependsOn**: []

## Advantages
✅ **Clear Boundaries**: Process isolation forces API discipline
✅ **Independent Releases**: Version Orchestrator without touching Agent
✅ **Team Scaling**: Separate teams can work in parallel without merge conflicts
✅ **Language Purity**: Each repo uses idiomatic tooling (no hybrid complexity)
✅ **Fault Isolation**: Agent crash doesn't kill Orchestrator (process boundary)

## Disadvantages
❌ **Repo Synchronization**: Must coordinate changes across 3 repos
❌ **Integration Testing**: Harder to test cross-language interactions
❌ **Onboarding Friction**: Developers must clone 2-3 repos
❌ **Atomic Commits**: Can't do single commit touching both Rust and Python
❌ **API Versioning Hell**: Breaking changes require coordinated releases

## Dependencies
None (foundational hypothesis)

## Metadata
- **Author**: FPF Phase 1 (Abduction)
- **Verified By**: FPF Phase 2 (Deduction)
- **Category**: Radical (microservices pattern for single-machine project)
- **Complexity**: High
- **Risk**: High (coordination overhead for solo/small team)
- **Verdict**: ❌ **FAIL** - NOT promoted to L1
- **Failure Reason**: Wrong scale for Phase 1 project (premature optimization)
- **Recommendation**: Reconsider for Phase 3+ if project scales to 5+ developers or multi-team structure
