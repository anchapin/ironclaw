# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

**IronClaw** is a local-first Agentic AI runtime designed to provide secure agent execution through Just-in-Time (JIT) Micro-VMs. The project is in early planning/prototype phase.

**Core Vision:** Replace the insecure "vibe coding" paradigm with rigorous "Agentic Engineering" - combining OpenClaw's usability with Nanoclaw's security.

## Architecture Principles

### The "Rust Wrapper, Python Brain" Design

The codebase follows a split architecture:

1. **Orchestrator (Rust)**: Lightweight binary handling:
   - CLI interface
   - Memory management
   - Micro-VM spawning (target: <200ms startup)
   - MCP client connections

2. **Agent Logic (Python)**: The reasoning loop:
   - Forked from Nanobot core (`loop.py`)
   - Kept under 4,000 lines for auditability
   - Handles agent decision-making and tool use

### Just-in-Time (JIT) Micro-VMs

Instead of persistent containers or host execution, agents run in ephemeral Micro-VMs:

- Spawn: Stripped-down Linux VM in <200ms
- Execute: Browser/tools run inside the VM
- Dispose: VM is destroyed after task completion
- Security: Malware cannot persist (the "infected" computer no longer exists)

### Native MCP Support

IronClaw is a native Model Context Protocol (MCP) client:
- Connects to any standard MCP Server (Google Drive, Slack, GitHub, Postgres, etc.)
- No proprietary "AgentSkills" or custom plugin systems
- Leverages the growing enterprise MCP ecosystem

## Key Security Feature: The "Approval Cliff"

High-stakes actions require explicit human approval:

**Green Actions (Autonomous):**
- Reading files
- Searching the web
- Checking logs
- Read-only operations

**Red Actions (Require Approval):**
- Editing code
- Deleting files
- Sending emails
- Transferring crypto/assets
- Any destructive or external communication

The UI presents a "Diff Card" showing exactly what will change before execution.

## Development Guidelines

### Code Philosophy

1. **Agentic Engineering over Vibe Coding**: Every line must be intentional, reviewed, and necessary
2. **Invisible Security**: Isolation happens automatically - users should never need to write Dockerfiles or manage containers manually
3. **Standardization**: Use existing protocols (MCP) rather than building custom systems
4. **Auditability**: Keep the codebase small and deterministic

### Performance Targets

- Startup time: <500ms for new agent sessions
- Memory footprint: Significantly less than OpenClaw (~200MB baseline)
- VM spawn time: <200ms using Firecracker-like technology

### Safety Requirements

- Zero reported RCEs or container escapes
- All file system writes must go through the Approval Cliff
- Micro-VMs must be truly ephemeral
- No persistence of agent state between VM sessions

## Roadmap Context

### Phase 1 - Foundation (Current/Planned)
- Fork Nanobot for Python reasoning loop
- Build Rust Orchestrator for MCP connections
- Implement basic autonomous ("Green Action") capabilities

### Phase 2 - Security
- Integrate Firecracker for JIT Micro-VM spawning
- Implement Approval Cliff UI for file operations
- Beta release to security-conscious developers

### Phase 3 - Advanced Features
- Private Mesh protocol for multi-agent collaboration
- Desktop GUI (Rust-based, NOT Electron)

## Competitive Context

IronClaw aims to position between:
- **OpenClaw**: Usable but insecure (host execution, CVE-2026-25253)
- **Nanoclaw**: Secure but high friction (manual Docker management)
- **Nanobot**: Minimalist codebase reference for the Python loop

## References

- PRD: `ironclaw_prd.md` - Complete product specification
- Nanobot core: Reference for the Python reasoning loop architecture
- MCP Protocol: Standard for tool/server connections
