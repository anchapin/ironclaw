#!/bin/bash
# PR Swarmit - Parallel PR Investigation and Fixing
# Usage: /pr-swarmit [options]

set -euo pipefail

# Default values
REPO_ROOT="$(git rev-parse --show-toplevel)"
WORKTREE_BASE="/tmp/$(basename "$REPO_ROOT")-pr-worktrees"
DRY_RUN=false
SPECIFIC_PRS=""

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Logging functions
log_info() { echo -e "${BLUE}[INFO]${NC} $1"; }
log_success() { echo -e "${GREEN}[SUCCESS]${NC} $1"; }
log_warning() { echo -e "${YELLOW}[WARNING]${NC} $1"; }
log_error() { echo -e "${RED}[ERROR]${NC} $1"; }

# Parse arguments
while [[ $# -gt 0 ]]; do
    case $1 in
        --dry-run)
            DRY_RUN=true
            shift
            ;;
        --prs)
            SPECIFIC_PRS="$2"
            shift 2
            ;;
        --help)
            echo "Usage: $0 [options]"
            echo "Options:"
            echo "  --dry-run       Show what would be done without making changes"
            echo "  --prs NUMBERS   Comma-separated list of PR numbers to fix"
            echo "  --help          Show this help message"
            exit 0
            ;;
        *)
            log_error "Unknown option: $1"
            exit 1
            ;;
    esac
done

# Ensure we're in a git repo
if ! git rev-parse --git-dir > /dev/null 2>&1; then
    log_error "Not in a git repository"
    exit 1
fi

# Create worktree base directory
mkdir -p "$WORKTREE_BASE"

# Function to get all open PRs
get_open_prs() {
    if [[ -n "$SPECIFIC_PRS" ]]; then
        echo "$SPECIFIC_PRS" | tr ',' '\n' | while read -r pr; do
            gh pr view "$pr" --json number,title,headRefName,statusCheckRollup
        done
    else
        gh pr list --state open --json number,title,headRefName,statusCheckRollup
    fi
}

# Function to check if PR has failing checks
has_failing_checks() {
    local pr_number=$1
    local failing_checks

    failing_checks=$(gh pr checks "$pr_number" --watch=false 2>&1 | grep -c "fail" || true)

    [[ "$failing_checks" -gt 0 ]]
}

# Function to get failure type for a PR
get_failure_type() {
    local pr_number=$1

    if gh pr checks "$pr_number" --watch=false 2>&1 | grep -q "Enforce Ratchet.*fail"; then
        echo "coverage_ratchet"
    elif gh pr checks "$pr_number" --watch=false 2>&1 | grep -q "Test Rust.*fail"; then
        echo "compilation"
    elif gh pr checks "$pr_number" --watch=false 2>&1 | grep -q "Bloat Detection.*fail"; then
        echo "bloat"
    elif gh pr checks "$pr_number" --watch=false 2>&1 | grep -q "Complexity.*fail"; then
        echo "complexity"
    else
        echo "unknown"
    fi
}

# Function to categorize PRs by failure type
categorize_prs() {
    local coverage_prs=()
    local compilation_prs=()
    local other_prs=()
    local passing_prs=()

    while IFS= read -r pr_number; do
        if ! has_failing_checks "$pr_number"; then
            passing_prs+=("$pr_number")
            continue
        fi

        failure_type=$(get_failure_type "$pr_number")

        case "$failure_type" in
            coverage_ratchet)
                coverage_prs+=("$pr_number")
                ;;
            compilation)
                compilation_prs+=("$pr_number")
                ;;
            *)
                other_prs+=("$pr_number")
                ;;
        esac
    done < <(get_open_prs | jq -r '.[].number')

    echo "COVERAGE_RATCHET: ${coverage_prs[*]:-none}"
    echo "COMPILATION: ${compilation_prs[*]:-none}"
    echo "OTHER: ${other_prs[*]:-none}"
    echo "PASSING: ${passing_prs[*]:-none}"
}

# Function to create worktree for a PR
create_worktree() {
    local pr_number=$1
    local branch_name=$2
    local worktree_path="$WORKTREE_BASE/pr-$pr_number"

    log_info "Creating worktree for PR #$pr_number at $worktree_path"

    if [[ -d "$worktree_path" ]]; then
        log_warning "Worktree already exists, removing..."
        git worktree remove "$worktree_path" 2>/dev/null || true
    fi

    if [[ "$DRY_RUN" == true ]]; then
        log_info "[DRY RUN] Would create: git worktree add $worktree_path $branch_name"
        return 0
    fi

    git worktree add "$worktree_path" "$branch_name"
    log_success "Worktree created for PR #$pr_number"
}

# Function to get PR details
get_pr_details() {
    local pr_number=$1

    gh pr view "$pr_number" --json title,headRefName,number,statusCheckRollup | jq -r '
        "PR #" + (.number | tostring) + ": " + .title + "\n" +
        "Branch: " + .headRefName + "\n" +
        "Status: " + (
            if .statusCheckRollup then
                (.statusCheckRollup | length | tostring) + " checks"
            else
                "No checks"
            end
        )
    '
}

# Main execution
main() {
    log_info "Starting PR Swarmit..."
    log_info "Repository: $REPO_ROOT"
    log_info "Worktree base: $WORKTREE_BASE"

    # Get and categorize PRs
    log_info "Analyzing open PRs..."

    local pr_count
    pr_count=$(get_open_prs | jq 'length')

    log_info "Found $pr_count open PR(s)"

    if [[ "$pr_count" -eq 0 ]]; then
        log_warning "No open PRs found"
        exit 0
    fi

    # Categorize PRs
    log_info "Categorizing PRs by failure type..."
    local categories
    categories=$(categorize_prs)

    echo ""
    log_info "=== PR Categorization ==="
    echo "$categories" | while IFS=: read -r category prs; do
        if [[ "$prs" != "none" ]]; then
            echo -e "${BLUE}$category${NC}: $prs"
        fi
    done
    echo ""

    # Show details for each PR
    log_info "=== PR Details ==="
    get_open_prs | jq -r '.[] | .number' | while read -r pr_number; do
        echo ""
        get_pr_details "$pr_number"

        # Show failing checks
        echo "Failing checks:"
        gh pr checks "$pr_number" --watch=false 2>&1 | grep "fail" | head -5 || echo "  None passing"
    done

    # Summary
    echo ""
    log_info "=== Summary ==="
    echo "$categories" | while IFS=: read -r category prs; do
        if [[ "$prs" != "none" ]]; then
            local count
            count=$(echo "$prs" | wc -w)
            echo -e "${GREEN}$category${NC}: $count PR(s)"
        fi
    done

    if [[ "$DRY_RUN" == true ]]; then
        echo ""
        log_warning "DRY RUN MODE - No changes were made"
        log_info "To execute, run without --dry-run flag"
    else
        echo ""
        log_info "Use Task tool to launch sub-agents for parallel fixing"
        log_info "See .claude/skills/pr-swarmit.md for sub-agent templates"
    fi
}

# Run main function
main "$@"
