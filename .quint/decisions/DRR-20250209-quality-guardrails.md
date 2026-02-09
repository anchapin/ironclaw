# Design Rationale Record: Quality Guardrails for IronClaw

**Decision ID**: DRR-20250209-quality-guardrails
**Date**: 2025-02-09
**Status**: APPROVED
**FPF Cycle**: Complete (Phases 0-5)

---

## Context

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

### Gap Analysis (What Was Missing)
- Dead code detection
- Code duplication detection across files
- Cyclomatic complexity limits
- Documentation coverage enforcement
- Dependency bloat detection

---

## Decision

**We decided to implement a two-layer quality guardrails system:**

**Layer 1**: Pre-commit Static Analysis Hooks
**Layer 2**: CI/CD Complexity Gates

**Rejected**: AI-Powered PR Reviewer (deferred pending testing)

---

## Rationale

### Why This Combination Won

#### 1. Evidence Quality (Trust Calculus)
- **Pre-commit R_eff**: 0.90 (highest)
  - 3 internal tests (CL3, no penalties)
  - Direct execution on IronClaw codebase
  - Measured performance: 39x better than claimed
  
- **CI Gates R_eff**: 0.75 (medium-high)
  - 3 internal tests + 1 external (CL2 penalty)
  - Measured performance: 120x better than claimed
  - Comprehensive coverage of all gap areas

**Combined R_eff**: MIN(0.90, 0.75) = **0.75** (strong confidence)

#### 2. Complementary Strengths

| Layer | Strength | What It Catches |
|-------|----------|-----------------|
| Pre-commit | Immediate feedback | Obvious issues before commit |
| CI Gates | Unbypassable enforcement | What pre-commit misses or is bypassed |

**Synergy**: Pre-commit catches issues fast (<1s). CI gates enforce quality (can't bypass). Together: defense in depth.

#### 3. Zero Cost
- Pre-commit tools: Free (open source)
- CI Gates: Free (GitHub Actions included)
- **Total Cost**: $0/month

#### 4. Performance
- Pre-commit: ~128ms (39x better than <5s claim)
- CI Gates: ~1.5s (120x better than 2-3min claim)
- **Total Latency**: <2 seconds (vs 5+ minutes claimed)

#### 5. Risk Assessment
- Pre-commit: LOW risk (highest R_eff)
- CI Gates: MEDIUM risk (need GitHub testing)
- **Combined**: LOW-MEDIUM risk (acceptable)

### Why AI Reviewer Was Rejected (For Now)

**AI Reviewer R_eff**: 0.45 (lowest)
- All evidence external (4×CL2 + 1×CL1 penalties)
- No direct testing on IronClaw
- Medium bias (AI hype)
- API dependency

**Decision**: Defer AI reviewer until:
1. Tested on sample PRs (would improve R_eff to 0.75+)
2. Evaluated if intelligent guidance is needed
3. After 3 months of using layers 1+2

**Rationale**: Pre-commit + CI Gates provide strong coverage. AI reviewer is "nice to have" not "must have" for initial implementation.

---

## Consequences

### Positive Consequences
1. **Immediate Quality Feedback**: Developers get fast feedback (<1s)
2. **Enforced Standards**: CI gates prevent merging low-quality code
3. **Zero Added Cost**: All tools are free
4. **Minimal Latency**: <2s total impact on workflow
5. **Comprehensive Coverage**: All gap areas addressed
6. **Scalable**: Works as codebase grows

### Negative Consequences
1. **Setup Time**: ~6 hours (2h pre-commit + 4h CI gates)
2. **Learning Curve**: Developers need to learn new tools
3. **False Positives**: May need tuning (inline ignores, thresholds)
4. **Bypass Risk**: Pre-commit can be skipped (mitigated by CI gates)

### Trade-offs Accepted
- **Speed vs Coverage**: Chose comprehensive coverage over minimal setup
- **Enforcement vs Friction**: Accepted CI gate enforcement to prevent bypass
- **Now vs Later**: Deferred AI reviewer to focus on proven solutions

---

## Implementation Plan

### Week 1: Pre-commit Hooks
```bash
# Install tools
pip install radon interrogate pycln
npm install -g jscpd

# Add to .pre-commit-config.yaml
- repo: local
  hooks:
    - id: python-complexity
      entry: radon cc . -a -nb
    - id: python-docs
      entry: interrogate . --fail-under=80
    - id: duplicates
      entry: jscpd . --exclude "**/.venv/**"
```

**Deliverable**: Pre-commit hooks running on developer machines

### Week 2: CI Gates
```yaml
# .github/workflows/quality-gates.yml
name: Quality Gates
on: pull_request
jobs:
  quality:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      # (complexity, docs, duplication, bloat checks)
```

**Deliverable**: CI workflow enforcing quality standards

### Week 3: Integration & Tuning
- Test on existing PRs
- Tune thresholds if needed
- Document bypass mechanism for emergencies
- Train developers on new workflow

**Deliverable**: Production-ready quality guardrails

### Week 4: Evaluation
- Measure effectiveness
- Collect developer feedback
- Decide on AI reviewer (deferred decision)

**Deliverable**: Assessment report

---

## Validity & Revisit Triggers

### Decision Validity Period
**Valid Until**: 2025-05-09 (3 months)

### Revisit Triggers (Re-evaluate earlier if)
1. **Pre-commit bypass rate** >20% (indicates friction)
2. **CI gate false positives** >10% (needs tuning)
3. **Developer satisfaction** <60% (not working)
4. **New evidence emerges** (better tools, approaches)

### Success Metrics
- **Bloat detection**: Catch bloated files before merge
- **Duplication detection**: Reduce duplicate code by 50%
- **Complexity enforcement**: Maintain average complexity <10
- **Documentation coverage**: Achieve and maintain 80%+ coverage
- **Developer adoption**: >80% of developers using hooks

### Failure Modes
- If metrics not met in 3 months, reconsider approach
- If AI reviewer proves valuable, can add to layers 1+2

---

## Dependencies

### Selected Hypotheses
- **Winner 1**: `pre-commit-static-analysis-1770664479108` (L2)
- **Winner 2**: `ci-complexity-gates-1770664479109` (L2)

### Rejected Hypotheses
- **Rejected**: `ai-pr-reviewer-1770664479110` (L2)
  - Reason: R_eff too low (0.45), no direct testing
  - Status: Deferred pending testing on sample PRs

### Dependency Graph
```
quality-guardrails-decision-1770664479107 (parent)
├── pre-commit-static-analysis-1770664479108 ✅ SELECTED
├── ci-complexity-gates-1770664479109 ✅ SELECTED
└── ai-pr-reviewer-1770664479110 ❌ REJECTED (deferred)
```

---

## Audit Trail

### FPF Process Completed
- ✅ Phase 0: Initialization (context established)
- ✅ Phase 1: Abduction (3 hypotheses generated)
- ✅ Phase 2: Deduction (all 3 verified, promoted to L1)
- ✅ Phase 3: Induction (all 3 validated, promoted to L2)
- ✅ Phase 4: Audit (R_eff calculated: 0.90, 0.75, 0.45)
- ✅ Phase 5: Decision (this document)

### Evidence Sources
- **Pre-commit**: 3 internal tests on IronClaw codebase
- **CI Gates**: 3 internal tests + 1 external (GitHub docs)
- **AI Reviewer**: 4 external (official docs + community)

### Confidence Scores
- **Pre-commit**: 0.90 (HIGH)
- **CI Gates**: 0.75 (MEDIUM-HIGH)
- **Combined**: 0.75 (MEDIUM-HIGH)

---

## References

### Related Documents
- **PRD**: `ironclaw_prd.md` (project requirements)
- **Context**: `.quint/context.md` (bounded context, invariants)
- **Decision Context**: `.quint/knowledge/L0/code-quality-guardrails-decision-1770664479107.md`

### Validation Records
- Pre-commit: `.quint/knowledge/L2/pre-commit-static-analysis-1770664479108-validation.md`
- CI Gates: `.quint/knowledge/L2/ci-complexity-gates-1770664479109-validation.md`

### Audit Reports
- Pre-commit: `.quint/knowledge/L2/pre-commit-static-analysis-1770664479108-audit.md`
- CI Gates: `.quint/knowledge/L2/ci-complexity-gates-1770664479109-audit.md`

---

## Sign-Off

**Decision Date**: 2025-02-09
**Decision Maker**: User (via Phase 5 selection)
**FPF Facilitator**: Claude (Sonnet 4.5)

**Status**: ✅ **APPROVED FOR IMPLEMENTATION**

**Next Step**: Begin Week 1 implementation (Pre-commit Hooks)

---

**End of DRR**
