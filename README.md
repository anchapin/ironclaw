# IronClaw

Local-first Agentic AI Runtime with Just-in-Time Micro-VMs.

**Vision**: Replace the insecure "vibe coding" paradigm with rigorous "Agentic Engineering" - combining OpenClaw's usability with Nanoclaw's security.

## Quick Start

```bash
# Install development dependencies
make install

# Run all tests
make test

# Format all code
make fmt

# Start development
make dev
```

## Project Structure

```
ironclaw/
├── orchestrator/    # Rust Orchestrator (CLI, VM spawning, MCP client)
├── agent/          # Python Reasoning Loop (forked from Nanobot)
├── docs/           # Documentation
├── scripts/        # Development tooling
└── .quint/         # FPF reasoning context
```

## Architecture

IronClaw follows a "Rust Wrapper, Python Brain" design:

- **Orchestrator (Rust)**: Lightweight binary handling CLI, memory management, and JIT Micro-VM spawning
- **Agent Logic (Python)**: The reasoning loop forked from Nanobot core, kept under 4,000 lines for auditability

## Development

### Prerequisites

- Rust 1.75+ (`curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh`)
- Python 3.11+ (system Python or pyenv)
- Git (for pre-commit hooks)

### TDD Workflow

1. Write failing test first (Red)
2. Implement minimal code to pass (Green)
3. Refactor while keeping tests green (Refactor)
4. Commit only when all tests pass

### Quality Gates

- **Python LOC Limit**: `loop.py` must remain under 4,000 lines (enforced by CI)
- **Pre-commit Hooks**: Automatic formatting (black, rustfmt) before commits
- **Security Tests**: All security invariants must pass before merges

## Roadmap

### Phase 1 - Foundation (Current)
- Fork Nanobot for Python reasoning loop
- Build Rust Orchestrator for MCP connections
- Implement basic "Green Action" autonomy

### Phase 2 - Security
- Integrate Firecracker for JIT Micro-VM spawning
- Implement Approval Cliff UI for file system writes
- Beta release to security-conscious developers

### Phase 3 - Advanced Features
- Private Mesh protocol for multi-agent collaboration
- Desktop GUI (Rust-based, NOT Electron)

## License

MIT License - see LICENSE file for details

## Contributing

1. Fork the repository
2. Create a feature branch
3. Write tests for your changes
4. Ensure all tests pass (`make test`)
5. Submit a pull request

**Note**: All contributions must comply with the "Agentic Engineering" principles - intentional, auditable, and minimal code.

## Safety

IronClaw is designed with security as the primary concern:
- **Zero Host Execution**: Agents never execute directly on host OS
- **Approval Cliff**: High-stakes actions require explicit human approval
- **Ephemeral VMs**: Micro-VMs are destroyed after task completion

See [CLAUDE.md](CLAUDE.md) for complete architecture principles.
