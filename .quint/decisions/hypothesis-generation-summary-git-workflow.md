# Phase 1: Hypothesis Generation - Git Workflow Automation

**Date**: 2026-02-09
**Phase**: Abduction (Hypothesis Generation)
**Decision Context**: Git Workflow Automation for AI-Assisted Development

---

## Problem Statement (Anomaly)

**Current State**: The IronClaw repository allows direct pushes to the `main` branch, creating risks for AI-assisted development:
1. AI coding agents (Claude Code, Copilot, etc.) can push directly to main without review
2. No automated link between GitHub issues and pull requests
3. Missing traceability between work items and code changes
4. No enforcement of feature branch workflow

**Desired State**: All code changes MUST follow:
1. Create GitHub issue first (for tracking)
2. Create feature branch from issue
3. Make changes on feature branch
4. Create PR linked to issue
5. Merge only through PR with approval

**Gap**: No automated enforcement mechanism exists

---

## Generated Hypotheses

### Parent Decision Context

**Holon**: `git-workflow-parent-decision-1770664479200`
**Kind**: Episteme (decision framing)
**Purpose**: Defines the problem space and success criteria for all child hypotheses

**Success Criteria** (ALL hypotheses must address):
- Block direct pushes to `main` branch (non-negotiable)
- Require GitHub issue before creating feature branch
- Link PR to issue using GitHub's automatic linking
- Use gh CLI for all automation
- Update CLAUDE.md with workflow instructions
- Pass pre-commit hooks before PR creation

---

### Hypothesis 1: GitHub Native + gh CLI (Conservative)

**Holon**: `github-native-gh-cli-1770664479201`
**Kind**: System
**Approach**: Use GitHub's native branch protection API + client-side pre-commit hook + git aliases

**Key Features**:
- GitHub branch protection rules (server-side enforcement)
- Pre-commit hook to block local main commits
- Git aliases for low-friction workflow
- Simple shell script wrapper
- Minimal complexity, high reliability

**Effort**: Medium (2-3 hours)
**Risk**: Low (reversible, well-documented)
**Best For**: Teams wanting simple, reliable enforcement

**Implementation Highlights**:
```bash
# Branch protection via gh CLI
gh api repos/:owner/:repo/branches/main/protection --method PUT \
  -f required_pull_request_reviews='{"required_approving_review_count":1}'

# Pre-commit hook blocks main commits
if [[ "$CURRENT_BRANCH" =~ ^(main|master)$ ]]; then
  echo "❌ Cannot commit directly to $CURRENT_BRANCH"
  exit 1
fi
```

---

### Hypothesis 2: Pre-commit Hook + CI Validation (Moderate)

**Holon**: `pre-commit-hook-ci-validation-1770664479202`
**Kind**: System
**Approach**: Enhanced pre-commit hook + GitHub Actions CI workflow + branch protection

**Key Features**:
- Multi-layer pre-commit validation (branch blocking, format checking, hook execution)
- GitHub Actions validates PR-issue linkage server-side
- GitHub Actions validates branch name format
- Required status check enforces CI passes before merge
- Enhanced git-workflow.sh wrapper with error handling

**Effort**: Medium-High (4-5 hours)
**Risk**: Medium (more moving parts)
**Best For**: Teams wanting server-side validation + client-side feedback

**Implementation Highlights**:
```yaml
# GitHub Actions validates PR body
- name: Check PR body for issue reference
  if: !contains(github.event.pull_request.body, 'Closes #')
  run: |
    echo "❌ PR must reference an issue"
    exit 1
```

---

### Hypothesis 3: Multi-Layer Defense (Comprehensive)

**Holon**: `multi-layer-defense-1770664479203`
**Kind**: System
**Approach**: Combine ALL validation mechanisms for maximum safety

**Key Features**:
- **Layer 1**: Client-side pre-commit hook (fast feedback)
- **Layer 2**: Git aliases (low-friction workflow)
- **Layer 3**: GitHub Actions CI validation (server-side enforcement)
- **Layer 4**: GitHub branch protection (hard block)
- **Layer 5**: Comprehensive workflow automation script
- **Layer 6**: Complete documentation in CLAUDE.md

**Effort**: High (1-2 days)
**Risk**: Low (very robust due to redundancy)
**Best For**: Production repositories requiring maximum safety

**Defense in Depth**: If any layer fails, others still protect the main branch

---

## Hypothesis Comparison

| Aspect | Hypothesis 1 (Conservative) | Hypothesis 2 (Moderate) | Hypothesis 3 (Comprehensive) |
|--------|----------------------------|-------------------------|-------------------------------|
| **Layers** | 2 (branch protection + pre-commit) | 3 (pre-commit + CI + protection) | 6 (all mechanisms) |
| **Effort** | 2-3 hours | 4-5 hours | 1-2 days |
| **Complexity** | Low | Medium | High |
| **Reliability** | High | Higher | Very High |
| **Server-Side Validation** | Yes (branch protection) | Yes (CI + protection) | Yes (CI + protection) |
| **Client-Side Feedback** | Yes (pre-commit) | Yes (enhanced pre-commit) | Yes (enhanced pre-commit) |
| **Git Aliases** | Yes | No | Yes |
| **Workflow Script** | Simple | Enhanced | Comprehensive |
| **Best For** | Small teams, quick setup | Medium teams, balance | Production, maximum safety |

---

## Decision Context

### Dependencies

All three hypotheses have **NO direct dependencies** on other holons. They are standalone implementations.

### Congruence Level

All hypotheses are **CL3** (same context) - they exist entirely within the repository automation domain.

### Alternative Relationship

These hypotheses are **MUTUALLY EXCLUSIVE** alternatives:
- You would implement ONE of these approaches
- They all solve the same problem with different trade-offs
- Parent decision `git-workflow-parent-decision-1770664479200` frames the choice

---

## Next Steps

### Phase 2: Verification (/q2-verify)

For each hypothesis, verify:
1. **Logical Consistency**: Does the approach actually enforce the workflow?
2. **Technical Feasibility**: Can it be implemented with available tools?
3. **Invariant Compliance**: Does it respect IronClaw's constraints?
4. **Completeness**: Does it address all success criteria?

### Phase 3: Validation (/q3-validate)

For the promoted hypothesis, validate:
1. **Prototype**: Implement the approach
2. **Testing**: Verify it blocks bad workflows
3. **AI Agent Testing**: Confirm AI agents follow it
4. **Documentation**: Update CLAUDE.md

---

## Recommendation (Pre-Verification)

Based on the problem requirements, I recommend starting with **Hypothesis 2 (Pre-commit Hook + CI Validation)** because:

1. **Balanced Complexity**: Not too simple (H1), not over-engineered (H3)
2. **Server-Side Enforcement**: GitHub Actions provides reliable validation that can't be bypassed
3. **Client-Side Feedback**: Enhanced pre-commit gives immediate feedback
4. **AI Agent Compatible**: Both layers guide AI agents to correct workflow
5. **Maintainable**: Moderate complexity, clear separation of concerns

**However**, this recommendation should be validated through Phase 2 verification.

---

## Audit Trail

**Holon IDs Created**:
- `git-workflow-parent-decision-1770664479200` (parent decision)
- `github-native-gh-cli-1770664479201` (Hypothesis 1)
- `pre-commit-hook-ci-validation-1770664479202` (Hypothesis 2)
- `multi-layer-defense-1770664479203` (Hypothesis 3)

**Storage**: `.quint/knowledge/L0/`
**Status**: Ready for Phase 2 verification
**R_eff Goal**: 0.95 (high confidence needed for git workflow)
