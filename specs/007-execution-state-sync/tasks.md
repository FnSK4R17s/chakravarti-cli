# Tasks: Execution State Synchronization Fix

**Feature**: 007-execution-state-sync  
**Branch**: `007-execution-state-sync`  
**Generated**: 2026-01-06

## Overview

This task list implements the fix for execution state synchronization between the Rust backend and React frontend. The fix addresses three issues:
1. Timer not starting (stuck at 0s)
2. Batch completion counter stuck at 0/N
3. Execution never showing as completed

## Task Summary

| Phase | Description | Task Count |
|-------|-------------|------------|
| Phase 1 | Setup | 2 |
| Phase 2 | Foundational - Message Types | 3 |
| Phase 3 | US1 - Timer and Progress | 4 |
| Phase 4 | US2 - Batch Completion | 4 |
| Phase 5 | US3 - Execution Status | 4 |
| Phase 6 | Polish & Validation | 3 |
| **Total** | | **20** |

---

## Phase 1: Setup

**Goal**: Prepare workspace and verify current state

- [x] T001 Checkout branch `007-execution-state-sync` and verify clean state
- [x] T002 Review current message flow in `crates/ckrv-ui/src/services/engine.rs` and `crates/ckrv-ui/frontend/src/components/ExecutionRunner.tsx`

**Checkpoint**: Understanding of current message mismatch confirmed

---

## Phase 2: Foundational - Message Types (BLOCKS ALL USER STORIES)

**Goal**: Define message types and enhance LogMessage struct

- [x] T003 [P] Create WebSocket message type definitions in `crates/ckrv-ui/frontend/src/types/websocket.ts`
- [x] T004 Enhance LogMessage struct with optional `status`, `batch_id`, `batch_name` fields in `crates/ckrv-ui/src/services/engine.rs`
- [x] T005 Add `LogMessage::status()` and `LogMessage::batch_status()` factory methods in `crates/ckrv-ui/src/services/engine.rs`

**Checkpoint**: Enhanced message types available for use

---

## Phase 3: User Story 1 - Timer and Progress Tracking (Priority: P1)

**Goal**: Fix elapsed timer to start when execution begins

**Independent Test**: Start execution, verify timer counts upward from 0s

**Bug Addressed**: Timer stuck at 0s

### Implementation for User Story 1

- [x] T006 [US1] Add `status: "running"` message send after execution starts in `crates/ckrv-ui/src/services/engine.rs` (line ~130)
- [x] T007 [US1] Add fallback handling for `type: "start"` to set running status in `crates/ckrv-ui/frontend/src/components/ExecutionRunner.tsx`
- [x] T008 [US1] Verify timer effect triggers on status change in `crates/ckrv-ui/frontend/src/components/ExecutionRunner.tsx` (lines 496-511)
- [x] T009 [US1] Test timer starts when receiving running status via WebSocket

**Checkpoint**: Timer starts counting when Run Execution is clicked

---

## Phase 4: User Story 2 - Batch Completion Tracking (Priority: P1)

**Goal**: Fix batch completion counter to update in real-time

**Independent Test**: Run execution with multiple batches, verify counter increments (1/14, 2/14, etc.)

**Bug Addressed**: Counter stuck at 0/14

### Implementation for User Story 2

- [x] T010 [US2] Add regex pattern to match `"Batch X completed on branch Y"` format in `crates/ckrv-ui/frontend/src/components/ExecutionRunner.tsx` (updateBatchFromLog function)
- [x] T011 [US2] Add `batch_status` message send on batch start in `crates/ckrv-ui/src/services/engine.rs` (line ~197)
- [x] T012 [US2] Add `batch_status` message send on batch complete in `crates/ckrv-ui/src/services/engine.rs` (line ~235)
- [x] T013 [US2] Add handler for `type: "batch_status"` messages in `crates/ckrv-ui/frontend/src/components/ExecutionRunner.tsx`

**Checkpoint**: Batch counter increments as batches complete

---

## Phase 5: User Story 3 - Execution Status Accuracy (Priority: P1)

**Goal**: Fix execution status to correctly transition to completed/failed

**Independent Test**: Run execution to completion, verify status shows "Completed"

**Bug Addressed**: Execution never shows as complete

### Implementation for User Story 3

- [x] T014 [US3] Add `status: "completed"` message send after all batches complete in `crates/ckrv-ui/src/services/engine.rs` (line ~267)
- [x] T015 [US3] Add `status: "failed"` message send on execution error in `crates/ckrv-ui/src/services/engine.rs` (error paths)
- [x] T016 [US3] Add fallback handling for `type: "success"` to set completed status in `crates/ckrv-ui/frontend/src/components/ExecutionRunner.tsx`
- [x] T017 [US3] Test execution status transitions through all states (starting → running → completed)

**Checkpoint**: Execution status correctly shows Completed or Failed

---

## Phase 6: Polish & Final Validation

**Goal**: Verify all fixes work together, clean up, test backward compatibility

- [x] T018 Run `make install` and verify build succeeds
- [ ] T019 Manual end-to-end test: Start execution, verify timer + counter + status all work
- [ ] T020 Verify backward compatibility: Old log message patterns still update UI correctly

**Checkpoint**: All acceptance criteria met

---

## Dependencies & Execution Order

### Phase Dependencies

- **Phase 1 (Setup)**: No dependencies - can start immediately
- **Phase 2 (Foundational)**: Depends on Phase 1 - BLOCKS all user stories
- **Phase 3-5 (User Stories)**: All depend on Phase 2 completion, can run in parallel
- **Phase 6 (Polish)**: Depends on all user stories complete

### User Story Dependencies

| Story | Priority | Depends On | Can Start After |
|-------|----------|------------|-----------------|
| US1 (Timer) | P1 | Phase 2 | Foundational |
| US2 (Counter) | P1 | Phase 2 | Foundational |
| US3 (Status) | P1 | Phase 2 | Foundational |

### Parallel Execution Opportunities

**Within Phase 2 (Foundational)**:
- T003 (frontend types) can run parallel to T004-T005 (backend types)

**Across User Stories**:
- All three user stories can be implemented in parallel after Phase 2
- T006-T009 (US1) || T010-T013 (US2) || T014-T017 (US3)

---

## Implementation Strategy

### MVP Scope

**Minimum Viable Fix**: Phase 1 + Phase 2 + Phase 3 (Timer fix only)
- This addresses the most visible symptom (timer stuck)
- Can be shipped independently

### Incremental Delivery

1. **First PR**: Backend status messages (T004-T006, T011-T012, T014-T015)
2. **Second PR**: Frontend handling (T003, T007-T010, T013, T016)
3. **Third PR**: Polish and validation (T018-T020)

### File Change Summary

| File | Changes |
|------|---------|
| `crates/ckrv-ui/src/services/engine.rs` | Add 5 status message sends, enhance LogMessage struct |
| `crates/ckrv-ui/frontend/src/components/ExecutionRunner.tsx` | Add regex pattern, add fallback handlers |
| `crates/ckrv-ui/frontend/src/types/websocket.ts` | New file with type definitions |

---

## Acceptance Criteria Mapping

| Criteria | Task(s) | Phase |
|----------|---------|-------|
| Elapsed timer starts counting | T006, T007, T008, T009 | Phase 3 |
| Batch counter updates | T010, T011, T012, T013 | Phase 4 |
| Status transitions correctly | T014, T015, T016, T017 | Phase 5 |
| Existing functionality works | T020 | Phase 6 |
| No console errors | T019 | Phase 6 |
