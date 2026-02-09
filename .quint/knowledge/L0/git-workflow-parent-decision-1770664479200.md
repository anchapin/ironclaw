# Git Workflow Automation Decision

**Holon ID**: git-workflow-parent-decision-1770664479200
**Kind**: episteme
**Created**: 2026-02-09
**Scope**: IronClaw repository, GitHub-hosted, AI-assisted development

---

## Problem Statement (Anomaly)

**Current State**: The repository allows direct pushes to the `main` branch, creating risks:
1. AI coding agents can push directly to main without review
2. No automated link between GitHub issues and pull requests
3. Missing traceability between work items and code changes
4. No enforcement of feature branch workflow

**Desired State**: All code changes must follow:
1. Create GitHub issue first (tracking)
2. Create feature branch from issue
3. Make changes on feature branch
4. Create PR linked to issue
5. Merge only through PR approval

**Gap**: No automated enforcement mechanism exists

---

## Decision Context

This is a **parent decision** framing the problem space. Child hypotheses will propose specific implementation approaches.

**Inheritance**: Child hypotheses MUST address ALL aspects:
- Branch protection (prevent direct main pushes)
- Issue tracking (gh CLI integration)
- PR workflow (enforce review process)
- Automation (pre-commit hooks, GitHub Actions)

---

## Success Criteria

Any solution MUST:
1. **Block direct pushes** to `main` branch (non-negotiable)
2. **Require GitHub issue** before creating feature branch
3. **Link PR to issue** using GitHub's automatic linking
4. **Use gh CLI** for all automation (standard tooling)
5. **Update CLAUDE.md** with workflow instructions
6. **Pass pre-commit hooks** before PR creation

---

## Constraints

**Technical**:
- Must use GitHub-native features (branch protection, checks)
- Must integrate with existing pre-commit hooks
- Must not require custom GitHub Apps (use gh CLI only)
- Must work with AI coding agents (Claude Code, GitHub Copilot)

**Operational**:
- Setup must be documented in CLAUDE.md
- Must be reproducible via scripts
- Must not break existing development workflow

---

## Dependency Context

This decision DEPENDS ON:
- Existing GitHub repository with `main` branch
- Installed `gh` CLI tool
- Existing pre-commit hooks configured
- Git repository initialized

---

## Child Hypotheses

The following approaches will be evaluated:

1. **GitHub Native + gh CLI** (Conservative)
   - Use GitHub branch protection rules
   - Pre-commit hook to check branch name
   - Shell script wrapper for git commands

2. **Pre-commit Hook + CI Validation** (Moderate)
   - Server-side pre-commit hook to block main commits
   - GitHub Actions to validate issue linkage
   - Automated PR creation workflow

3. **Multi-Layer Defense** (Comprehensive)
   - Combine GitHub branch protection
   - Pre-commit hook for branch validation
   - GitHub Actions for PR validation
   - Custom git aliases for workflow automation

---

## Audit Metadata

**R_eff Goal**: 0.95 (high confidence needed for git workflow)
**Complexity**: Medium (requires GitHub API integration)
**Risk Level**: Low (reversible changes, no data migration)
**Decision Deadline**: Phase 1 Foundation completion
