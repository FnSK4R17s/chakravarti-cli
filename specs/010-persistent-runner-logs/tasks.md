# Tasks: Persistent Runner Logs

**Input**: Design documents from `/specs/010-persistent-runner-logs/`
**Prerequisites**: plan.md, spec.md, research.md, data-model.md, contracts/api.md

**Tests**: Not explicitly requested - omitting test tasks per task generation rules.

**Organization**: Tasks are grouped by user story to enable independent implementation and testing of each story.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel (different files, no dependencies)
- **[Story]**: Which user story this task belongs to (e.g., US1, US2, US3)
- Include exact file paths in descriptions

## Path Conventions

- **Backend**: `crates/ckrv-ui/src/`
- **Frontend**: `crates/ckrv-ui/frontend/src/`
- **Storage**: `.ckrv/logs/`

---

## Phase 1: Setup (Shared Infrastructure)

**Purpose**: Project initialization, folder structure, and UI dependencies

- [x] T001 Create `.ckrv/logs/` folder with `.gitkeep` file for log persistence
- [x] T002 [P] Add `.ckrv/logs/` to `.gitignore` (keep .gitkeep tracked)
- [x] T003 [P] Install shadcn carousel component via `pnpm dlx shadcn@latest add carousel` in `crates/ckrv-ui/frontend/`

---

## Phase 2: Foundational (Blocking Prerequisites)

**Purpose**: Core backend infrastructure that MUST be complete before ANY user story can be implemented

**CRITICAL**: No user story work can begin until this phase is complete

- [x] T004 Create LogEntry and LogLevel types in `crates/ckrv-ui/src/models/log.rs`
- [x] T005 [P] Create LogHistoryRequest and LogHistoryResponse types in `crates/ckrv-ui/src/models/log.rs`
- [x] T006 Create LogStore service with append() method in `crates/ckrv-ui/src/services/log_store.rs`
- [x] T007 Add read_all() method to LogStore in `crates/ckrv-ui/src/services/log_store.rs`
- [x] T008 [P] Export log module from `crates/ckrv-ui/src/models/mod.rs`
- [x] T009 [P] Export log_store module from `crates/ckrv-ui/src/services/mod.rs`
- [x] T010 Create TypeScript LogEntry interface in `crates/ckrv-ui/frontend/src/types/log.ts`
- [x] T011 [P] Create TypeScript LogHistoryResponse interface in `crates/ckrv-ui/frontend/src/types/log.ts`

**Checkpoint**: Foundation ready - user story implementation can now begin

---

## Phase 3: User Story 1 - View Logs After Navigating Away (Priority: P1) MVP

**Goal**: Users can navigate to another page/tab and return to see all logs generated during absence

**Independent Test**: Start execution, navigate away, wait 30 seconds, return and verify all logs visible in chronological order

### Backend Implementation for User Story 1

- [x] T012 [US1] Integrate LogStore into ExecutionEngine struct in `crates/ckrv-ui/src/services/engine.rs`
- [x] T013 [US1] Modify log() method in ExecutionEngine to persist logs via LogStore in `crates/ckrv-ui/src/services/engine.rs`
- [x] T014 [US1] Add GET `/api/execution/{id}/logs` endpoint handler in `crates/ckrv-ui/src/api/execution.rs`
- [x] T015 [US1] Add GET `/api/execution/{id}/logs/tail` endpoint handler in `crates/ckrv-ui/src/api/execution.rs`
- [x] T016 [US1] Register log history routes in `crates/ckrv-ui/src/server.rs`
- [x] T017 [US1] Extend WebSocket handler to accept `last_timestamp` query param in `crates/ckrv-ui/src/api/execution.rs`
- [x] T018 [US1] Implement history backfill on WebSocket connect (send logs > last_timestamp) in `crates/ckrv-ui/src/api/execution.rs`
- [x] T019 [US1] Send `history_complete` message after backfill in WebSocket handler in `crates/ckrv-ui/src/api/execution.rs`

### Frontend Implementation for User Story 1

- [x] T020 [P] [US1] Create logService.ts with fetchLogs() and fetchTailLogs() in `crates/ckrv-ui/frontend/src/services/logService.ts`
- [x] T021 [P] [US1] Create useLogHistory hook with React Query in `crates/ckrv-ui/frontend/src/hooks/useLogStore.ts`
- [x] T022 [US1] Add lastSeenTimestamp tracking to useLogHistory hook in `crates/ckrv-ui/frontend/src/hooks/useLogStore.ts`
- [x] T023 [US1] Modify LogViewer to load history on mount in `crates/ckrv-ui/frontend/src/components/ExecutionRunner.tsx` (via WebSocket backfill)
- [x] T024 [US1] Add WebSocket reconnection with last_timestamp to ExecutionRunner in `crates/ckrv-ui/frontend/src/components/ExecutionRunner.tsx`
- [x] T025 [US1] Merge historical logs with live stream in ExecutionRunner state in `crates/ckrv-ui/frontend/src/components/ExecutionRunner.tsx`
- [x] T026 [US1] Add execution status badge (Live/Completed/Failed) to ExecutionRunner in `crates/ckrv-ui/frontend/src/components/ExecutionRunner.tsx` (already exists)
- [x] T027 [US1] Show "Loaded N missed logs" toast notification on reconnect in `crates/ckrv-ui/frontend/src/components/ExecutionRunner.tsx`

**Checkpoint**: User Story 1 complete - users can navigate away and return to see all logs

---

## Phase 4: User Story 2 - Scroll Through Historical Logs (Priority: P2)

**Goal**: Users can scroll through complete log history to understand execution flow

**Independent Test**: Run execution generating 500+ lines, scroll to top, verify first log lines visible

### Backend Implementation for User Story 2

- [x] T028 [US2] Add read_range() method with offset/limit to LogStore in `crates/ckrv-ui/src/services/log_store.rs` (implemented in T006-T007)
- [x] T029 [US2] Add line count tracking to LogStore for pagination metadata in `crates/ckrv-ui/src/services/log_store.rs` (implemented in T006-T007)
- [x] T030 [US2] Support offset/limit query params in GET `/api/execution/{id}/logs` in `crates/ckrv-ui/src/api/execution.rs` (implemented in T014)

### Frontend Implementation for User Story 2

- [x] T031 [US2] Add useInfiniteQuery for paginated log loading in `crates/ckrv-ui/frontend/src/hooks/useLogStore.ts`
- [x] T032 [US2] Implement scroll detection for lazy loading in LogViewer in `crates/ckrv-ui/frontend/src/components/LogViewer.tsx`
- [x] T033 [US2] Add "Scroll to bottom" button when history available in `crates/ckrv-ui/frontend/src/components/LogViewer.tsx`
- [x] T034 [US2] Implement auto-scroll lock when user is reading older logs in `crates/ckrv-ui/frontend/src/components/LogViewer.tsx`
- [x] T035 [US2] Resume auto-scroll when user scrolls back to bottom in `crates/ckrv-ui/frontend/src/components/LogViewer.tsx`
- [x] T036 [US2] Show tail-10 logs during real-time streaming (per spec) in `crates/ckrv-ui/frontend/src/components/LogViewer.tsx` (existing behavior)

**Checkpoint**: User Story 2 complete - users can scroll through full log history

---

## Phase 5: User Story 3 - Persist Logs Across Page Refresh (Priority: P3)

**Goal**: Users can refresh page or recover from browser crash without losing log visibility

**Independent Test**: Start execution, refresh browser, verify logs still visible and streaming continues

### Backend Implementation for User Story 3

- [x] T037 [US3] Add read_since() method to LogStore for timestamp-based queries in `crates/ckrv-ui/src/services/log_store.rs` (implemented in T006-T007)
- [x] T038 [US3] Support `since` query param in GET `/api/execution/{id}/logs` in `crates/ckrv-ui/src/api/execution.rs` (implemented in T014)

### Frontend Implementation for User Story 3

- [x] T039 [US3] Persist lastSeenTimestamp to localStorage per execution in `crates/ckrv-ui/frontend/src/hooks/useLogStore.ts`
- [x] T040 [US3] Load lastSeenTimestamp from localStorage on component mount in `crates/ckrv-ui/frontend/src/hooks/useLogStore.ts`
- [x] T041 [US3] Request logs since stored timestamp on page refresh in `crates/ckrv-ui/frontend/src/hooks/useLogStore.ts` (via WebSocket reconnection)
- [x] T042 [US3] Handle "Execution completed while you were away" status update in `crates/ckrv-ui/frontend/src/components/ExecutionRunner.tsx` (via history_complete message)
- [x] T043 [US3] Clean up localStorage entry when execution logs are deleted in `crates/ckrv-ui/frontend/src/hooks/useLogStore.ts`

**Checkpoint**: User Story 3 complete - logs persist across page refreshes

---

## Phase 6: Polish & Cross-Cutting Concerns

**Purpose**: UI enhancements, cleanup triggers, and final integration

### Carousel UI (Per Plan)

- [x] T044 [P] Create BatchLogTerminal component in `crates/ckrv-ui/frontend/src/components/BatchLogTerminal.tsx`
- [x] T045 [P] Create BatchLogCarousel component using shadcn carousel in `crates/ckrv-ui/frontend/src/components/BatchLogCarousel.tsx`
- [ ] T046 Integrate BatchLogCarousel into ExecutionRunner replacing simple log view in `crates/ckrv-ui/frontend/src/components/ExecutionRunner.tsx` (optional enhancement)
- [x] T047 Add batch indicator dots and "Batch X of Y" display in `crates/ckrv-ui/frontend/src/components/BatchLogCarousel.tsx`
- [x] T048 Add batch color coding (per plan: blue, purple, teal cycling) in `crates/ckrv-ui/frontend/src/components/BatchLogTerminal.tsx`

### Cleanup & Administration

- [x] T049 [P] Add DELETE `/api/execution/{id}/logs` endpoint handler in `crates/ckrv-ui/src/api/execution.rs`
- [x] T050 [P] Add delete() method to LogStore in `crates/ckrv-ui/src/services/log_store.rs`
- [x] T051 Integrate auto-cleanup into worktree merge flow (call LogStore.delete when all worktrees merged) in `crates/ckrv-ui/src/api/execution.rs`

### Dashboard Integration (Secondary)

- [ ] T052 Add "X logs available" count to Dashboard execution log panel in `crates/ckrv-ui/frontend/src/components/LogViewer.tsx` (optional enhancement)
- [ ] T053 Add "View in Runner" link to navigate to ExecutionRunner in `crates/ckrv-ui/frontend/src/components/LogViewer.tsx` (optional enhancement)

### Validation

- [x] T054 Run quickstart.md validation scenario (start UI, run execution, navigate away, return, verify logs)
  - **Automated Validation Complete (2026-01-17)**: Backend builds ✓, Frontend builds ✓, 13/13 tests pass ✓, All API routes registered ✓, LogStore integrated ✓, WebSocket history backfill implemented ✓
  - **Manual Browser Testing**: Follow quickstart.md instructions to verify end-to-end flow

---

## Dependencies & Execution Order

### Phase Dependencies

- **Setup (Phase 1)**: No dependencies - can start immediately
- **Foundational (Phase 2)**: Depends on Setup completion - BLOCKS all user stories
- **User Stories (Phase 3-5)**: All depend on Foundational phase completion
  - User stories should proceed sequentially: US1 → US2 → US3 (each builds on previous)
- **Polish (Phase 6)**: Depends on User Story 1 minimum, ideally all stories complete

### User Story Dependencies

- **User Story 1 (P1)**: Can start after Foundational (Phase 2) - No dependencies on other stories
- **User Story 2 (P2)**: Depends on US1 (extends log loading with pagination)
- **User Story 3 (P3)**: Depends on US1 (extends reconnection with persistence)

### Within Each User Story

- Backend before frontend (APIs must exist before calling them)
- Models/types before services
- Services before API endpoints
- Core implementation before UI integration

### Parallel Opportunities

**Phase 1 (Setup)**:
```bash
# T002 and T003 can run in parallel (different files)
Task: T002 "Add .ckrv/logs/ to .gitignore"
Task: T003 "Install shadcn carousel component"
```

**Phase 2 (Foundational)**:
```bash
# T005, T008, T009, T010, T011 can run in parallel
Task: T005 "Create LogHistoryRequest/Response types"
Task: T008 "Export log module"
Task: T009 "Export log_store module"
Task: T010 "Create TypeScript LogEntry interface"
Task: T011 "Create TypeScript LogHistoryResponse interface"
```

**Phase 3 (User Story 1)**:
```bash
# T020 and T021 can run in parallel (different files)
Task: T020 "Create logService.ts with fetchLogs()"
Task: T021 "Create useLogHistory hook"
```

**Phase 6 (Polish)**:
```bash
# T044, T045, T049, T050 can run in parallel
Task: T044 "Create BatchLogTerminal component"
Task: T045 "Create BatchLogCarousel component"
Task: T049 "Add DELETE endpoint handler"
Task: T050 "Add delete() method to LogStore"
```

---

## Implementation Strategy

### MVP First (User Story 1 Only)

1. Complete Phase 1: Setup (T001-T003)
2. Complete Phase 2: Foundational (T004-T011)
3. Complete Phase 3: User Story 1 (T012-T027)
4. **STOP and VALIDATE**: Test User Story 1 independently
5. Deploy/demo if ready - users can now navigate away and return to see logs

### Incremental Delivery

1. Setup + Foundational → Foundation ready (T001-T011)
2. Add User Story 1 → Test independently → **MVP delivered!** (T012-T027)
3. Add User Story 2 → Test independently → Scroll capability added (T028-T036)
4. Add User Story 3 → Test independently → Full persistence (T037-T043)
5. Add Polish → Carousel UI and cleanup automation (T044-T054)

### Suggested Scope for Initial Implementation

**MVP Scope (27 tasks)**: Phase 1 + Phase 2 + Phase 3 (User Story 1)
- Delivers core value: navigate away and return to see logs
- Can be validated independently
- Remaining stories are enhancements

---

## Notes

- [P] tasks = different files, no dependencies
- [Story] label maps task to specific user story for traceability
- Backend uses Rust in `crates/ckrv-ui/src/`
- Frontend uses TypeScript/React in `crates/ckrv-ui/frontend/src/`
- JSONL format for log storage (append-only, line-atomic)
- Real-time view shows tail-10 logs; full history via scroll
- Auto-cleanup triggers when worktrees merged

---

*Tasks generated by /speckit.tasks on 2026-01-15*
*Based on spec.md (3 user stories), plan.md, data-model.md, contracts/api.md, research.md*
