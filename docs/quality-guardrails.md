# IronClaw Quality Guardrails

**Implemented**: 2025-02-09  
**Decision**: DRR-20250209-quality-guardrails  
**Status**: ✅ Active (Layer 1: Pre-commit Hooks)

---

## Overview

IronClaw uses automated quality guardrails to maintain code quality while supporting rapid "vibe coding" based on the PRD. These guardrails catch common issues before they enter the codebase.

---

## Layer 1: Pre-commit Hooks (Active ✅)

### What They Do

Pre-commit hooks run automatically on your machine before you can commit code. They provide **immediate feedback** (<1 second) and catch issues early.

### Tools Installed

| Tool | Purpose | What It Catches |
|------|---------|-----------------|
| **radon** | Complexity analysis | Functions with complexity >10 |
| **interrogate** | Documentation coverage | Modules with <60% docstring coverage |
| **pycln** | Dead code detection | Unused imports and variables |
| **jscpd** | Duplicate code | Code blocks duplicated across files |

### How They Work

```bash
# 1. Make your changes
git add .

# 2. Commit (hooks run automatically)
git commit -m "Add feature"

# Hooks run in sequence:
# ✓ Complexity check (radon)
# ✓ Documentation check (interrogate)
# ✓ Dead code check (pycln)
# ✓ Duplicate check (jscpd)

# If all pass: commit succeeds
# If any fail: commit blocked with details
```

### Installation (One-time Setup)

```bash
# Already done! But if you need to reinstall:
cd agent
source .venv/bin/activate
pip install radon interrogate pycln
npm install -g jscpd
pre-commit install
```

### Bypassing Hooks (Emergency Only)

If you absolutely need to bypass hooks (not recommended):

```bash
git commit --no-verify -m "Emergency bypass"
```

**Note**: CI/CD Layer 2 will still catch issues, so use only for true emergencies.

---

## Configuration

### Complexity Thresholds

**Current Settings**:
- Maximum complexity: **10** (radon grade B)
- Target complexity: **5** (radon grade A)

**What gets flagged**:
- Functions with cyclomatic complexity >10
- Classes with complexity >20
- Modules with complexity >50

**How to fix**:
- Break large functions into smaller ones
- Extract repeated logic into helper functions
- Simplify conditional logic

### Documentation Coverage

**Current Settings**:
- Minimum coverage: **60%** (starting threshold)
- Target coverage: **80%** (will increase over time)

**What gets flagged**:
- Functions without docstrings
- Classes without module documentation
- Missing parameter/return type documentation

**How to fix**:
```python
def process_agent_action(action: Action) -> Result:
    """Process an agent action through the approval cliff.
    
    Args:
        action: The agent action to process (green or red)
    
    Returns:
        Result of the action processing
        
    Example:
        >>> process_agent_action(Action.read_file())
        Result(status='success', data='...')
    """
    # ... implementation
```

### Duplicate Code Detection

**Current Settings**:
- Minimum duplicate lines: **10**
- Maximum duplicate lines: **1000**
- Maximum duplicates: **5** per file

**What gets flagged**:
- Copy-pasted code blocks
- Similar function implementations
- Repeated patterns that should be extracted

**How to fix**:
- Extract common logic into shared functions
- Use inheritance/composition instead of copying
- Create utility modules for repeated operations

### Dead Code Detection

**What gets flagged**:
- Unused imports
- Unused variables
- Unused functions (must be exported somewhere)
- Dead imports from refactoring

**How to fix**:
- Remove unused imports
- Delete or comment out dead code
- Use `# noqa` or `# pragma: no cover` for intentional cases

---

## Performance

**Measured on IronClaw codebase**:

| Check | Time | Status |
|-------|------|--------|
| radon (complexity) | ~60ms | ✅ |
| interrogate (docs) | ~340ms | ✅ |
| pycln (dead code) | ~40ms | ✅ |
| jscpd (duplicates) | ~45s (first run) | ⚠️ |
| **Total** | **~50s** | ✅ |

**Note**: jscpd is slow on first run but fast on subsequent runs due to caching.

---

## Troubleshooting

### Issue: Hook fails but code is correct

**Solution**: False positive - add inline ignore

```python
# radon: ignore
def complex_function_required_by_business_logic():
    # ...
```

### Issue: Documentation check fails

**Solution**: Add docstring or exempt if truly internal

```python
def _internal_helper(x):  # interrogate: ignore
    """Internal helper, no docs needed."""
    # ...
```

### Issue: Duplicate code flagged but it's necessary

**Solution**: Add to .jscpd.json ignore patterns

```json
{
  "ignore": [
    "**/tests/**",  # Test code can be similar
    "**/examples/**"
  ]
}
```

---

## Next Steps

### Layer 2: CI/CD Gates (Coming Week 2)

Pre-commit hooks can be bypassed. CI/CD gates will:
- ✅ Run same checks in GitHub Actions
- ✅ Block PRs that fail quality checks
- ✅ Cannot be bypassed (enforces quality)

### When to Use Layers

```
Local Development (Layer 1)
  ↓ Fast feedback (<1s)
  ↓ Fix issues before pushing
  
Push to GitHub (Layer 2)
  ↓ Runs automatically on PR
  ↓ Blocks merge if quality fails
  ↓ Cannot bypass
```

---

## Feedback

If you encounter issues or have suggestions:
1. Check this documentation first
2. Ask in team chat/Slack
3. Create issue with "quality-guardrails" label

---

## Related Documentation

- **Decision Record**: `.quint/decisions/DRR-20250209-quality-guardrails.md`
- **Project Context**: `CLAUDE.md`
- **PRD**: `ironclaw_prd.md`

---

**Last Updated**: 2025-02-09  
**Maintainer**: IronClaw Team  
**Questions?** See `CLAUDE.md` or create an issue.
