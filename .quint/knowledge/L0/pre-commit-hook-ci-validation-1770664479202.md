# Hypothesis: Pre-commit Hook + GitHub Actions CI Validation

**Holon ID**: pre-commit-hook-ci-validation-1770664479202
**Kind**: system
**Created**: 2026-02-09
**Decision Context**: git-workflow-parent-decision-1770664479200
**Depends On**: []

---

## Title

Use client-side pre-commit hooks to block main commits + GitHub Actions to validate PR-issue linkage

---

## Method (Recipe)

### Implementation Steps

#### 1. Enhanced Pre-commit Hook: Multi-Layer Defense

Create `.git/hooks/pre-commit` (or add to `.pre-commit-config.yaml`):

```bash
#!/bin/bash
set -e

# Layer 1: Block commits to protected branches
PROTECTED_BRANCHES="^(main|master|develop)$"
CURRENT_BRANCH=$(git rev-parse --abbrev-ref HEAD)

if [[ "$CURRENT_BRANCH" =~ $PROTECTED_BRANCHES ]]; then
  echo "❌ BLOCKED: Cannot commit directly to $CURRENT_BRANCH"
  echo ""
  echo "Required workflow:"
  echo "  1. Create GitHub issue: gh issue create --title 'Description'"
  echo "  2. Create feature branch: git checkout -b feature/ISSUE-NUM-description"
  echo "  3. Make changes and commit"
  echo "  4. Create PR: gh pr create --body 'Closes #ISSUE-NUM'"
  exit 1
fi

# Layer 2: Validate branch name format
if [[ ! "$CURRENT_BRANCH" =~ ^feature/[0-9]+- ]]; then
  echo "⚠️  WARNING: Branch name doesn't match feature/ISSUE-NUM-description"
  echo "   PR creation may be blocked by CI"
  read -p "Continue anyway? (y/N) " -n 1 -r
  echo
  if [[ ! $REPLY =~ ^[Yy]$ ]]; then
    exit 1
  fi
fi

# Layer 3: Run existing pre-commit hooks
echo "🔍 Running pre-commit checks..."
cd agent && .venv/bin/pre-commit run --all-files || true

exit 0
```

#### 2. GitHub Actions: PR Validation Workflow

Create `.github/workflows/pr-validation.yml`:

```yaml
name: PR Validation

on:
  pull_request:
    types: [opened, edited, synchronize]

permissions:
  pull-requests: read

jobs:
  validate-issue-linkage:
    name: Validate Issue Linkage
    runs-on: ubuntu-latest
    steps:
      - name: Check PR body for issue reference
        env:
          PR_BODY: ${{ github.event.pull_request.body }}
        run: |
          if ! echo "$PR_BODY" | grep -qE "(Closes|Fixes|Resolves) #[0-9]+"; then
            echo "❌ PR must reference an issue"
            echo "   Add 'Closes #ISSUE-NUM' to PR body"
            exit 1
          fi
          echo "✅ Issue linkage validated"

      - name: Extract issue number
        id: extract
        env:
          PR_BODY: ${{ github.event.pull_request.body }}
        run: |
          ISSUE=$(echo "$PR_BODY" | grep -oE "#[0-9]+" | head -1 | tr -d '#')
          echo "issue=$ISSUE" >> $GITHUB_OUTPUT

      - name: Validate issue exists
        env:
          ISSUE: ${{ steps.extract.outputs.issue }}
          GH_TOKEN: ${{ secrets.GITHUB_TOKEN }}
        run: |
          if ! gh issue view "$ISSUE" &>/dev/null; then
            echo "❌ Issue #$ISSUE does not exist"
            exit 1
          fi
          echo "✅ Issue #$ISSUE exists"

      - name: Comment on PR
        if: always()
        env:
          ISSUE: ${{ steps.extract.outputs.issue }}
          GH_TOKEN: ${{ secrets.GITHUB_TOKEN }}
        run: |
          gh pr comment "$PR_NUMBER" --body "✅ Validated against issue #$ISSUE"
        env:
          PR_NUMBER: ${{ github.event.pull_request.number }}

  validate-branch-name:
    name: Validate Branch Name
    runs-on: ubuntu-latest
    steps:
      - name: Check branch name format
        env:
          BRANCH: ${{ github.event.pull_request.head.ref }}
        run: |
          if ! [[ "$BRANCH" =~ ^feature/[0-9]+- ]]; then
            echo "❌ Branch must be feature/ISSUE-NUM-description"
            echo "   Current: $BRANCH"
            exit 1
          fi
          echo "✅ Branch name valid: $BRANCH"
```

#### 3. Required Status Check (Enforcement)

Add to `.github/workflows/pr-validation.yml`:

```yaml
# Add at the end
  required-check:
    name: Required Check
    runs-on: ubuntu-latest
    needs: [validate-issue-linkage, validate-branch-name]
    if: always()
    steps:
      - name: All checks passed
        run: |
          if [[ "${{ needs.validate-issue-linkage.result }}" != "success" ]] || \
             [[ "${{ needs.validate-branch-name.result }}" != "success" ]]; then
            echo "❌ PR validation failed"
            exit 1
          fi
          echo "✅ All validations passed"
```

Then configure branch protection to require this check:

```bash
gh api repos/:owner/:repo/branches/main/protection \
  --method PUT \
  -f required_status_checks='{"strict":true,"contexts":["PR Validation"]}' \
  -f enforce_admins=true \
  -f required_pull_request_reviews='{"required_approving_review_count":1}'
```

#### 4. Automation Script: Enhanced Wrapper

Create `scripts/git-workflow.sh`:

```bash
#!/bin/bash
set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

# Color output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

info() { echo -e "${GREEN}ℹ${NC} $1"; }
warn() { echo -e "${YELLOW}⚠${NC} $1"; }
error() { echo -e "${RED}❌${NC} $1"; }

# Command: start - Begin new feature
start_feature() {
  local ISSUE=$1
  local DESC=${2:-"feature"}

  # Validate issue exists
  if ! gh issue view "$ISSUE" &>/dev/null; then
    error "Issue #$ISSUE does not exist"
    echo ""
    echo "Create it first:"
    echo "  gh issue create --title 'Description' --body 'Details'"
    exit 1
  fi

  info "Issue #$ISSUE found: $(gh issue view "$ISSUE" --json title -q .title)"

  # Create branch
  local BRANCH="feature/$ISSUE-${DESC// /-}"
  git checkout -b "$BRANCH"

  info "Created branch: $BRANCH"
  info "Now make your changes and commit normally"
}

# Command: submit - Create PR
submit_pr() {
  local CURRENT_BRANCH=$(git rev-parse --abbrev-ref HEAD)

  # Validate branch name
  if [[ ! "$CURRENT_BRANCH" =~ ^feature/[0-9]+- ]]; then
    error "Branch name must be feature/ISSUE-NUM-description"
    exit 1
  fi

  # Extract issue number
  local ISSUE=$(echo "$CURRENT_BRANCH" | cut -d- -f2)

  # Check if PR already exists
  if gh pr list --head "$CURRENT_BRANCH" | grep -q .; then
    warn "PR already exists for this branch"
    gh pr view --web
    exit 0
  fi

  # Create PR
  info "Creating PR for issue #$ISSUE..."
  gh pr create \
    --title "Work on #$ISSUE: $(gh issue view "$ISSUE" --json title -q .title)" \
    --body "Closes #$ISSUE" \
    --base main

  info "PR created successfully"
  gh pr view --web
}

# Command: status - Show workflow status
show_status() {
  local CURRENT_BRANCH=$(git rev-parse --abbrev-ref HEAD)

  echo "Current branch: $CURRENT_BRANCH"

  if [[ "$CURRENT_BRANCH" =~ ^feature/[0-9]+- ]]; then
    local ISSUE=$(echo "$CURRENT_BRANCH" | cut -d- -f2)
    echo "Linked issue: #$ISSUE"
    gh issue view "$ISSUE" --json title,state,url || echo "Issue not found"
  else
    warn "Not a feature branch"
  fi
}

# Main
case "$1" in
  start)
    start_feature "$2" "$3"
    ;;
  submit)
    submit_pr
    ;;
  status)
    show_status
    ;;
  *)
    echo "Usage: $0 {start|submit|status}"
    echo ""
    echo "Examples:"
    echo "  $0 start 123 'Add MCP client'"
    echo "  $0 submit"
    echo "  $0 status"
    exit 1
    ;;
esac
```

#### 5. Update CLAUDE.md

Add comprehensive section:

```markdown
## Git Workflow (Automated Enforcement)

### Philosophy
AI coding agents must follow the same discipline as human developers. All code changes require:
1. **Issue Tracking** - Work must link to a GitHub issue
2. **Code Review** - Changes must go through PRs
3. **Validation** - Automated checks must pass

### Workflow

#### Starting New Work
```bash
# 1. Create GitHub issue
gh issue create \
  --title "Implement MCP client connection" \
  --body "Add ability to connect to MCP servers from orchestrator"
# Returns: Issue #42

# 2. Create feature branch (validates issue exists)
./scripts/git-workflow.sh start 42 "mcp-client-connection"
# Output: Created branch feature/42-mcp-client-connection

# 3. Work and commit (pre-commit hooks run automatically)
git add .
git commit -m "Add MCP client module"
```

#### Submitting Work
```bash
# Create PR (auto-links to issue from branch name)
./scripts/git-workflow.sh submit
# Output: PR created with body "Closes #42"
```

#### Checking Status
```bash
./scripts/git-workflow.sh status
# Shows current branch and linked issue
```

### Automated Validations

**Client-Side (Pre-commit)**:
- ❌ Blocks commits to `main`/`master`
- ⚠️  Warns on non-feature branch names
- 🔍 Runs existing pre-commit hooks

**Server-Side (GitHub Actions)**:
- ✅ Validates PR body contains issue reference
- ✅ Validates referenced issue exists
- ✅ Validates branch name format
- ✅ Comments on PR with validation results

**Branch Protection**:
- 🔒 Direct pushes to main are blocked
- ✅ PRs require 1 approval
- ✅ CI checks must pass
- ✅ PRs must link to issues

### Error Messages

If you try to commit to main:
```
❌ BLOCKED: Cannot commit directly to main

Required workflow:
  1. Create GitHub issue: gh issue create --title 'Description'
  2. Create feature branch: git checkout -b feature/ISSUE-NUM-description
  3. Make changes and commit
  4. Create PR: gh pr create --body 'Closes #ISSUE-NUM'
```

If you try to create PR without issue linkage:
```
❌ PR must reference an issue
   Add 'Closes #ISSUE-NUM' to PR body
```

### AI Agent Integration

When working with Claude Code, GitHub Copilot, or other AI agents:
1. The agent will naturally follow the workflow (blocked otherwise)
2. Pre-commit hooks guide the agent to correct process
3. GitHub Actions validate the agent's output
4. No manual enforcement needed - it's automatic
```

---

## Scope

**Applies To**:
- GitHub-hosted repositories with Actions enabled
- Teams using AI coding agents
- Projects requiring strong workflow enforcement
- Open source projects with external contributors

**Requirements**:
- GitHub Actions enabled (free for public repos)
- `gh` CLI installed
- Write access to `.github/workflows/`
- Admin access for branch protection setup

**Does NOT Apply To**:
- Local-only development (no GitHub remote)
- Repositories without GitHub Actions

---

## Rationale

```json
{
  "anomaly": "AI agents need multi-layer validation to ensure proper git hygiene",
  "approach": "Client-side pre-commit hooks (fast feedback) + server-side GitHub Actions (enforceable validation) + branch protection (hard block)",
  "alternatives_rejected": [
    "GitHub native only (no client-side feedback)",
    "Pre-commit only (can be bypassed)",
    "Manual process review (unenforceable)"
  ],
  "advantages": [
    "Pre-commit hooks = immediate feedback before commit",
    "GitHub Actions = server-side enforcement (can't bypass)",
    "Branch protection = ultimate safety net",
    "Comprehensive validation at multiple layers",
    "Clear error messages guide users (and AI agents)",
    "Fully automated - no manual review needed"
  ]
}
```

---

## Implementation Complexity

**Effort**: Medium-High (4-5 hours setup)
- Enhanced pre-commit hook: 60 minutes
- GitHub Actions workflow: 120 minutes
- Branch protection config: 30 minutes
- Enhanced git-workflow.sh: 90 minutes
- CLAUDE.md documentation: 60 minutes

**Risk**: Medium
- More moving parts (pre-commit + Actions + branch protection)
- GitHub Actions has learning curve
- Pre-commit hooks can be bypassed with --no-verify

**Maintenance**: Low-Medium
- GitHub Actions workflow may need updates
- Pre-commit hook logic may need refinement
- GitHub API changes could break validations

---

## Success Metrics

1. ✅ Pre-commit hook blocks local main commits
2. ✅ GitHub Actions validates PR-issue linkage
3. ✅ Branch name format enforced
4. ✅ Branch protection requires CI checks
5. ✅ Clear error messages at every layer
6. ✅ AI agents follow workflow automatically

---

## Failure Modes

| Failure | Mitigation |
|---------|------------|
| User bypasses pre-commit with --no-verify | GitHub Actions still validates server-side |
| GitHub Actions fails (rate limit, bug) | Branch protection still blocks direct pushes |
| Issue number extraction fails | Use robust regex, provide manual override |
| Branch protection not enabled | Document setup, add verification script |

---

## Dependencies

**Direct Dependencies**: None

**Indirect Dependencies**:
- GitHub Actions enabled
- GitHub token for API calls
- Existing pre-commit infrastructure

**Congruence Level**: CL3 (same context - all within repo automation)

---

## Next Steps (If Promoted)

1. Implement enhanced pre-commit hook
2. Create GitHub Actions workflow
3. Configure branch protection with required checks
4. Write enhanced git-workflow.sh wrapper
5. Update CLAUDE.md with comprehensive workflow docs
6. Test end-to-end with AI coding agent
7. Create setup verification script
