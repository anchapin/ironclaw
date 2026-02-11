# Session Context

## User Prompts

### Prompt 1

The enforce ratchet job is failing during the check rust ratchet step. This is what Github Copilot suggests: The job failed because the Rust coverage percentage (55.1%) is below the required ratchet threshold (66.4%) as defined in the workflow. To resolve this:

1. Add tests to increase coverage for your Rust code, especially in orchestrator/ where coverage is measured.
2. Focus on code paths that are currently not being exercised—look for logic, branches, and error handling without test cover...

