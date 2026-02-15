# LuminaGuard Phase 2.2+ Quick Reference Card

## Current Status
```
✅ Phase 2.1 TUI Framework - COMPLETE
✅ Phase 2.2 TUI Color Rendering - COMPLETE & READY FOR PR
🚀 Parallel Workstreams - 6 BRANCHES READY TO START
```

## Active Branches (Pick One to Continue)

```bash
# ✅ COMPLETE - Ready for PR (first choice)
git checkout feature/202-tui-diffcard-rendering

# 🚀 LLM INTEGRATION - Highest impact (40h)
git checkout feature/193-llm-agent-reasoning
# Work in: agent/loop.py
# Test: cd agent && python -m pytest tests/ -v

# 🚀 VM POOL TRACKING - Quick win (8h)
git checkout feature/197-vm-pool-tracking
# Work in: orchestrator/src/vm/pool.rs
# Test: cd orchestrator && cargo test --lib vm::pool

# 🚀 HTTP TRANSPORT - Quick win (8h)  
git checkout feature/196-http-transport-tests
# Work in: orchestrator/src/mcp/http_transport.rs
# Test: cd orchestrator && cargo test --lib mcp::http_transport

# 🚀 ROOTFS SECURITY - Critical for prod (40h)
git checkout feature/202-rootfs-security
# Work in: orchestrator/src/vm/
# Test: cd orchestrator && cargo test --lib vm::

# 🚀 APPLE HV - macOS support (50h)
git checkout feature/199-apple-hv-impl
# Work in: orchestrator/src/vm/apple_hv.rs (NEW)
# Test: cd orchestrator && cargo test --lib vm:: (macOS only)
```

## Standard Workflow

```bash
# 1. Start work on a branch
git checkout feature/XXX-name
bd update luminaguard-ID --status in_progress

# 2. Make changes
# ... edit files ...
git add .
git commit -m "feat: describe change"

# 3. Verify quality (REQUIRED BEFORE PUSH)
cd orchestrator && cargo fmt
cd orchestrator && cargo clippy -- -D warnings
cd orchestrator && cargo test --lib
# Or for Python:
cd agent && black .
cd agent && flake8 .
cd agent && python -m pytest tests/ -v

# 4. Push and create PR
git push origin feature/XXX-name
gh pr create --title "Implement XXX (Phase 2.2)" --body "Closes #NNN"
bd update luminaguard-ID --status closed

# 5. Sync beads
bd sync

# 6. Celebrate! 🎉
```

## Testing Checklists

### Rust (Orchestrator)
```bash
cd orchestrator

# Full test suite
cargo test --lib

# Specific module
cargo test --lib vm::pool::tests
cargo test --lib approval::tui::tests
cargo test --lib mcp::http_transport::tests

# With output
cargo test --lib -- --nocapture

# Benchmarks (optional)
cargo bench
```

### Python (Agent)
```bash
cd agent

# Activate venv
source .venv/bin/activate

# Run tests
python -m pytest tests/ -v

# With coverage
python -m pytest tests/ --cov=agent --cov-report=term-missing

# Specific test
python -m pytest tests/test_loop.py::test_think -v
```

## Code Quality Gates

### Before Every Push
```bash
# 1. Format
cd orchestrator && cargo fmt
cd agent && black agent/ && black tests/

# 2. Lint
cd orchestrator && cargo clippy -- -D warnings
cd agent && flake8 agent/ tests/

# 3. Test
cd orchestrator && cargo test --lib
cd agent && python -m pytest tests/ -v

# 4. Coverage (must maintain ≥75%)
cd agent && python -m pytest tests/ --cov --cov-report=term-missing
```

## Key Files

### Phase 2.2 TUI (DONE)
```
orchestrator/src/approval/tui.rs         ⭐ MODIFIED (463 lines, color-coded)
orchestrator/src/approval/diff.rs        ✅ INTEGRATED
orchestrator/src/approval/action.rs      ✅ INTEGRATED
orchestrator/src/approval/mod.rs         ✅ INTEGRATED
```

### Ready for Next Work
```
agent/loop.py                             ← LLM Integration entry point
orchestrator/src/vm/pool.rs               ← VM Pool metrics
orchestrator/src/mcp/http_transport.rs    ← HTTP test fixes
orchestrator/src/vm/                      ← Rootfs security
orchestrator/src/vm/apple_hv.rs           ← Apple HV (new file)
```

## Documentation Map

| Document | Purpose | Size |
|----------|---------|------|
| `SESSION_SUMMARY_2026_02_14.md` | What was done, what's next | 360 lines |
| `PARALLEL_WORKSTREAM_STATUS.md` | Status + quick-start guide | 399 lines |
| `PHASE_2_2_PARALLEL_IMPLEMENTATION.md` | Detailed roadmap + architecture | 459 lines |
| `QUICK_REFERENCE.md` | This file - commands & workflow | - |

## Effort Estimates

| Task | Hours | Status | Branch |
|------|-------|--------|--------|
| Phase 2.2 TUI | 10h | ✅ DONE | feature/202-tui-diffcard-rendering |
| LLM Integration | 40h | 🚀 READY | feature/193-llm-agent-reasoning |
| VM Pool Tracking | 8h | 🚀 READY | feature/197-vm-pool-tracking |
| HTTP Transport | 8h | 🚀 READY | feature/196-http-transport-tests |
| Rootfs Security | 40h | 🚀 READY | feature/202-rootfs-security |
| Apple HV | 50h | 🚀 READY | feature/199-apple-hv-impl |
| **TOTAL** | **156h** | **In Progress** | |

## Metrics (Current)

| Metric | Value | Target | Status |
|--------|-------|--------|--------|
| Test Coverage | 76% | 75%+ | ✅ |
| Tests Passing | 245/245 | 100% | ✅ |
| Clippy Warnings | 0 | 0 | ✅ |
| TUI FPS | 60+ | 60+ | ✅ |

## GitHub & Beads Commands

```bash
# Create GitHub issue
gh issue create --title "Task name" --body "Details"

# Create feature branch
git checkout -b feature/NNN-description main

# Create PR
gh pr create --title "Description" --body "Closes #NNN"

# Link to beads issue
bd create --title "Task name" --body "Description"
bd update luminaguard-ID --status in_progress
bd update luminaguard-ID --status closed
bd sync

# List ready issues
bd ready
```

## Emergency Commands

```bash
# Undo last commit (before push)
git reset --soft HEAD~1

# Undo all uncommitted changes
git checkout .

# Switch back to main
git checkout main

# Delete a branch
git branch -D feature/xxx

# Check test status
cargo test --lib 2>&1 | tail -20
```

## Performance Targets

| Component | Target | Achieved | Notes |
|-----------|--------|----------|-------|
| TUI Render | 60+ FPS | ✅ | Per-frame render <20ms |
| VM Spawn | <200ms | ~110ms | Firecracker direct |
| Startup | <500ms | <50ms | `present_tui_approval()` |
| HTTP Latency | <100ms | TBD | Phase 2.2 HTTP transport |
| LLM Reasoning | <5s | TBD | Phase 2.2 LLM integration |

## Critical Files DON'T Edit

```
.git/                      ← Git internals
.beads/                    ← Task database
orchestrator/Cargo.lock    ← Auto-generated
.worktrees/                ← Worktree copies
target/                    ← Build artifacts
node_modules/              ← Dependencies
.venv/                     ← Python venv
```

## Success Criteria (Session End)

- [ ] All tests passing (`cargo test --lib`)
- [ ] Coverage maintained (≥75%)
- [ ] Zero clippy warnings
- [ ] PR created with description
- [ ] Beads issue updated
- [ ] Feature branch pushed
- [ ] No merge conflicts expected

---

**Last Updated**: 2026-02-14
**Branches**: 6 ready
**Tests**: 245 passing
**Coverage**: 76%
**Ready to Ship**: Phase 2.2 TUI (feature/202-tui-diffcard-rendering)
