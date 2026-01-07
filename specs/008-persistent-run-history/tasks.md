# Tasks: Persistent Run History

**Feature**: 008-persistent-run-history  
**Branch**: `008-persistent-run-history`  
**Generated**: 2026-01-07

## Overview

This task list implements persistent run history for the Execution Runner, including YAML-based storage, history panel UI, completion summaries, and UI consistency updates.

## Task Summary

| Phase | Description | Task Count |
|-------|-------------|------------|
| Phase 1 | Setup | 2 |
| Phase 2 | Foundational - Data Types & Storage | 8 |
| Phase 3 | US1 - View Execution History | 10 |
| Phase 4 | US2 - UI Consistency | 6 |
| Phase 5 | US3 - Run Completion Summary | 5 |
| Phase 6 | US4 - Resume/Retry Failed Runs | 5 |
| Phase 7 | Polish & Validation | 4 |
| **Total** | | **40** |

---

## Phase 1: Setup

**Goal**: Prepare workspace and create file structure

- [x] T001 Checkout branch `008-persistent-run-history` and verify clean state
- [x] T002 Create directory structure for new files: `crates/ckrv-ui/src/models/`, `crates/ckrv-ui/frontend/src/hooks/`

**Checkpoint**: Directory structure ready for new files

---

## Phase 2: Foundational - Data Types & Storage (BLOCKS ALL USER STORIES)

**Goal**: Implement core data types and YAML storage service

### Backend Types

- [x] T003 [P] Create Run, BatchResult, RunSummary structs in `crates/ckrv-ui/src/models/history.rs`
- [x] T004 [P] Create RunHistory struct with YAML serialization in `crates/ckrv-ui/src/models/history.rs`
- [x] T005 Add models module export in `crates/ckrv-ui/src/models/mod.rs`

### Backend Storage Service

- [x] T006 Create HistoryService with load_history() in `crates/ckrv-ui/src/services/history.rs`
- [x] T007 Add save_history() with atomic write (temp file + rename) in `crates/ckrv-ui/src/services/history.rs`
- [x] T008 Add create_run() and update_run() methods in `crates/ckrv-ui/src/services/history.rs`
- [x] T009 Add services module export in `crates/ckrv-ui/src/services/mod.rs`

### Frontend Types

- [x] T010 [P] Create Run, BatchResult, RunSummary interfaces in `crates/ckrv-ui/frontend/src/types/history.ts`

**Checkpoint**: Data types and storage functions available for use

---

## Phase 3: User Story 1 - View Execution History (Priority: P1)

**Goal**: Persist and display run history that survives page refresh

**Independent Test**: Run execution, refresh page, verify history visible with correct status

### Backend API

- [x] T011 [US1] Create GET /api/history/{spec} handler in `crates/ckrv-ui/src/api/history.rs`
- [x] T012 [US1] Create GET /api/history/{spec}/{run_id} handler in `crates/ckrv-ui/src/api/history.rs`
- [x] T013 [US1] Create POST /api/history/{spec} handler for new runs in `crates/ckrv-ui/src/api/history.rs`
- [x] T014 [US1] Create PATCH /api/history/{spec}/{run_id} for updates in `crates/ckrv-ui/src/api/history.rs`
- [x] T015 [US1] Register history routes in `crates/ckrv-ui/src/api/mod.rs`

### Engine Integration

- [x] T016 [US1] Integrate history persistence on run start in `crates/ckrv-ui/src/services/engine.rs`
- [x] T017 [US1] Integrate history persistence on batch status change in `crates/ckrv-ui/src/services/engine.rs`
- [x] T018 [US1] Integrate history persistence on run complete/fail in `crates/ckrv-ui/src/services/engine.rs`

### Frontend

- [x] T019 [US1] Create useRunHistory hook in `crates/ckrv-ui/frontend/src/hooks/useRunHistory.ts`
- [x] T020 [US1] Create RunHistoryPanel component in `crates/ckrv-ui/frontend/src/components/RunHistoryPanel.tsx`

**Checkpoint**: Run history persists and displays after page refresh

---

## Phase 4: User Story 2 - UI Consistency (Priority: P2)

**Goal**: Match Runner page styling to other application pages

**Independent Test**: Visual comparison of Runner vs Planner/Tasks pages

### UI Updates

- [ ] T021 [US2] Update Runner page header to match other pages in `crates/ckrv-ui/frontend/src/components/ExecutionRunner.tsx`
- [ ] T022 [US2] Update panel layout (sidebar + main) to match in `crates/ckrv-ui/frontend/src/components/ExecutionRunner.tsx`
- [ ] T023 [US2] Update empty state styling to match in `crates/ckrv-ui/frontend/src/components/ExecutionRunner.tsx`
- [ ] T024 [US2] Update batch card styling to use shared design tokens in `crates/ckrv-ui/frontend/src/components/ExecutionRunner.tsx`
- [ ] T025 [US2] Update button styles to match other pages in `crates/ckrv-ui/frontend/src/components/ExecutionRunner.tsx`
- [ ] T026 [US2] Add shared CSS variables for consistent spacing in `crates/ckrv-ui/frontend/src/index.css`

**Checkpoint**: Runner page visually consistent with other pages

---

## Phase 5: User Story 3 - Run Completion Summary (Priority: P2)

**Goal**: Display clear summary when runs complete

**Independent Test**: Complete a full run, verify summary shows batches, time, branches

### Frontend Components

- [x] T027 [US3] Create CompletionSummary component in `crates/ckrv-ui/frontend/src/components/CompletionSummary.tsx`
- [x] T028 [US3] Add summary display logic (batches, time, branches) in `crates/ckrv-ui/frontend/src/components/CompletionSummary.tsx`
- [x] T029 [US3] Add failure summary variant (partial success) in `crates/ckrv-ui/frontend/src/components/CompletionSummary.tsx`
- [x] T030 [US3] Integrate CompletionSummary into ExecutionRunner in `crates/ckrv-ui/frontend/src/components/ExecutionRunner.tsx`
- [x] T031 [US3] Add WebSocket handler for run_completed message in `crates/ckrv-ui/frontend/src/components/ExecutionRunner.tsx`

**Checkpoint**: Completion summary displays with accurate information

---

## Phase 6: User Story 4 - Resume/Retry Failed Runs (Priority: P3)

**Goal**: Allow resuming incomplete runs or retrying failed batches

**Independent Test**: Fail a batch, use Retry Failed to re-run only failed batches

### Backend

- [ ] T032 [US4] Add resume_run() method to engine in `crates/ckrv-ui/src/services/engine.rs`
- [ ] T033 [US4] Add retry_failed_batches() method in `crates/ckrv-ui/src/services/engine.rs`
- [ ] T034 [US4] Create POST /api/execution/resume endpoint in `crates/ckrv-ui/src/api/execution.rs`

### Frontend

- [ ] T035 [US4] Add "Resume" button to incomplete runs in `crates/ckrv-ui/frontend/src/components/RunHistoryPanel.tsx`
- [ ] T036 [US4] Add "Retry Failed" button to failed runs in `crates/ckrv-ui/frontend/src/components/RunHistoryPanel.tsx`

**Checkpoint**: Users can resume or retry runs without starting from scratch

---

## Phase 7: Polish & Final Validation

**Goal**: Verify all features work together, handle edge cases, clean up

- [x] T037 Add concurrent run detection (409 Conflict) in `crates/ckrv-ui/src/api/history.rs`
- [x] T038 Add graceful degradation for corrupted YAML in `crates/ckrv-ui/src/services/history.rs`
- [x] T039 Run `make install` and verify build succeeds
- [ ] T040 Manual end-to-end test: Run, refresh, verify history, completion summary

**Checkpoint**: All acceptance criteria met

---

## Dependencies & Execution Order

### Phase Dependencies

- **Phase 1 (Setup)**: No dependencies - can start immediately
- **Phase 2 (Foundational)**: Depends on Phase 1 - BLOCKS all user stories
- **Phase 3 (US1)**: Depends on Phase 2 - core feature
- **Phase 4 (US2)**: Depends on Phase 2 - can run parallel with Phase 3
- **Phase 5 (US3)**: Depends on Phase 3 - needs history in place
- **Phase 6 (US4)**: Depends on Phase 3 - needs history in place
- **Phase 7 (Polish)**: Depends on all user stories complete

### User Story Dependencies

| Story | Priority | Depends On | Can Start After |
|-------|----------|------------|-----------------|
| US1 (History) | P1 | Phase 2 | Foundational |
| US2 (UI Consistency) | P2 | Phase 2 | Foundational (parallel with US1) |
| US3 (Completion Summary) | P2 | US1 | View History |
| US4 (Resume/Retry) | P3 | US1 | View History |

### Parallel Execution Opportunities

**Within Phase 2 (Foundational)**:
- T003-T004 (backend types) || T010 (frontend types)

**Across User Stories**:
- US1 and US2 can run in parallel after Phase 2
- T011-T018 (US1 backend) || T021-T026 (US2 UI)

---

## Implementation Strategy

### MVP Scope

**Minimum Viable Product**: Phase 1 + Phase 2 + Phase 3 (US1: View History)
- Core value delivered: Run history persists
- Can ship independently

### Incremental Delivery

1. **First PR**: Backend types + storage + API (T003-T018)
2. **Second PR**: Frontend history panel (T019-T020)
3. **Third PR**: UI consistency (T021-T026)
4. **Fourth PR**: Completion summary (T027-T031)
5. **Fifth PR**: Resume/retry + polish (T032-T040)

### File Change Summary

| File | Type | Phase | Changes |
|------|------|-------|---------|
| `models/history.rs` | Backend | 2 | NEW - Run, BatchResult types |
| `services/history.rs` | Backend | 2 | NEW - YAML storage service |
| `api/history.rs` | Backend | 3 | NEW - REST API handlers |
| `services/engine.rs` | Backend | 3 | Integrate history persistence |
| `types/history.ts` | Frontend | 2 | NEW - TypeScript interfaces |
| `hooks/useRunHistory.ts` | Frontend | 3 | NEW - Data fetching hook |
| `components/RunHistoryPanel.tsx` | Frontend | 3 | NEW - History list UI |
| `components/CompletionSummary.tsx` | Frontend | 5 | NEW - Summary display |
| `components/ExecutionRunner.tsx` | Frontend | 3-5 | Integrate history, summary, UI updates |
| `index.css` | Frontend | 4 | Add shared CSS variables |

---

## Acceptance Criteria Mapping

| Criteria | Task(s) | Phase |
|----------|---------|-------|
| History persists after refresh | T011-T020 | Phase 3 |
| History loads < 1 second | T011, T019 | Phase 3 |
| Visual distinction for status | T020 | Phase 3 |
| UI matches other pages | T021-T026 | Phase 4 |
| Completion summary shows | T027-T031 | Phase 5 |
| Storage < 50KB per run | T003-T004, T007 | Phase 2 |
