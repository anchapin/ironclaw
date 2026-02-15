# Phase 2.2+ Parallel Implementation Plan

**Status**: Starting parallel workstreams (2026-02-14)
**Total Team Capacity**: 6 parallel feature branches
**Expected Completion**: 10-12 weeks

## Overview

After completing Phase 2.1 (TUI Framework), we now parallelize the remaining work to unblock progress across multiple domains:

1. **Phase 2.2 TUI Enhancement** (10h) - Makes TUI visually useful
2. **LLM Agent Integration** (40h) - Intelligent reasoning layer
3. **VM Pool & Tracking** (8h) - Resource monitoring
4. **HTTP Transport** (8h) - Remote MCP support
5. **Security Hardening** (40h) - Rootfs + capabilities
6. **Apple HV** (50h) - macOS VM support

---

## Parallel Workstream 1: Phase 2.2 TUI DiffCard Rendering (10h)

**Branch**: `feature/202-tui-diffcard-rendering`
**Depends**: Phase 2.1 ✅ (complete)
**Blocks**: Phase 2.3+ (UI polish)

### Goals
- [x] Color-code changes by risk level (🟢→🔴🔴)
- [ ] Render scrollbar indicator
- [ ] Text wrapping for long lines
- [ ] Optional syntax highlighting for file paths

### Implementation Steps

#### Step 1: Risk Level Color Styling (2h)
Update `render_content()` in `tui.rs` to apply color to each change:

```rust
enum ChangeColor {
    Green,   // FileCreate, Low risk
    Yellow,  // FileEdit, Medium risk
    Red,     // FileDelete, High risk
    Magenta, // CommandExec, Critical
}

// Map changes to colors based on ActionType.risk_level()
// Apply via ratatui Style::default().fg(Color::Green)
```

#### Step 2: Scrollbar Rendering (2h)
Add visual scrollbar indicator to content area:

```rust
fn render_scrollbar(area: Rect, scroll_offset: u16, total_lines: u16) -> Line {
    let track_height = area.height;
    let scroll_pos = (scroll_offset as f64 / total_lines.max(1) as f64) * track_height as f64;
    // Unicode box drawing chars: ▐ ▌ █
}
```

#### Step 3: Text Wrapping (2h)
Wrap long lines to terminal width:

```rust
fn wrap_text(text: &str, width: usize) -> Vec<String> {
    // Break long lines intelligently at word boundaries
    // Handle ANSI color codes (don't count in width)
}
```

#### Step 4: Syntax Highlighting (2h)
Highlight file paths and commands:

```rust
fn highlight_line(line: &str) -> Span {
    // Detect patterns: /path/to/file → cyan
    // Detect patterns: command args → magenta
    // Apply via ratatui Span styling
}
```

#### Step 5: Testing & Polish (2h)
- Unit tests for color mapping
- Integration tests with DiffCard rendering
- Terminal width responsiveness tests

### Success Criteria
- [ ] All changes render with appropriate colors
- [ ] Scrollbar appears and tracks position correctly
- [ ] Lines wrap without truncation (except at terminal edge)
- [ ] Tests cover 85%+ of TUI rendering
- [ ] Compiles with -D warnings (zero clippy warnings)

---

## Parallel Workstream 2: LLM Agent Integration (#193) (40h)

**Branch**: `feature/193-llm-agent-reasoning`
**Depends**: Phase 2.1 ✅
**Blocks**: Phase 3 validation

### Goals
- Replace keyword-based reasoning in `agent/loop.py`
- Integrate with LLM (Claude, GPT-4, etc.)
- Use `ActionType.requires_approval()` to classify actions
- Make `ApprovalManager` the security gate for all destructive actions

### Implementation Steps

#### Step 1: LLM Integration Framework (8h)
- Add LLM client abstraction (support Claude API, OpenAI, etc.)
- Implement prompt engineering for action classification
- Cache action classification decisions

#### Step 2: Agent Loop Refactor (10h)
- Replace keyword-based `think()` function
- Implement LLM-based reasoning loop
- Add tool selection via LLM

#### Step 3: Approval Cliff Integration (10h)
- Call `ActionType.requires_approval()` before tool execution
- Route Red actions through `ApprovalManager`
- Record all decisions in audit trail

#### Step 4: Testing & Validation (8h)
- Unit tests with mocked LLM
- Integration tests with real approval workflow
- Property-based tests for safety

#### Step 5: Documentation (4h)
- LLM configuration guide
- Prompt engineering patterns
- API cost estimation

### Success Criteria
- [ ] LLM replaces keyword-based reasoning
- [ ] Green actions auto-execute (fast path)
- [ ] Red actions require approval (slow path)
- [ ] Audit trail captures all decisions
- [ ] 80%+ test coverage

---

## Parallel Workstream 3: VM Pool Tracking (#197) (8h)

**Branch**: `feature/197-vm-pool-tracking`
**Depends**: Existing VM pool ✅
**Blocks**: Production monitoring

### Goals
- Track active VM instances
- Monitor task queue status
- Implement pool metrics API

### Implementation
```rust
pub struct PoolMetrics {
    pub active_vms: usize,
    pub queued_tasks: usize,
    pub avg_spawn_time_ms: f64,
    pub snapshot_freshness: Duration,
}

pub async fn get_pool_metrics() -> Result<PoolMetrics>;
```

### Success Criteria
- [ ] Metrics endpoint implemented
- [ ] Integration with existing pool
- [ ] HTTP API for remote monitoring
- [ ] Tests with synthetic load

---

## Parallel Workstream 4: HTTP Transport Integration Tests (#196) (8h)

**Branch**: `feature/196-http-transport-tests`
**Depends**: HTTP transport ✅ (from Phase 2)
**Blocks**: HTTP transport rollout

### Goals
- Fix test timeouts in HTTP MCP transport
- Improve test isolation
- Add race condition detection

### Implementation
```rust
#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn test_http_transport_with_proper_timeout() {
    // Use distinct ports per test
    // Proper server cleanup
    // Timeout detection
}
```

### Success Criteria
- [ ] All HTTP transport tests pass consistently
- [ ] No timeout flakiness
- [ ] <5s test runtime for full suite

---

## Parallel Workstream 5: Rootfs Security Hardening (#202) (40h)

**Branch**: `feature/202-rootfs-security`
**Depends**: Existing VM security ✅
**Blocks**: Production deployment

### Goals
- Drop unnecessary capabilities (cap_kill, cap_sys_admin, etc.)
- Enable SELinux in VMs
- Implement AppArmor profiles
- Enforce resource limits

### Implementation

#### Step 1: Capability Dropping (12h)
```rust
const REQUIRED_CAPS: &[Capability] = &[
    Capability::cap_chown,
    Capability::cap_setfcap,
    Capability::cap_sys_chroot,
];

fn drop_unnecessary_caps(vm_config: &mut VmConfig) {
    vm_config.dropped_capabilities = ALL_CAPABILITIES
        .iter()
        .filter(|c| !REQUIRED_CAPS.contains(c))
        .copied()
        .collect();
}
```

#### Step 2: SELinux Integration (12h)
- Boot VMs with SELinux enabled
- Define minimal security contexts
- Test enforcement mode

#### Step 3: AppArmor Profiles (8h)
- Create profiles for common operations
- Enforce on VM processes
- Test rule effectiveness

#### Step 4: Resource Limits (8h)
- CPU, memory, I/O limits via cgroups
- File descriptor limits
- Process limits

### Success Criteria
- [ ] Capabilities dropped to minimum needed
- [ ] SELinux boots in enforcing mode
- [ ] AppArmor profiles active
- [ ] Resource limits enforced
- [ ] Security tests pass

---

## Parallel Workstream 6: Apple HV (macOS) Implementation (#199) (50h)

**Branch**: `feature/199-apple-hv-impl`
**Depends**: Hypervisor trait ✅, VM config ✅
**Blocks**: macOS support

### Goals
- Implement `Hypervisor` trait for Apple Virtualization.framework
- VM lifecycle management on macOS
- Integration with existing pool

### Implementation

#### Step 1: vz Crate Integration (15h)
- Wrap `vz` crate (Virtualization.framework bindings)
- Implement VM config → vz config translation
- Handle macOS-specific features (Rosetta, GPU passthrough)

#### Step 2: VM Lifecycle (15h)
```rust
pub struct AppleHvInstance {
    vm: vz::VirtualMachine,
    id: String,
    socket: String,
}

impl Hypervisor for AppleHvImpl {
    async fn spawn(&self, config: &VmConfig) -> Result<Box<dyn VmInstance>>;
}

impl VmInstance for AppleHvInstance {
    async fn stop(&mut self) -> Result<()>;
}
```

#### Step 3: Snapshot Integration (15h)
- Snapshot pool support for macOS
- Shared memory optimization
- Resource cleanup

#### Step 4: Testing (5h)
- Unit tests (mocked vz)
- Integration tests (macOS only)
- Performance benchmarks

### Success Criteria
- [ ] `Hypervisor` trait implemented for macOS
- [ ] VMs spawn in <500ms
- [ ] Snapshot pool working
- [ ] CI/CD tests pass on macOS

---

## Resource Allocation

### Timeline (Sequential → Parallel)

```
Week 1-2:   Phase 2.2 TUI (baseline work) + VM Pool & HTTP Tests (parallel setup)
Week 3-4:   TUI complete + LLM framework (Phase 1) + Security hardening (Phase 1)
Week 5-6:   LLM integration complete + Rootfs hardening (Phase 2)
Week 7-10:  Apple HV implementation (longest path)
Week 10-12: Integration & validation

Critical Path: Apple HV (50h) → most time, can start after Phase 2.1
Fastest Path: HTTP Transport Tests (8h) → can finish in parallel
```

### Effort Estimates

| Workstream | Effort | Dependencies | Ready Now? |
|----------|--------|--------------|------------|
| Phase 2.2 TUI | 10h | Phase 2.1 ✅ | Yes |
| LLM Integration | 40h | Phase 2.1 ✅ | Yes |
| VM Pool Tracking | 8h | Current VM ✅ | Yes |
| HTTP Transport Tests | 8h | Current HTTP ✅ | Yes |
| Rootfs Hardening | 40h | Current VM ✅ | Yes |
| Apple HV | 50h | Traits ✅ | Yes |
| **Total** | **156h** | | |

---

## Git Workflow

### Creating Feature Branches

```bash
# Phase 2.2 TUI
git checkout feature/202-tui-diffcard-rendering
bd update luminaguard-esd --status in_progress

# LLM Integration
git checkout -b feature/193-llm-agent-reasoning main
bd update luminaguard-1i4 --status in_progress

# VM Pool Tracking
git checkout -b feature/197-vm-pool-tracking main
bd update luminaguard-sw3 --status in_progress

# HTTP Transport Tests
git checkout -b feature/196-http-transport-tests main
bd update luminaguard-rg7 --status in_progress

# Rootfs Hardening
git checkout -b feature/202-rootfs-security main
bd update luminaguard-c9n --status in_progress

# Apple HV
git checkout -b feature/199-apple-hv main
bd update luminaguard-jnt --status in_progress
```

### Commit Message Convention

```
<type>: <title>

<description>

Related: luminaguard-<id>
Relates-to: #193
```

### Integration Strategy

1. **Phase 2.2 TUI** → PR to main (blocks nothing)
2. **LLM Integration** → PR to main (parallel, no conflicts)
3. **VM Pool** → PR to main (parallel)
4. **HTTP Tests** → PR to main (parallel)
5. **Rootfs Security** → PR to main (may conflict with VM changes, merge last)
6. **Apple HV** → PR to main (macOS only, no conflicts)

---

## Testing & Quality Gates

### Phase 2.2 TUI
- Unit tests: 20+ (color mapping, text wrapping, scrollbar)
- Integration tests: 10+ (render with various DiffCards)
- Target coverage: 85%+ of `tui.rs`

### LLM Integration
- Unit tests: 30+ (LLM call mocking, action classification)
- Integration tests: 15+ (full workflow)
- Safety tests: 10+ (approval cliff enforcement)
- Target coverage: 80%+ of `loop.py`

### VM Pool
- Unit tests: 8+
- Integration tests: 5+ (synthetic load)
- Target coverage: 80%+

### HTTP Transport
- Unit tests: 15+ (timeout, retry, load balancing)
- Integration tests: 10+ (real server interaction)
- Target coverage: 85%+

### Rootfs Hardening
- Security tests: 20+ (capability checks, SELinux)
- Integration tests: 10+ (full VM lifecycle)
- Target coverage: 80%+

### Apple HV
- Unit tests: 20+ (mocked vz)
- Integration tests: 10+ (macOS only)
- Target coverage: 75%+

---

## Success Metrics

### Code Quality
- [x] All branches pass CI/CD
- [x] Coverage maintained at 75%+
- [x] Zero clippy warnings (-D warnings)
- [x] All tests pass

### Performance
- [ ] TUI renders at 60+ FPS
- [ ] LLM reasoning completes in <5s average
- [ ] VM spawn completes in <200ms
- [ ] HTTP transport with <100ms latency

### Security
- [ ] Rootfs hardening reduces attack surface
- [ ] Approval cliff blocks all Red actions
- [ ] Audit trails complete and queryable

---

## Notes for Next Session

1. **Sync with beads before starting**: `bd sync`
2. **Update branch status**: `bd update <id> --status in_progress`
3. **Add notes regularly**: `bd update <id> --note "..."`
4. **On completion**: `bd close <id> --reason "..." && bd sync`
5. **Push regularly**: Don't accumulate commits locally

---

**Created**: 2026-02-14
**Modified**: 2026-02-14
**Status**: READY FOR IMPLEMENTATION
