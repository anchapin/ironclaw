# IronClaw Bounded Context

**Session:** 2025-02-09
**Project:** IronClaw - Local-first Agentic AI Runtime
**Phase:** Planning/Prototype (Pre-Phase 1)

---

## Vocabulary

| Term | Definition |
|------|------------|
| **Agentic Engineering** | Rigorous, intentional approach to AI agent development contrasting with "vibe coding" - emphasizes auditability, minimal dependencies, and deterministic behavior |
| **JIT Micro-VM** | Just-in-Time Micro-Virtual Machine - ephemeral Firecracker-like Linux VM that spawns in <200ms, executes agent tasks, then is destroyed |
| **Approval Cliff** | Security boundary separating autonomous "Green Actions" (read-only) from "Red Actions" (destructive/external) requiring human approval via Diff Card UI |
| **MCP (Model Context Protocol)** | Standard protocol for tool/server connections - IronClaw acts as native MCP client, not proprietary plugin system |
| **Orchestrator** | Lightweight Rust binary handling CLI, memory management, and Micro-VM spawning |
| **Reasoning Loop** | Python-based agent decision-making engine forked from Nanobot core (`loop.py`), kept under 4,000 lines |
| **Private Mesh** | Encrypted local network (e.g., WireGuard) enabling secure multi-agent communication without public internet exposure |
| **Vibe Coding** | Pejorative term for AI-generated codebases without intentional architecture - leads to bloat, unmaintainability, and security vulnerabilities |

---

## Invariants

### Security Invariants (NON-NEGOTIABLE)
1. **Zero Host Execution**: Agents MUST NEVER execute directly on host OS - all code runs inside JIT Micro-VMs
2. **Approval Required**: All "Red Actions" (file writes, deletions, external communications) MUST pass through Approval Cliff with explicit human consent
3. **Ephemeral State**: Micro-VMs MUST be destroyed after task completion - no persistence of agent state or malware between sessions
4. **No CVE-2026-25253**: Must NOT repeat OpenClaw's critical RCE vulnerability - full shell access to host is FORBIDDEN

### Architecture Invariants
5. **Rust Wrapper, Python Brain**: Orchestrator MUST be Rust (memory safety, performance); Agent Logic MUST be Python forked from Nanobot (auditability, <4000 LOC)
6. **Native MCP Only**: MUST NOT implement proprietary "AgentSkills" or custom plugin systems - use standard MCP protocol exclusively
7. **Startup Performance**: New agent sessions MUST initialize in <500ms (target: <200ms VM spawn time)
8. **Memory Efficiency**: Baseline footprint MUST be significantly less than OpenClaw's ~200MB (target: comparable to Nanobot's ~45MB)

### Code Quality Invariants
9. **Auditability**: Python reasoning loop MUST remain under 4,000 lines of code
10. **Determinism**: Codebase MUST be small, intentional, and reviewable - no "vibe coding" or AI-generated bloat
11. **Standardization**: MUST leverage existing protocols (MCP, Firecracker) rather than reinventing systems
12. **Zero Manual Container Management**: Users MUST NEVER write Dockerfiles or manage containers manually - isolation is automatic

### Operational Invariants
13. **Local-First**: Core execution happens locally - no dependency on cloud services for agent runtime
14. **Private Mesh Security**: Multi-agent communication MUST NOT traverse public internet - encrypted local network only
15. **GUI Tech Stack**: Desktop GUI MUST be Rust-based (NOT Electron) - no web technology bloat for native applications

---

## Technical Constraints

### Performance Targets
- VM Spawn Time: <200ms (Firecracker-like technology)
- Session Startup: <500ms total
- Memory Baseline: ~45MB (comparable to Nanobot)
- Startup Comparison: 10x faster than OpenClaw (8s+ → <500ms)

### Technology Stack
- **Orchestrator**: Rust (memory safety, zero-cost abstractions)
- **Agent Logic**: Python 3.x (Nanobot fork)
- **Micro-VM**: Firecracker or similar (KVM-based)
- **Protocol**: MCP (Model Context Protocol) - native client implementation
- **Networking**: WireGuard or similar (Private Mesh)

### Architecture Split
```
┌─────────────────────────────────────┐
│     Rust Orchestrator (< 500ms)     │
│  • CLI Interface                    │
│  • Memory Management                │
│  • VM Lifecycle (spawn/destroy)     │
│  • MCP Client Connections           │
└──────────────┬──────────────────────┘
               │
               ▼
┌─────────────────────────────────────┐
│   Python Reasoning Loop (Nanobot)   │
│  • Agent Decision Making            │
│  • Tool Use Orchestration           │
│  • < 4,000 LOC (auditable)          │
└──────────────┬──────────────────────┘
               │
               ▼
┌─────────────────────────────────────┐
│    JIT Micro-VM (< 200ms spawn)     │
│  • Stripped Linux Kernel            │
│  • Browser/Tools Execution          │
│  • Ephemeral (destroyed on exit)    │
└─────────────────────────────────────┘
```

---

## Competitive Landscape Context

| Product | Security | Usability | IronClaw's Position |
|---------|----------|-----------|---------------------|
| **OpenClaw** | ❌ Host execution (CVE-2026-25253) | ✅ Easy setup | **Replace** - Match usability, fix security |
| **Nanoclaw** | ✅ Docker isolation | ❌ High friction | **Improve** - Keep security, remove friction |
| **Nanobot** | ✅ Minimal, auditable | ⚠️ Requires expertise | **Fork** - Use as Python loop reference |

---

## Roadmap Phase Context

**Current State**: Phase 0 (Planning/Prototype)
**Next Phase**: Phase 1 - Foundation (Months 1-2)
- Fork Nanobot for Python reasoning loop
- Build Rust Orchestrator for MCP connections
- Implement basic "Green Action" autonomy

**Phase 2 Goal**: Security (Months 3-4)
- Integrate Firecracker for JIT Micro-VMs
- Implement Approval Cliff UI
- Beta to security-conscious developers

**Phase 3 Goal**: Advanced Features (Months 5-6)
- Private Mesh protocol
- Rust-based Desktop GUI (NOT Electron)

---

## Success Metrics

1. **Safety**: 0 reported RCEs or container escapes (FOREVER)
2. **Performance**: <500ms startup time for new sessions
3. **Adoption**: 50+ verified community MCP servers working out-of-the-box
4. **Code Quality**: Python loop remains <4,000 LOC
5. **Memory**: Baseline footprint <100MB (target: ~45MB)

---

## Non-Goals (Explicitly Out of Scope)

1. ❌ Proprietary plugin ecosystems (use MCP)
2. ❌ Cloud-based agent execution (local-first)
3. ❌ Electron-based GUI (use Rust native)
4. ❌ Social network features (Moltbook failure - use Private Mesh instead)
5. ❌ Persistent container daemons (use JIT Micro-VMs)
6. ❌ Manual container management (Dockerfiles, docker-compose - all automated)

---

## Decision Log Context

This bounded context will guide all FPF reasoning cycles. When generating hypotheses (Phase 1), verifying logic (Phase 2), or validating (Phase 3), all reasoning MUST respect these invariants as non-negotiable constraints.

**Key Principle**: When invariants conflict, Security Invariants (#1-4) ALWAYS trump Performance or Convenience concerns.
