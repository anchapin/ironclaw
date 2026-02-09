# Holon: Code Quality Guardrails Decision

**ID**: code-quality-guardrails-decision-1770664479107
**Level**: L0 (Hypothesis)
**Kind**: episteme
**Created**: 2025-02-09

## Content

### Problem Statement

User will be "vibe coding" based on PRD and needs automated guardrails to prevent:

1. **Bloat**: Unnecessary files, large files, dead code, unused dependencies
2. **Duplication**: Code or functionality duplicated in different locations
3. **Verbosity**: Inefficient, overly verbose code patterns
4. **Documentation gaps**: Insufficient comments explaining what code does

### Existing Coverage

- ✅ Formatting (black, rustfmt)
- ✅ Linting (clippy, flake8, mypy, pylint)
- ✅ Coverage enforcement (75% ratchet)
- ✅ Basic file checks (large files, merge conflicts, secrets)

### Gap Analysis

Missing:
- Dead code detection (unused functions, imports)
- Code duplication detection across files
- Cyclomatic complexity limits (prevent overly complex functions)
- Documentation coverage enforcement
- Dependency bloat detection

### Success Criteria

Any solution MUST:
- Catch duplicate code before merge
- Flag dead or unused code
- Prevent overly complex functions (complexity threshold)
- Enforce documentation coverage (docstrings/comments)
- Work within existing CI/CD (GitHub Actions)
- Not significantly slow down development workflow

## Scope

**Applies to**: IronClaw monorepo (Rust `orchestrator/` + Python `agent/`)
**Affects**: All commits and PRs
**Languages**: Rust 1.75+, Python 3.11+
**CI/CD**: GitHub Actions (existing infrastructure)

## Rationale

```json
{
  "anomaly": "User will vibe code based on PRD - risk of bloat, duplication, verbose code, missing docs",
  "approach": "Create decision context to evaluate competing quality guardrail strategies",
  "alternatives_rejected": [
    "Manual code review (doesn't scale, inconsistent)",
    "No guardrails (guaranteed bloat and decay)",
    "Post-hoc cleanup (too late, expensive)"
  ],
  "constraints": [
    "Must integrate with existing GitHub Actions",
    "Must support both Rust and Python",
    "Must not add significant latency to commits",
    "Must provide actionable feedback to developers"
  ],
  "source": "User input - manually injected via q1-add",
  "note": "Decision context for evaluating competing approaches to code quality automation"
}
```

## Status

**Status**: pending
**Verification**: None yet
**Validation**: None yet
