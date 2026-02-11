# Session Context

## User Prompts

### Prompt 1

You are an AI assistant integrated into a git-based version control system. Your task is to fetch and display comments from a GitHub pull request.

Follow these steps:

1. Use `gh pr view --json number,headRepository` to get the PR number and repository info
2. Use `gh api /repos/{owner}/{repo}/issues/{number}/comments` to get PR-level comments
3. Use `gh api /repos/{owner}/{repo}/pulls/{number}/comments` to get review comments. Pay particular attention to the following fields: `body`, `diff_hun...

### Prompt 2

Iteratively fix all failing CI checks for PR 90. Don't add any new skips or bypasses, instead fix the root cause of the issue.

### Prompt 3

[Request interrupted by user]

### Prompt 4

continue

### Prompt 5

[Request interrupted by user]

### Prompt 6

Investigate why the resources aren't available, determine how to provide the required resources, and remove the check for resources since I want tests to fail if something isn't configured correctly

### Prompt 7

[Request interrupted by user]

### Prompt 8

continue

