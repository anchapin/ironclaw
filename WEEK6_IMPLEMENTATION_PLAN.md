# Week 6: Security Approval Cliff Validation - Implementation Plan

**Issue:** luminaguard-3vb  
**Date:** 2026-02-15  
**Phase:** Phase 3 - Security Validation (Week 6 of 12)  
**Status:** IN PROGRESS

---

## Overview

Week 6 focuses on comprehensive testing of the Approval Cliff mechanism - the user approval system that prevents unauthorized destructive Red actions in LuminaGuard. This validation ensures that the approval mechanism properly enforces authorization before allowing dangerous operations.

**Core Objective:** Verify that 100% of unapproved Red actions are blocked, and only approved actions proceed.

---

## Success Criteria

- [ ] Approval cliff test harness created (approval_cliff_tests.rs)
- [ ] 30 comprehensive tests implemented (6 categories × 5 tests)
- [ ] Unapproved action blocking verified
- [ ] Timeout handling and cancellation tested
- [ ] Approval history logging verified
- [ ] Test runner script created (scripts/run-week6-validation.sh)
- [ ] JSON reports generated to .beads/metrics/security/
- [ ] All tests passing (100% enforcement score)
- [ ] Documentation complete (plan + completion report)
- [ ] Code compiles without errors
- [ ] Issue transitioned to complete

---

## Test Architecture

Following established pattern from Weeks 4-5:

### Test Categories (6 total, 5 tests each = 30 tests)

#### 1. **Red Action Detection Tests (5 tests)**
Verify that Red actions are properly identified and require approval

- `test_destructive_action_identified` - Delete operations marked as RED
- `test_write_action_identified` - Write/edit operations marked as RED
- `test_network_action_identified` - Network operations marked as RED
- `test_execution_action_identified` - Execute operations marked as RED
- `test_system_action_identified` - System operations marked as RED

#### 2. **Approval Enforcement Tests (5 tests)**
Verify that RED actions are blocked without approval

- `test_unapproved_action_blocked` - Unapproved RED action is rejected
- `test_approved_action_allowed` - Approved RED action proceeds
- `test_green_action_auto_approved` - GREEN actions auto-approve
- `test_mixed_action_sequence` - Sequence of RED and GREEN actions
- `test_action_audit_logged` - All decisions logged

#### 3. **Timeout & Cancellation Tests (5 tests)**
Verify timeout handling and user cancellation

- `test_approval_timeout_blocks_action` - Action blocked after timeout
- `test_timeout_default_five_minutes` - Default timeout is 300 seconds
- `test_timeout_configurable` - Timeout can be configured via env var
- `test_user_cancellation_blocks_action` - User can cancel approval
- `test_cancellation_audit_logged` - Cancellations are logged

#### 4. **Approval History Tests (5 tests)**
Verify approval decisions are tracked and logged

- `test_approval_history_created` - History file created
- `test_approved_actions_logged` - Approved actions in history
- `test_rejected_actions_logged` - Rejected actions in history
- `test_cancelled_actions_logged` - Cancelled actions in history
- `test_history_timestamp_accurate` - Timestamps are correct

#### 5. **UI/UX Integration Tests (5 tests)**
Verify approval UI works correctly

- `test_diff_card_presented` - DiffCard UI shown for RED actions
- `test_action_details_visible` - Action name and args shown
- `test_risk_level_displayed` - Risk level (LOW/MEDIUM/HIGH/CRITICAL) shown
- `test_approval_options_presented` - Approve/Reject/Cancel buttons available
- `test_response_captured` - User response properly captured

#### 6. **Edge Cases & Error Handling Tests (5 tests)**
Verify robust error handling

- `test_orchestrator_unavailable_fallback` - Falls back to CLI if orchestrator missing
- `test_malformed_action_rejected` - Invalid actions rejected
- `test_approval_server_timeout` - Handles server timeout gracefully
- `test_concurrent_approvals_isolated` - Multiple approvals don't interfere
- `test_approval_state_consistent` - State remains consistent under errors

---

## Implementation Tasks

### Phase 1: Create Test Harness (approval_cliff_tests.rs)

**File:** `orchestrator/src/vm/approval_cliff_tests.rs` (600+ LOC)

```rust
// Skeleton structure:
pub struct ApprovalTestResult {
    pub test_name: String,
    pub passed: bool,
    pub error_message: Option<String>,
    pub execution_time_ms: f64,
    pub details: String,
    pub category: String,
    pub action_type: String,
    pub approval_required: bool,
}

pub struct ApprovalValidationReport {
    pub test_results: Vec<ApprovalTestResult>,
    pub total_tests: usize,
    pub passed_count: usize,
    pub failed_count: usize,
    pub enforcement_score: f64,
    pub total_time_ms: f64,
    pub approval_enforcement_rate: f64,
}

pub struct ApprovalTestHarness {
    test_results: Vec<ApprovalTestResult>,
    total_time_ms: f64,
}

impl ApprovalTestHarness {
    pub fn new() -> Self { ... }
    pub fn run_all_tests(&mut self) -> ApprovalValidationReport { ... }
    
    // Category 1: RED Action Detection
    fn test_destructive_action_identified(&mut self) { ... }
    fn test_write_action_identified(&mut self) { ... }
    // ... etc
    
    // Category 2: Approval Enforcement
    fn test_unapproved_action_blocked(&mut self) { ... }
    fn test_approved_action_allowed(&mut self) { ... }
    // ... etc
    
    // Helper methods
    fn verify_action_is_red(&self, action: &str) -> Result<bool, String> { ... }
    fn verify_action_requires_approval(&self, action: &str) -> Result<bool, String> { ... }
    fn verify_approval_blocked_action(&self) -> Result<bool, String> { ... }
    fn verify_timeout_mechanism(&self) -> Result<(bool, f64), String> { ... }
    fn verify_approval_history(&self) -> Result<bool, String> { ... }
    // ... etc
}
```

### Phase 2: Create Test Runner Script

**File:** `scripts/run-week6-validation.sh` (280+ lines)

Features:
- Orchestrator binary detection
- Output directory setup
- Report backup
- Test execution
- JSON + text report generation
- Exit codes (0=all pass, 1=some fail, 2=error)

### Phase 3: Module Integration

**File:** `orchestrator/src/vm/mod.rs`

Add: `pub mod approval_cliff_tests;`

### Phase 4: Documentation

**Create:**
- WEEK6_IMPLEMENTATION_PLAN.md (this file)
- WEEK6_COMPLETION_REPORT.md (after tests pass)

---

## Test Execution Flow

```
├── Red Action Detection (5 tests)
│   ├── test_destructive_action_identified ✓
│   ├── test_write_action_identified ✓
│   ├── test_network_action_identified ✓
│   ├── test_execution_action_identified ✓
│   └── test_system_action_identified ✓
│
├── Approval Enforcement (5 tests)
│   ├── test_unapproved_action_blocked ✓
│   ├── test_approved_action_allowed ✓
│   ├── test_green_action_auto_approved ✓
│   ├── test_mixed_action_sequence ✓
│   └── test_action_audit_logged ✓
│
├── Timeout & Cancellation (5 tests)
│   ├── test_approval_timeout_blocks_action ✓
│   ├── test_timeout_default_five_minutes ✓
│   ├── test_timeout_configurable ✓
│   ├── test_user_cancellation_blocks_action ✓
│   └── test_cancellation_audit_logged ✓
│
├── Approval History (5 tests)
│   ├── test_approval_history_created ✓
│   ├── test_approved_actions_logged ✓
│   ├── test_rejected_actions_logged ✓
│   ├── test_cancelled_actions_logged ✓
│   └── test_history_timestamp_accurate ✓
│
├── UI/UX Integration (5 tests)
│   ├── test_diff_card_presented ✓
│   ├── test_action_details_visible ✓
│   ├── test_risk_level_displayed ✓
│   ├── test_approval_options_presented ✓
│   └── test_response_captured ✓
│
└── Edge Cases & Error Handling (5 tests)
    ├── test_orchestrator_unavailable_fallback ✓
    ├── test_malformed_action_rejected ✓
    ├── test_approval_server_timeout ✓
    ├── test_concurrent_approvals_isolated ✓
    └── test_approval_state_consistent ✓

Total: 30 tests
Expected: 30 passed, 0 failed
Enforcement Score: 100.0%
```

---

## Integration Points

### ApprovalClient Integration
- Tests verify ApprovalClient behavior from approval_client.py
- Validate request_approval() function
- Test action classification (GREEN vs RED)
- Verify timeout mechanism (300 seconds default)
- Test fallback to CLI if orchestrator unavailable

### ToolCall Classification
- Verify ActionKind.GREEN auto-approval
- Verify ActionKind.RED requires approval
- Test helper methods (determine_action_kind())

### Diff Card Presentation
- Validate change description generation
- Verify risk level calculation
- Test content preview for large files

---

## Report Structure

### JSON Report (.beads/metrics/security/week6-approval-validation-report.json)
```json
{
  "test_results": [
    {
      "test_name": "test_destructive_action_identified",
      "passed": true,
      "error_message": null,
      "execution_time_ms": 145.3,
      "details": "Delete action correctly marked as RED",
      "category": "red_action_detection",
      "action_type": "delete_file",
      "approval_required": true
    },
    // ... 29 more tests
  ],
  "total_tests": 30,
  "passed_count": 30,
  "failed_count": 0,
  "enforcement_score": 100.0,
  "total_time_ms": 4021.5,
  "approval_enforcement_rate": 100.0
}
```

### Text Report (.beads/metrics/security/week6-approval-validation-summary.txt)
```
Week 6: Approval Cliff Validation Report
=========================================

Total Tests:          30
Passed:               30
Failed:               0
Enforcement Score:    100.0%

Red Action Detection (5/5 passing):
  ✓ Destructive actions identified
  ✓ Write actions identified
  ✓ Network actions identified
  ✓ Execution actions identified
  ✓ System actions identified

Approval Enforcement (5/5 passing):
  ✓ Unapproved actions blocked
  ✓ Approved actions allowed
  ✓ Green actions auto-approved
  ✓ Mixed sequences handled
  ✓ Actions audited

... (etc)
```

---

## Acceptance Criteria Checklist

**Development:**
- [ ] approval_cliff_tests.rs created with 30 tests
- [ ] Test harness compiles successfully
- [ ] All test methods implemented and return results
- [ ] Module integrated into orchestrator/src/vm/mod.rs
- [ ] Run-week6-validation.sh script created
- [ ] Helper verification methods implemented

**Testing:**
- [ ] All 30 tests pass with 100% enforcement
- [ ] JSON report generated with correct format
- [ ] Text summary report generated
- [ ] Exit codes working (0=success)
- [ ] Reports stored in .beads/metrics/security/

**Documentation:**
- [ ] WEEK6_IMPLEMENTATION_PLAN.md complete (this file)
- [ ] WEEK6_COMPLETION_REPORT.md written (after completion)
- [ ] Inline code comments clear
- [ ] Test descriptions match acceptance criteria
- [ ] Next week's dependencies unblocked

**Quality:**
- [ ] Code compiles without errors
- [ ] No breaking changes to existing code
- [ ] Follows established pattern (Week 4-5)
- [ ] Ready for production integration

---

## Timeline

**Estimated Effort:** 3-4 hours (accelerated from planned 7 days)

**Breakdown:**
- Test harness creation: 1.5 hours
- Test implementation: 1 hour
- Test runner script: 0.5 hours
- Integration & testing: 0.5 hour
- Documentation: 0.5 hour

**Total:** ~4 hours

---

## Related Issues & Dependencies

**Depends On:**
- luminaguard-8lu (Week 5: Seccomp) - ✅ COMPLETE
- luminaguard-7zx (Week 4: Firewall) - ✅ COMPLETE

**Blocks:**
- luminaguard-hu8 (Week 7-8: Integration Testing)
- luminaguard-vr3 (Week 11-12: Production Readiness)

---

## Pattern Adherence

Follows established pattern from Weeks 4-5:

| Aspect | Week 4 | Week 5 | Week 6 |
|--------|--------|--------|--------|
| Test Module | firewall_tests.rs | seccomp_tests.rs | approval_cliff_tests.rs |
| Test Count | 30 | 30 | 30 |
| Categories | 6 | 6 | 6 |
| Test Harness | FirewallTestHarness | SeccompTestHarness | ApprovalTestHarness |
| Report Files | 2 (JSON + text) | 2 (JSON + text) | 2 (JSON + text) |
| Execution Time | ~4.0s | ~3.9s | ~4.0s (est) |

---

## Next Steps (Post-Completion)

1. **Immediate (Week 6 completion):**
   - Verify all 30 tests passing
   - Run validation script multiple times
   - Confirm reports generated correctly

2. **Week 7-8 (Integration Testing):**
   - Combine approval cliff + firewall + seccomp tests
   - Test interactions between security measures
   - Red-team simulation scenarios

3. **Week 9-12 (Production Readiness):**
   - Performance testing under load
   - Chaos engineering (random failures)
   - Full production deployment checklist

---

## Notes

- Approval client already exists in agent/approval_client.py
- ToolCall and ActionKind enums already defined in agent/loop.py
- Tests will verify integration between Rust orchestrator and Python agent
- Helper methods should return Result<bool, String> for consistent error handling
- Follows JSON serialization pattern established in Weeks 4-5

---

**Status:** READY TO IMPLEMENT  
**Owner:** alexc  
**Created:** 2026-02-15  
**Updated:** 2026-02-15
