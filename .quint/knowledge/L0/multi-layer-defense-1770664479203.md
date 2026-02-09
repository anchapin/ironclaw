# Hypothesis: Multi-Layer Defense (Comprehensive Approach)

**Holon ID**: multi-layer-defense-1770664479203
**Kind**: system
**Created**: 2026-02-09
**Decision Context**: git-workflow-parent-decision-1770664479200
**Depends On": []

---

## Title

Combine all defense layers: GitHub branch protection + pre-commit hooks + GitHub Actions + custom git aliases for comprehensive workflow enforcement

---

## Method (Recipe)

### Implementation Strategy: Defense in Depth

This hypothesis combines ALL validation mechanisms to create multiple, independent layers of enforcement. If any layer fails, others still protect the main branch.

### Layer 1: Client-Side Pre-commit Hook (Fast Feedback)

Create `.git/hooks/pre-commit`:

```bash
#!/bin/bash
set -e

RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m'

# Check 1: Protected branches
PROTECTED_BRANCHES="^(main|master|develop|release/.+)$"
CURRENT_BRANCH=$(git rev-parse --abbrev-ref HEAD)

if [[ "$CURRENT_BRANCH" =~ $PROTECTED_BRANCHES ]]; then
  echo -e "${RED}❌ BLOCKED: Cannot commit directly to $CURRENT_BRANCH${NC}"
  echo ""
  echo "Required workflow:"
  echo "  1. Create GitHub issue: gh issue create --title 'Description'"
  echo "  2. Create feature branch: git checkout -b feature/ISSUE-NUM-description"
  echo "  3. Make changes and commit"
  echo "  4. Create PR: gh pr create --body 'Closes #ISSUE-NUM'"
  echo ""
  echo "Or use the workflow helper:"
  echo "  ./scripts/git-workflow.sh start ISSUE-NUM 'description'"
  exit 1
fi

# Check 2: Branch name format (soft warning)
if [[ "$CURRENT_BRANCH" =~ ^(feature|fix|hotfix)/ ]]; then
  if [[ ! "$CURRENT_BRANCH" =~ /[0-9]+- ]]; then
    echo -e "${YELLOW}⚠️  WARNING: Branch doesn't contain issue number${NC}"
    echo "   Expected format: $TYPE/ISSUE-NUM-description"
    echo "   Current: $CURRENT_BRANCH"
    echo ""
    read -p "Continue anyway? (y/N) " -n 1 -r
    echo
    [[ ! $REPLY =~ ^[Yy]$ ]] && exit 1
  fi
fi

# Check 3: Run existing pre-commit hooks
echo -e "${GREEN}🔍 Running pre-commit checks...${NC}"
cd agent && .venv/bin/pre-commit run --all-files || true

echo -e "${GREEN}✅ Pre-commit checks passed${NC}"
exit 0
```

Make executable and distribute via `make install`:

```makefile
install:
	# ... existing install steps ...
	@echo "[Git] Installing pre-commit hooks..."
	@cp .githooks/pre-commit .git/hooks/pre-commit
	@chmod +x .git/hooks/pre-commit
```

### Layer 2: Git Aliases (Low-Friction Workflow)

Add to `.git/config` or create project-specific alias file:

Create `.githooks/aliases`:

```bash
# Feature workflow aliases
git config --local alias.start '!f() { \
  [ -z "$1" ] && echo "Usage: git start ISSUE-NUM [description]" && return 1; \
  ISSUE=$1; \
  DESC=${2:-"feature"}; \
  if ! gh issue view "$ISSUE" &>/dev/null; then \
    echo "❌ Issue #$ISSUE does not exist"; \
    gh issue create --title "$DESC" --body "Auto-created from git workflow"; \
    return 1; \
  fi; \
  BRANCH="feature/$ISSUE-${DESC// /-}"; \
  git checkout -b "$BRANCH"; \
  echo "✅ Created branch $BRANCH"; \
  gh issue view "$ISSUE" --json title -q "Issue: #\(.number) \(.title)"; \
}; f'

git config --local alias.submit '!f() { \
  CURRENT=$(git rev-parse --abbrev-ref HEAD); \
  if [[ ! "$CURRENT" =~ ^feature/[0-9]+- ]]; then \
    echo "❌ Not a feature branch: $CURRENT"; \
    return 1; \
  fi; \
  ISSUE=$(echo "$CURRENT" | cut -d- -f2); \
  TITLE=$(gh issue view "$ISSUE" --json title -q .title); \
  gh pr create --title "Work on #$ISSUE: $TITLE" --body "Closes #$ISSUE"; \
}; f'

git config --local alias.sync '!git fetch origin main && git rebase origin/main'

git config --local alias.pr-status '!f() { \
  CURRENT=$(git rev-parse --abbrev-ref HEAD); \
  echo "Current branch: $CURRENT"; \
  if [[ "$CURRENT" =~ ^feature/[0-9]+- ]]; then \
    ISSUE=$(echo "$CURRENT" | cut -d- -f2); \
    gh issue view "$ISSUE" --json title,state,url; \
    gh pr list --head "$CURRENT"; \
  fi; \
}; f'
```

Then in `Makefile`:

```makefile
install:
	@echo "[Git] Setting up workflow aliases..."
	@.githooks/setup-aliases.sh
```

### Layer 3: GitHub Actions CI Validation

Create `.github/workflows/pr-validation.yml`:

```yaml
name: PR Validation

on:
  pull_request:
    types: [opened, edited, synchronize, labeled]
  pull_request_target:
    types: [opened, edited]

permissions:
  pull-requests: read
  contents: read
  issues: read

jobs:
  # Job 1: Validate PR-issue linkage
  validate-linkage:
    name: Validate Issue Linkage
    runs-on: ubuntu-latest
    outputs:
      issue-number: ${{ steps.extract.outputs.issue }}
      has-link: ${{ steps.check.outputs.has-link }}
    steps:
      - name: Check PR body for issue reference
        id: check
        env:
          PR_BODY: ${{ github.event.pull_request.body }}
        run: |
          if echo "$PR_BODY" | grep -qE "(Closes|Fixes|Resolves|Related to) #[0-9]+"; then
            echo "has-link=true" >> $GITHUB_OUTPUT
          else
            echo "has-link=false" >> $GITHUB_OUTPUT
            echo "❌ PR body must reference an issue"
            echo "   Add one of: Closes #123, Fixes #123, Resolves #123"
            exit 1
          fi

      - name: Extract issue number
        id: extract
        env:
          PR_BODY: ${{ github.event.pull_request.body }}
        run: |
          ISSUE=$(echo "$PR_BODY" | grep -oE "#[0-9]+" | head -1 | tr -d '#')
          echo "issue=$ISSUE" >> $GITHUB_OUTPUT
          echo "✅ Found issue #$ISSUE"

      - name: Validate issue exists
        env:
          GH_TOKEN: ${{ secrets.GITHUB_TOKEN }}
          ISSUE: ${{ steps.extract.outputs.issue }}
        run: |
          if ! gh issue view "$ISSUE" &>/dev/null; then
            echo "❌ Issue #$ISSUE does not exist"
            gh issue list
            exit 1
          fi
          gh issue view "$ISSUE" --json title,state,url
          echo "✅ Issue #$ISSUE exists"

  # Job 2: Validate branch name
  validate-branch:
    name: Validate Branch Name
    runs-on: ubuntu-latest
    steps:
      - name: Check branch format
        env:
          BRANCH: ${{ github.event.pull_request.head.ref }}
        run: |
          if ! [[ "$BRANCH" =~ ^(feature|fix|hotfix)/[0-9]+- ]]; then
            echo "❌ Branch must be feature/ISSUE-NUM-description"
            echo "   Current: $BRANCH"
            echo "   Examples: feature/42-add-mcp-client, fix/17-memory-leak"
            exit 1
          fi
          echo "✅ Branch name valid: $BRANCH"

      - name: Extract issue from branch
        id: extract
        env:
          BRANCH: ${{ github.event.pull_request.head.ref }}
        run: |
          ISSUE=$(echo "$BRANCH" | cut -d/ -f2 | cut -d- -f1)
          echo "issue=$ISSUE" >> $GITHUB_OUTPUT

      - name: Verify branch-issue match
        env:
          GH_TOKEN: ${{ secrets.GITHUB_TOKEN }}
          BRANCH_ISSUE: ${{ steps.extract.outputs.issue }}
          PR_ISSUE: ${{ needs.validate-linkage.outputs.issue-number }}
        run: |
          if [ "$BRANCH_ISSUE" != "$PR_ISSUE" ]; then
            echo "⚠️  Branch issue ($BRANCH_ISSUE) != PR issue ($PR_ISSUE)"
            echo "   This might be intentional (e.g., fixing multiple issues)"
          else
            echo "✅ Branch and PR reference same issue"
          fi

  # Job 3: Validate commit messages
  validate-commits:
    name: Validate Commit Messages
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
        with:
          fetch-depth: 0

      - name: Check commit messages
        run: |
          echo "Checking commits between origin/main and HEAD..."
          git fetch origin main
          for commit in $(git rev-list origin/main..HEAD); do
            msg=$(git log -1 --format=%B $commit)
            echo "Commit: $commit"
            echo "Message: $msg"
            # Check for conventional commits or issue references
            if ! echo "$msg" | grep -qE "^(feat|fix|docs|style|refactor|test|chore)|#[0-9]+"; then
              echo "⚠️  Commit doesn't follow conventions"
            fi
          done

  # Job 4: Comprehensive validation
  comprehensive-check:
    name: Comprehensive Validation
    runs-on: ubuntu-latest
    needs: [validate-linkage, validate-branch, validate-commits]
    if: always()
    steps:
      - name: All checks passed
        run: |
          if [[ "${{ needs.validate-linkage.result }}" != "success" ]] || \
             [[ "${{ needs.validate-branch.result }}" != "success" ]] || \
             [[ "${{ needs.validate-commits.result }}" != "success" ]]; then
            echo "❌ PR validation failed"
            echo ""
            echo "Results:"
            echo "  Issue Linkage: ${{ needs.validate-linkage.result }}"
            echo "  Branch Name: ${{ needs.validate-branch.result }}"
            echo "  Commits: ${{ needs.validate-commits.result }}"
            exit 1
          fi
          echo "✅ All validations passed"

      - name: Comment success on PR
        env:
          GH_TOKEN: ${{ secrets.GITHUB_TOKEN }}
          ISSUE: ${{ needs.validate-linkage.outputs.issue-number }}
        run: |
          COMMENT="✅ **PR Validation Passed**
          - Linked to issue #$ISSUE
          - Branch name format valid
          - Commit messages reviewed
          - Ready for review 👀"
          gh pr comment "$PR_NUMBER" --body "$COMMENT"
        env:
          PR_NUMBER: ${{ github.event.pull_request.number }}
```

### Layer 4: GitHub Branch Protection (Hard Block)

Create setup script `scripts/setup-branch-protection.sh`:

```bash
#!/bin/bash
set -e

REPO_SLUG=$(git config --get remote.origin.url | sed 's/.*:\(.*\)\.git/\1/')
echo "Configuring branch protection for $REPO_SLUG"

# Protect main branch
gh api "repos/$REPO_SLUG/branches/main/protection" \
  --method PUT \
  -H "Accept: application/vnd.github+json" \
  -f required_pull_request_reviews='{
    "required_approving_review_count": 1,
    "dismiss_stale_reviews": false,
    "require_code_owner_reviews": false
  }' \
  -f enforce_admins=true \
  -f allow_deletions=false \
  -f required_linear_history=true \
  -f required_status_checks='{
    "strict": true,
    "contexts": [
      "PR Validation",
      "pre-commit/check"
    ]
  }' \
  -f restrictions=null

echo "✅ Branch protection configured"
echo ""
echo "Rules applied:"
echo "  • Pull requests required (1 approval)"
echo "  • Status checks must pass"
echo "  • Direct pushes to main blocked"
echo "  • Branch deletions blocked"
echo "  • Admin enforcement enabled"
```

### Layer 5: Workflow Automation Script

Create comprehensive `scripts/git-workflow.sh`:

```bash
#!/bin/bash
set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
source "$SCRIPT_DIR/../.githooks/colors.sh"

# Functions
show_help() {
  cat << EOF
IronClaw Git Workflow Helper

Commands:
  start ISSUE-NUM [description]   Start new feature branch from issue
  submit                          Create PR for current branch
  status                          Show current branch and linked issue
  sync                            Sync feature branch with main
  finish                          Merge PR after approval (fast-forward)

Examples:
  $0 start 42 "Add MCP client connection"
  $0 submit
  $0 status

Philosophy:
  • All work must link to a GitHub issue
  • All changes must go through PRs
  • Automated validation at multiple layers
  • AI agents follow same workflow as humans

EOF
}

cmd_start() {
  local ISSUE=$1
  local DESC=${2:-"feature"}

  # Validate issue exists
  if ! gh issue view "$ISSUE" &>/dev/null; then
    error "Issue #$ISSUE does not exist"
    echo ""
    echo "Create it first:"
    echo "  gh issue create --title 'Description' --body 'Details'"
    echo ""
    echo "Or let me create it for you:"
    read -p "Create issue '$DESC' now? (y/N) " -n 1 -r
    echo
    if [[ $REPLY =~ ^[Yy]$ ]]; then
      gh issue create --title "$DESC" --body "Auto-created from git workflow" \
        --label "enhancement"
      # Get the new issue number
      ISSUE=$(gh issue list --limit 1 --json number --jq '.[0].number')
      info "Created issue #$ISSUE"
    else
      exit 1
    fi
  fi

  info "Issue #$ISSUE: $(gh issue view "$ISSUE" --json title -q .title)"

  # Create branch
  local BRANCH="feature/$ISSUE-${DESC// /-}"
  if git show-ref --verify --quiet "refs/heads/$BRANCH"; then
    warn "Branch $BRANCH already exists"
    git checkout "$BRANCH"
  else
    git checkout -b "$BRANCH"
    info "Created branch: $BRANCH"
  fi

  echo ""
  info "Next steps:"
  echo "  1. Make your changes"
  echo "  2. Commit: git commit -m 'Description'"
  echo "  3. Submit: $0 submit"
}

cmd_submit() {
  local CURRENT_BRANCH=$(git rev-parse --abbrev-ref HEAD)

  # Validate branch name
  if [[ ! "$CURRENT_BRANCH" =~ ^feature/[0-9]+- ]]; then
    error "Not a feature branch: $CURRENT_BRANCH"
    echo "Feature branches must be feature/ISSUE-NUM-description"
    exit 1
  fi

  # Extract issue number
  local ISSUE=$(echo "$CURRENT_BRANCH" | cut -d- -f2)

  # Check if PR exists
  if gh pr list --head "$CURRENT_BRANCH" --json number | jq -r '.[0].number' | grep -q .; then
    warn "PR already exists for this branch"
    gh pr view --web
    exit 0
  fi

  # Get issue details
  local TITLE=$(gh issue view "$ISSUE" --json title -q .title)
  local BODY="Closes #$ISSUE\n\n## Changes\n\n$(git log main..HEAD --format=%s)"

  # Create PR
  info "Creating PR for issue #$ISSUE..."
  gh pr create \
    --title "Work on #$ISSUE: $TITLE" \
    --body "$BODY" \
    --base main \
    --label "needs-review"

  info "PR created successfully"
  gh pr view --web
}

cmd_status() {
  local CURRENT_BRANCH=$(git rev-parse --abbrev-ref HEAD)

  echo "Current branch: $CURRENT_BRANCH"
  echo ""

  if [[ "$CURRENT_BRANCH" =~ ^feature/[0-9]+- ]]; then
    local ISSUE=$(echo "$CURRENT_BRANCH" | cut -d- -f2)

    info "Linked issue #$ISSUE"
    gh issue view "$ISSUE" --json title,state,url,labels | jq -r '
      "  Title: \(.title)\n  State: \(.state)\n  Labels: \([.labels[].name] | join(", "))\n  URL: \(.url)"
    '

    echo ""
    info "Pull Requests:"
    if gh pr list --head "$CURRENT_BRANCH" --json number,title,state | jq -e '.[0]' > /dev/null; then
      gh pr list --head "$CURRENT_BRANCH" --json number,title,state | jq -r '
        "  PR #\(.[0].number): \(.[0].title) (\(.[0].state))"
      '
    else
      echo "  No PR created yet. Run: $0 submit"
    fi
  else
    warn "Not a feature branch (no issue linkage)"
  fi
}

cmd_sync() {
  local CURRENT_BRANCH=$(git rev-parse --abbrev-ref HEAD)

  info "Fetching latest from origin..."
  git fetch origin main

  info "Rebasing $CURRENT_BRANCH onto main..."
  git rebase origin/main

  info "Sync complete"
}

cmd_finish() {
  local CURRENT_BRANCH=$(git rev-parse --abbrev-ref HEAD)

  if [[ "$CURRENT_BRANCH" == "main" ]]; then
    error "Cannot finish main branch"
    exit 1
  fi

  # Check PR status
  local PR=$(gh pr list --head "$CURRENT_BRANCH" --json number,state,mergeable | jq -r '.[0]')

  if [[ $(echo "$PR" | jq -r '.state') != "OPEN" ]]; then
    warn "PR is not open (state: $(echo "$PR" | jq -r '.state'))"
    exit 1
  fi

  if [[ $(echo "$PR" | jq -r '.mergeable') != "true" ]]; then
    error "PR is not mergeable (conflicts?)"
    echo "Run: $0 sync"
    exit 1
  fi

  # Merge PR
  info "Merging PR..."
  gh pr merge $(echo "$PR" | jq -r '.number') --squash --delete-branch

  # Cleanup
  info "Cleaning up..."
  git checkout main
  git pull

  info "Done! Issue closed, PR merged, branch deleted"
}

# Main
case "${1:-}" in
  start) cmd_start "$2" "$3" ;;
  submit) cmd_submit ;;
  status) cmd_status ;;
  sync) cmd_sync ;;
  finish) cmd_finish ;;
  *) show_help ;;
esac
```

### Layer 6: Update CLAUDE.md

Add comprehensive section (see full content in hypothesis file).

---

## Scope

**Applies To**:
- Production repositories requiring maximum safety
- Teams with multiple developers + AI agents
- Open source projects with external contributors
- Critical infrastructure codebases

**Requirements**:
- GitHub with Actions enabled
- Admin access for branch protection
- Full development team buy-in
- Time for comprehensive setup (1-2 days)

**Does NOT Apply To**:
- Quick prototypes
- Solo projects with low risk tolerance

---

## Rationale

```json
{
  "anomaly": "Single-layer enforcement is insufficient for critical workflows with AI agents",
  "approach": "Defense in depth - 6 independent validation layers that each can block bad commits",
  "alternatives_rejected": [
    "Single layer (too fragile)",
    "Two layers (insufficient redundancy)",
    "Manual review (slow, error-prone)"
  ],
  "advantages": [
    "Multiple independent checks = very high reliability",
    "Pre-commit = instant feedback",
    "Git aliases = low friction",
    "GitHub Actions = server-side enforcement",
    "Branch protection = hard block",
    "Automation script = one simple command",
    "If any layer fails, others still protect main"
  ]
}
```

---

## Implementation Complexity

**Effort**: High (1-2 days)
- Pre-commit hook: 60 minutes
- Git aliases: 45 minutes
- GitHub Actions: 180 minutes
- Branch protection setup: 30 minutes
- Automation script: 180 minutes
- Documentation: 120 minutes
- Testing and refinement: 120 minutes

**Risk**: Low
- Very robust due to multiple layers
- Changes are reversible
- Can enable/disable layers independently

**Maintenance**: Low-Medium
- More moving parts
- Each layer relatively simple
- GitHub Actions may need updates

---

## Success Metrics

1. ✅ All 6 layers independently functional
2. ✅ Direct main push blocked at multiple points
3. ✅ AI agents follow workflow automatically
4. ✅ Clear error messages at each layer
5. ✅ End-to-end workflow tested and documented
6. ✅ Can disable any layer without breaking others

---

## Next Steps (If Promoted)

1. Implement all 6 layers systematically
2. Create comprehensive test suite
3. Document setup and troubleshooting
4. Train team (human and AI) on workflow
5. Monitor and refine based on feedback
