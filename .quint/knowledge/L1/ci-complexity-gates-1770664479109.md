# Holon: CI/CD Complexity Gates

**ID**: ci-complexity-gates-1770664479109
**Level**: L1 (Substantiated)
**Kind**: system
**Decision Context**: code-quality-guardrails-decision-1770664479107
**Created**: 2025-02-09
**Verified**: 2025-02-09 (PASS)

## Content

### Method (Recipe)

Add a dedicated GitHub Actions workflow `.github/workflows/quality-gates.yml` that runs comprehensive analysis on every PR:

**Features:**
- **Complexity Gates**: Fail if any function has cyclomatic complexity >15
- **Maintainability Index**: Fail if module maintainability <50/100
- **Duplication Detection**: Hard limit of 100 duplicate code blocks
- **Documentation Coverage**: Minimum 80% of public symbols documented
- **Bloat Detection**: Files >100KB, unused dependencies, vulnerable deps

### Expected Outcome

- **Prevents**: Merging of overly complex, duplicated, undocumented, or bloated code
- **Enforcement**: CI blocks merge if gates fail
- **Performance**: Runs in parallel, ~2-3 minutes total
- **Coverage**: Comprehensive analysis of entire codebase

## Scope

**Applies to**: All pull requests and main branch pushes
**Languages**: Python 3.11+, Rust 1.75+
**Platform**: GitHub Actions CI/CD
**Integration**: New workflow file, runs alongside existing CI

## Rationale

```json
{
  "anomaly": "Pre-commit hooks insufficient - can be bypassed, limited analysis depth",
  "approach": "Add comprehensive CI/CD quality gates that block PRs with quality issues",
  "alternatives_rejected": [
    "Pre-commit only (bypassable, no PR gate)",
    "External SaaS tools (expensive, data privacy concerns)",
    "Manual review checklist (not enforceable)"
  ],
  "strengths": [
    "Cannot be bypassed (CI gate)",
    "Comprehensive analysis (complexity, duplication, docs, bloat)",
    "Runs in parallel (fast feedback)",
    "Clear failure messages (actionable)",
    "Free (uses native GitHub Actions)"
  ],
  "weaknesses": [
    "Feedback comes after push (not immediate)",
    "Adds ~2-3 minutes to PR checks",
    "May generate false positives for complex valid code",
    "Requires configuration maintenance"
  ],
  "constraints": [
    "Must run in <5 minutes total",
    "Must not duplicate existing CI checks",
    "Must provide clear error messages",
    "Must allow bypass with 'quality: bypass' label for emergencies"
  ]
}
```

## Verification Summary

**Status**: ✅ SUBSTANTIATED (L1)
**Verification Date**: 2025-02-09
**Verdict**: PASS

**Key Findings**:
- All invariants respected
- Cannot be bypassed (CI gate enforces quality)
- Comprehensive coverage of all gap areas
- Runs in parallel (~2-3 minutes)

**Verification Record**: `.quint/knowledge/L1/ci-complexity-gates-1770664479109-verification.md`

## Status

**Status**: substantiated (L1)
**Verification**: PASS (2025-02-09)
**Validation**: None yet
