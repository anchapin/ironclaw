# Holon: Dual Repository Microservices Pattern

**ID**: dual-repo-microservices-1739116784000
**Level**: L0 (Hypothesis)
**Kind**: system
**Decision Context**: project-structure-decision-1739116780000
**Created**: 2025-02-09

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
- **Category:** Radical (microservices pattern for single-machine project)
- **Complexity**: High
- **Risk**: High (coordination overhead for solo/small team)
