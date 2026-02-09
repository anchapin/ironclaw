# Holon: Pre-commit Static Analysis Hooks

**ID**: pre-commit-static-analysis-1770664479108
**Level**: L2 (Validated)
**Kind**: system
**Decision Context**: code-quality-guardrails-decision-1770664479107
**Created**: 2025-02-09
**Validated**: 2025-02-09 (PASS)

## Content

### Method (Recipe)

Add lightweight pre-commit hooks to `.pre-commit-config.yaml` for local, fast feedback:

**Toolchain:**
- `pycln`: Remove unused Python imports/variables
- `radon`: Python cyclomatic complexity (max 10 per function)
- `cargo-complexity`: Rust complexity metrics
- `interrogate`: Python docstring coverage (min 80%)
- `jscpd`: Copy-paste detection across all files

**Configuration files to add:**
- `pyproject.toml`: Complexity thresholds, documentation rules
- `.jscpd.json`: Duplication detection config (exclude tests, defaults)

### Expected Outcome

- **Prevents**: Dead code, overly complex functions, undocumented modules, duplicate code
- **Enforcement**: Local only, blocks commit if checks fail
- **Performance**: <5 seconds for typical commits
- **Coverage**: Both Rust and Python code

## Scope

**Applies to**: All commits to the repository
**Languages**: Python 3.11+, Rust 1.75+
**Platform**: Local development (pre-commit)
**Integration**: Extends existing `.pre-commit-config.yaml`

## Rationale

```json
{
  "anomaly": "Missing local feedback for bloat, duplication, complexity, and documentation gaps",
  "approach": "Add pre-commit hooks that run fast static analysis before commits are allowed",
  "alternatives_rejected": [
    "CI-only gates (too slow, feedback comes after push)",
    "Manual review (inconsistent, doesn't catch all cases)",
    "Heavyweight tools (would slow down commits too much)"
  ],
  "strengths": [
    "Fast local feedback (fail fast, fix fast)",
    "Extends existing pre-commit infrastructure",
    "No additional CI latency",
    "Developer-friendly (runs on their machine)"
  ],
  "weaknesses": [
    "Can be bypassed with --no-verify",
    "Requires developers to keep tools updated",
    "May generate false positives for complex but valid code",
    "Limited to static analysis (can't detect runtime bloat)"
  ],
  "constraints": [
    "Must add <5 seconds to commit time",
    "Must work offline (no network calls)",
    "Must be installable via pip/cargo"
  ]
}
```

## Verification Summary

**Status**: ✅ SUBSTANTIATED (L1)
**Verification Date**: 2025-02-09
**Verdict**: PASS

**Key Findings**:
- All invariants respected
- Extends existing infrastructure (low adoption cost)
- Fast local feedback (<5 seconds)
- Bypassable (acceptable for conservative approach)

**Verification Record**: `.quint/knowledge/L1/pre-commit-static-analysis-1770664479108-verification.md`

## Status

**Status**: substantiated (L1)
**Verification**: PASS (2025-02-09)
**Validation**: None yet

## Validation

**Status**: validated (L2)
**Validation Date**: 2025-02-09
**Test Type**: Internal (Code Execution)
**Result**: PASS
**Evidence**: Performance ~128ms (39x better than <5s claim), all tools working, zero cost
**Validation Record**: `.quint/knowledge/L2/pre-commit-static-analysis-1770664479108-validation.md`

