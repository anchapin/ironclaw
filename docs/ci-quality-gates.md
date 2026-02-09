# CI/CD Quality Gates - Layer 2

**Implemented**: 2025-02-09  
**Status**: ✅ Active (Unbypassable Enforcement)  
**Workflow**: `.github/workflows/quality-gates.yml`

---

## Overview

CI/CD Quality Gates provide **unbypassable enforcement** of code quality standards. They run automatically on every pull request and push to the main branch.

### Key Difference from Pre-commit Hooks

| Feature | Pre-commit (Layer 1) | CI/CD Gates (Layer 2) |
|---------|---------------------|----------------------|
| **Speed** | <1 second | ~1.5 seconds |
| **Location** | Local machine | GitHub Actions (cloud) |
| **Enforcement** | Bypassable with `--no-verify` | **Cannot be bypassed** (blocks merge) |
| **Trigger** | Before commit | After push/PR |
| **Feedback** | Immediate | After push (1-2 min delay) |

---

## What Gets Checked

### 1. Complexity Analysis (radon)
```yaml
Threshold: Max complexity grade C (15)
Blocks: PRs with functions/classes exceeding threshold
```

**What it catches**:
- Functions with cyclomatic complexity >15
- Classes with excessive complexity
- Modules that are hard to maintain

**Example failure**:
```
❌ Function 'process_action' has complexity 18 (max 15)
```

### 2. Documentation Coverage (interrogate)
```yaml
Threshold: 60% docstring coverage
Blocks: PRs with insufficient documentation
```

**What it catches**:
- Functions without docstrings
- Classes without module documentation
- Missing parameter/return docs

**Example failure**:
```
❌ Documentation coverage: 45% (minimum: 60%)
Missing docs in:
  - agent/processor.py: process_items
  - agent/handlers.py: Handler.dispatch
```

### 3. Dead Code Detection (pycln)
```yaml
Action: Reports unused code (warns, doesn't block)
```

**What it catches**:
- Unused imports
- Unused variables
- Dead code from refactoring

**Note**: Currently advisory only, can be made blocking in future.

### 4. Duplicate Code Detection (jscpd)
```yaml
Threshold: Max 5 duplicate code blocks
Blocks: PRs with excessive duplication
```

**What it catches**:
- Copy-pasted code blocks
- Similar implementations
- Repeated patterns

**Example failure**:
```
❌ Found 8 duplicate code blocks (max: 5)
agent/handler1.py:45-60 ↔ agent/handler2.py:78-93
```

### 5. Bloat Detection
```yaml
Threshold: 
  - No Python files >100KB
  - loop.py ≤ 4000 lines
Blocks: PRs with bloated files
```

**What it catches**:
- Files exceeding size limits
- loop.py growing beyond maintainable size
- General code bloat

**Example failure**:
```
❌ agent/legacy.py is 157KB (max: 100KB)
```

---

## How It Works

### Normal Flow

```bash
# 1. Developer commits and pushes
git add .
git commit -m "Add feature"
git push origin feature-branch

# 2. GitHub Actions automatically triggers
# (runs on every PR to main)

# 3. 5 quality jobs run in parallel:
├── Complexity Analysis (radon)     ← 30 seconds
├── Documentation Coverage          ← 20 seconds
├── Dead Code Detection             ← 15 seconds
├── Duplicate Detection             ← 25 seconds
└── Bloat Detection                 ← 10 seconds

# 4. If all pass:
✅ PR can be merged

# 5. If any fail:
❌ PR blocked until fixed
```

### Job Status on PR

On your pull request page, you'll see:

```
Quality Gates
├── ✅ Complexity Analysis - Passed
├── ✅ Documentation Coverage - Passed (100%)
├── ✅ Dead Code Detection - Passed
├── ✅ Duplicate Detection - Passed (0 clones)
└── ✅ Bloat Detection - Passed

Status: All checks passed ✅
```

---

## Bypass Mechanism

### When to Bypass

**Use sparingly** for emergencies only:
- Security hotfixes that can't wait
- Critical production incidents
- Documentation errors (typos, formatting)

### How to Bypass

**Step 1**: Add the bypass label to your PR
```
GitHub PR → Labels → Add "quality: bypass"
```

**Step 2**: Re-run or wait for checks
```
Quality gates will be skipped
⚠️  PR can be merged without enforcement
```

**Step 3**: Document the reason
```
In PR description: "Bypassed quality gates because..."
```

### Bypass Audit

All bypasses are:
- Logged in GitHub Actions
- Visible in PR comments
- Tracked for review

**Abuse of bypass mechanism** will be addressed by team leads.

---

## Configuration

### Thresholds

| Check | Current | Target | When to Increase |
|-------|---------|--------|------------------|
| Complexity | 15 (grade C) | 10 (grade B) | After 3 months |
| Documentation | 60% | 80% | After 1 month |
| Duplicates | 5 blocks | 3 blocks | After 3 months |
| File Size | 100KB | 50KB | When needed |
| loop.py LOC | 4000 | 4000 (hard) | Never (invariant) |

### Parallel Execution

All 5 quality jobs run **in parallel** on GitHub Actions:
- Total runtime: ~30-60 seconds (not 5×)
- Faster feedback than sequential execution
- Independent checks don't block each other

---

## Performance

**Actual vs Claimed** (from validation):

| Metric | Claimed | Actual | Improvement |
|--------|---------|--------|-------------|
| Total Runtime | 2-3 minutes | ~1.5 minutes | **2x better** |
| Complexity | 30s | ~20s | 1.5x better |
| Documentation | 45s | ~20s | 2.25x better |
| Dead Code | 20s | ~15s | 1.33x better |
| Duplicates | 30s | ~25s | 1.2x better |
| Bloat | 15s | ~10s | 1.5x better |

**Note**: First run may be slower (caching), subsequent runs are faster.

---

## Troubleshooting

### Issue: CI fails but pre-commit passed

**Cause**: CI may have stricter thresholds or check entire codebase

**Solution**:
1. Check CI logs for specific failure
2. Fix the issue locally
3. Commit and push again
4. CI will re-run automatically

### Issue: False positive

**Solution**: Add inline ignore or discuss with team

```python
# radon: ignore
def complex_but_necessary():
    """This function is complex but required for business logic."""
    # ...
```

### Issue: Workflow not triggering

**Check**:
1. Is the PR targeting `main` branch?
2. Is the workflow file valid YAML?
3. Are GitHub Actions enabled for the repo?

### Issue: Need to bypass but label not working

**Alternative**: Maintain documentation only - create issue to track
```
Issue: "Quality gates bypassed - PR #123"
Reason: "Security hotfix, cannot wait for refactoring"
```

---

## Integration with Pre-commit Hooks

### Layer 1 + Layer 2 = Defense in Depth

```
Developer Workflow:
┌─────────────────────────────────────────────────────────┐
│  1. Write code                                          │
│  2. Commit (pre-commit runs, <1s)                      │
│     → Catch issues early, fix immediately               │
│  3. Push to GitHub                                      │
│  4. CI runs (quality gates, ~1.5min)                   │
│     → Enforce quality, blocks merge if fails            │
│  5. Merge to main (only if all pass)                   │
└─────────────────────────────────────────────────────────┘
```

**Benefits**:
- **Pre-commit**: Fast feedback, catch obvious issues
- **CI Gates**: Enforced quality, catch what pre-commit misses or bypasses
- **Together**: Comprehensive coverage, unbypassable enforcement

---

## Cost

**GitHub Actions Usage**:
- Free tier: 2000 minutes/month
- This workflow: ~1.5 minutes per PR
- **Break-even**: 1333 PRs/month (you won't hit this)
- **Cost**: $0 (within free tier)

---

## Related Documentation

- **Pre-commit Hooks**: `docs/quality-guardrails.md` (Layer 1)
- **Decision Record**: `.quint/decisions/DRR-20250209-quality-guardrails.md`
- **Implementation Summary**: `.quint/decisions/week2-implementation-summary.md`
- **Validation**: `.quint/knowledge/L2/ci-complexity-gates-1770664479109-validation.md`

---

## Feedback & Issues

**Problem with quality gates?**
1. Check this documentation
2. Review CI logs in GitHub Actions
3. Ask in team chat
4. Create issue with "ci-quality" label

**Want to change thresholds?**
- Discuss with team first
- Update `.github/workflows/quality-gates.yml`
- Update this documentation
- Communicate change to team

---

**Last Updated**: 2025-02-09  
**Maintainer**: IronClaw Team  
**Status**: ✅ Active
