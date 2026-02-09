# Hypothesis: GitHub Native Branch Protection + gh CLI Automation

**Holon ID**: github-native-gh-cli-1770664479201
**Kind**: system
**Created**: 2026-02-09
**Decision Context**: git-workflow-parent-decision-1770664479200
**Depends On**: []

---

## Title

Use GitHub native branch protection rules with gh CLI automation to enforce feature branch workflow

---

## Method (Recipe)

### Implementation Steps

#### 1. GitHub Branch Protection Rule (via gh CLI)

```bash
# Protect main branch - require PR + review
gh api repos/:owner/:repo/branches/main/protection \
  --method PUT \
  -f required_pull_request_reviews='{"required_approving_review_count":1}' \
  -f enforce_admins=true \
  -f allow_deletions=false \
  -f required_status_checks='{"strict":true,"contexts":["pre-commit/check"]}'
```

**Result**: Direct pushes to main are blocked at GitHub level

#### 2. Pre-commit Hook: Branch Name Validation

Create `.git/hooks/pre-commit`:

```bash
#!/bin/bash
# Block commits to main/master locally
CURRENT_BRANCH=$(git rev-parse --abbrev-ref HEAD)
if [[ "$CURRENT_BRANCH" =~ ^(main|master)$ ]]; then
  echo "❌ Cannot commit directly to $CURRENT_BRANCH"
  echo "   Create a feature branch: git checkout -b feature/ISSUE-NUM-description"
  exit 1
fi
exit 0
```

#### 3. Git Alias: Feature Branch Workflow

Add to `.git/config` or `~/.gitconfig`:

```ini
[alias]
  # Start new feature from issue
  start = "!f() { ISSUE=$1; DESC=$2; git checkout -b feature/$ISSUE-${DESC// /-}; gh issue view $ISSUE; }; f"
  # Submit PR linked to issue
  submit = "!f() { gh pr create --title \"$(git rev-parse --abbrev-ref HEAD | cut -d- -f2-)\" --body \"Closes #$1\"; }; f"
  # Sync with main
  sync = "!git fetch origin main && git rebase origin/main"
```

#### 4. Shell Wrapper: `git-workflow` Script

Create `scripts/git-workflow.sh`:

```bash
#!/bin/bash
# Wrapper for gh CLI to ensure issue exists first

case "$1" in
  start)
    ISSUE=$2
    if ! gh issue view "$ISSUE" &>/dev/null; then
      echo "❌ Issue #$ISSUE does not exist. Create it first:"
      echo "   gh issue create --title 'Description' --body 'Details'"
      exit 1
    fi
    BRANCH="feature/$ISSUE-${3// /-}"
    git checkout -b "$BRANCH"
    echo "✅ Created branch $BRANCH for issue #$ISSUE"
    ;;
  submit)
    # Extract issue number from branch name
    ISSUE=$(git rev-parse --abbrev-ref HEAD | cut -d- -f2)
    gh pr create --title "Work on #$ISSUE" --body "Closes #$ISSUE"
    ;;
esac
```

#### 5. Update CLAUDE.md

Add section:

```markdown
## Git Workflow (AI-Agent Enforced)

### Starting New Work
1. Create GitHub issue first:
   ```bash
   gh issue create --title "Add MCP client" --body "Implementation details..."
   # Returns: Issue #123
   ```

2. Create feature branch:
   ```bash
   ./scripts/git-workflow.sh start 123 "mcp-client"
   # Creates: feature/123-mcp-client
   ```

3. Work and commit normally:
   ```bash
   git add .
   git commit -m "Implement MCP connection"
   ```

4. Submit PR:
   ```bash
   ./scripts/git-workflow.sh submit
   # Creates PR linked to issue #123
   ```

### Branch Protection Rules
- ❌ Direct pushes to main are BLOCKED
- ✅ PRs require 1 approval
- ✅ Pre-commit checks must pass
- ✅ PRs must link to existing issue
```

---

## Scope

**Applies To**:
- GitHub-hosted repositories
- Teams using AI coding agents (Claude Code, Copilot, etc.)
- Projects requiring issue tracking traceability

**Requirements**:
- `gh` CLI installed and authenticated
- GitHub repo admin access
- Git 2.30+ for branch protection features

**Does NOT Apply To**:
- Local-only development (no GitHub remote)
- Emergency hotfixes (would require admin override)
- External contributors without repo access

---

## Rationale

```json
{
  "anomaly": "AI agents can push directly to main, bypassing review and issue tracking",
  "approach": "Use GitHub's native branch protection API (enforced server-side) combined with client-side pre-commit hooks and gh CLI automation",
  "alternatives_rejected": [
    "Server-side git hooks (too complex, requires GitHub Enterprise)",
    "Manual process documentation (unenforceable, agents will ignore)",
    "Custom GitHub App (over-engineering, maintenance burden)"
  ],
  "advantages": [
    "GitHub native = most reliable enforcement point",
    "gh CLI = official GitHub tool, well-documented",
    "Pre-commit hook = local safety net before push rejection",
    "Git aliases = low-friction workflow for developers",
    "Fully reversible if needed"
  ]
}
```

---

## Implementation Complexity

**Effort**: Medium (2-3 hours setup)
- Branch protection: 15 minutes (gh CLI command)
- Pre-commit hook: 30 minutes (script + testing)
- Git aliases: 45 minutes (documentation + testing)
- Shell wrapper: 60 minutes (error handling + edge cases)
- CLAUDE.md update: 30 minutes

**Risk**: Low
- No data migration
- Changes are reversible
- GitHub settings can be reverted via UI or gh CLI

**Maintenance**: Low
- No ongoing maintenance
- Scripts are self-contained
- GitHub handles enforcement

---

## Success Metrics

1. ✅ Direct push to main fails with clear error message
2. ✅ `git-workflow.sh start` validates issue exists
3. ✅ Created PRs automatically link to issues
4. ✅ Pre-commit hook blocks local main commits
5. ✅ CLAUDE.md documents complete workflow
6. ✅ AI coding agents follow the workflow automatically

---

## Failure Modes

| Failure | Mitigation |
|---------|------------|
| gh CLI not installed | Document in setup, add check in script |
| User not repo admin | Document admin requirements, provide setup guide |
| Pre-commit hook not executable | Add executable bit in make install |
| Branch name parsing fails | Use robust regex, provide manual override |

---

## Dependencies

**Direct Dependencies**: None (standalone implementation)

**Indirect Dependencies**:
- Existing GitHub repository
- `gh` CLI tool
- Git installation

**Congruence Level**: CL3 (same context - all within repo automation)

---

## Next Steps (If Promoted)

1. Create branch protection setup script
2. Implement pre-commit hook
3. Write git-workflow.sh wrapper
4. Add git aliases to project docs
5. Update CLAUDE.md with workflow section
6. Test with AI coding agent (verify workflow is followed)
