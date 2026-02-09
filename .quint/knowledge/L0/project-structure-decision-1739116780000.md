# Holon: Project Structure Decision

**ID**: project-structure-decision-1739116780000
**Level**: L0 (Hypothesis)
**Kind**: episteme
**Created**: 2025-02-09

## Content

### Problem Statement
IronClaw requires a dual-language project structure (Rust Orchestrator + Python reasoning loop) that supports:
- Test-driven development from day one
- Strict security invariants (no host execution, ephemeral VMs)
- CI/CD integration for both languages
- Clear separation of concerns
- Compliance with "Agentic Engineering" principles

### Decision Context
This holon groups competing hypotheses for how to structure the IronClaw repository. Each alternative must address:
1. Workspace layout (monorepo vs polyrepo)
2. Test organization (unit, integration, property-based)
3. Development tooling (pre-commit, CI/CD, formatting)
4. Documentation structure
5. Git workflow enforcement

### Success Criteria
Any proposed structure MUST:
- Support TDD workflow from the start
- Enforce security invariants via tests
- Enable <500ms startup time validation
- Keep Python loop <4,000 LOC (enforceable via tooling)
- Support both Rust and Python ecosystems

### Alternatives
See sibling hypotheses:
- unified-monorepo-tdd
- dual-repo-microservices
- cargo-workspace-pyproject
- no-deps-minimal

## Scope
Applies to: IronClaw Phase 1 (Foundation) - Months 1-2
Languages: Rust (Orchestrator), Python (Reasoning Loop)
Team Size: Initially 1-2 developers, scalable to small team

## Rationale
```json
{
  "anomaly": "No established project structure - only documentation exists",
  "approach": "Create decision context to evaluate competing structural approaches",
  "alternatives_rejected": ["Picking structure without evaluation (vibe coding)"]
}
```

## Relations
- **MemberOf**: None (root decision holon)
- **DependsOn**: None

## Metadata
- **Author**: FPF Phase 1 (Abduction)
- **Status**: Proposed
- **Confidence**: Medium (requires evaluation in Phase 2)
