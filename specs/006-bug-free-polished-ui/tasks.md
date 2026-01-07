# Tasks: Bug-Free and Polished Chakravarti CLI UI

**Input**: Design documents from `/specs/006-bug-free-polished-ui/`  
**Prerequisites**: plan.md ✅, spec.md ✅, research.md ✅, data-model.md ✅, contracts/ ✅, quickstart.md ✅, bugs.md ✅

**Tests**: E2E tests are REQUIRED per spec TR-001 through TR-007. Each bug fix requires a regression test (SC-009).

**Organization**: Tasks are grouped by user story to enable independent implementation and testing. Bug fixes are mapped to the user story they support.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel (different files, no dependencies)
- **[Story]**: Which user story this task belongs to (US1, US2, US3, US4, US5)
- Include exact file paths in descriptions

## Path Conventions

- **Frontend**: `crates/ckrv-ui/frontend/src/`
- **Backend**: `crates/ckrv-ui/src/`
- **Tests**: `crates/ckrv-ui/frontend/tests/`

---

## Phase 1: Setup (Test Infrastructure) ✅ COMPLETE

**Purpose**: Set up E2E testing infrastructure with proper isolation

- [x] T001 Install Playwright and configure in `crates/ckrv-ui/frontend/package.json`
- [x] T002 Create test helper for isolated temp directories in `crates/ckrv-ui/frontend/tests/helpers/test-project.ts`
- [x] T003 [P] Create sample project fixture in `crates/ckrv-ui/frontend/tests/fixtures/sample-project/`
- [x] T004 [P] Create Playwright config with temp directory isolation in `crates/ckrv-ui/frontend/playwright.config.ts`
- [x] T005 [P] Add test scripts to `crates/ckrv-ui/frontend/package.json` (test:e2e, test:e2e:ui)
- [x] T006 Add CKRV_PROJECT_ROOT environment variable support in `crates/ckrv-ui/src/server.rs`

---

## Phase 2: Foundational (Shared Components) ✅ COMPLETE

**Purpose**: Create reusable components that multiple bug fixes and user stories depend on

**⚠️ CRITICAL**: No user story work can begin until this phase is complete

- [x] T007 Create ErrorBoundary component in `crates/ckrv-ui/frontend/src/components/ErrorBoundary.tsx` (BUG-006)
- [x] T008 [P] Create LoadingButton component in `crates/ckrv-ui/frontend/src/components/ui/LoadingButton.tsx` (BUG-004)
- [x] T009 [P] Create LoadingOverlay component in `crates/ckrv-ui/frontend/src/components/ui/LoadingOverlay.tsx` (BUG-004)
- [x] T010 [P] Add animation timing CSS custom properties in `crates/ckrv-ui/frontend/src/index.css` (BUG-009)
- [x] T011 Create useWebSocketReconnect hook in `crates/ckrv-ui/frontend/src/hooks/useWebSocketReconnect.ts` (BUG-002)
- [x] T012 [P] Create useFocusTrap hook in `crates/ckrv-ui/frontend/src/hooks/useFocusTrap.ts` (BUG-010)
- [x] T013 [P] Create useTimeout hook with cleanup in `crates/ckrv-ui/frontend/src/hooks/useTimeout.ts` (BUG-005)
- [x] T014 Wrap App with ErrorBoundary in `crates/ckrv-ui/frontend/src/App.tsx`
- [x] T015 Add data-testid attributes to key interactive elements across all components

**Checkpoint**: Foundation ready - user story implementation can now begin

---

## Phase 3: User Story 1 - Execution Runner Reliability (Priority: P1) 🎯 MVP ✅ COMPLETE

**Goal**: Fix UI freezing during execution and ensure reliable WebSocket handling

**Independent Test**: Start multi-batch execution, verify continuous log streaming, progress updates, and responsive controls throughout execution lifecycle.

**Bugs Addressed**: BUG-001 (UI Freeze), BUG-002 (WebSocket Reconnection), BUG-003 (Terminal Reset), BUG-005 (Memory Leak), BUG-007 (Log Scroll), BUG-008 (XTerm Race)

### E2E Tests for User Story 1

> **NOTE: Write these tests FIRST, ensure they FAIL before implementation**

- [x] T016 [P] [US1] E2E test for execution start/stop flow in `crates/ckrv-ui/frontend/tests/e2e/execution-runner.spec.ts`
- [x] T017 [P] [US1] E2E test for rapid log streaming (50+ messages/second) in `crates/ckrv-ui/frontend/tests/e2e/execution-runner.spec.ts`
- [x] T018 [P] [US1] E2E test for WebSocket reconnection in `crates/ckrv-ui/frontend/tests/e2e/execution-runner.spec.ts`
- [x] T019 [P] [US1] E2E test for terminal reset on new execution in `crates/ckrv-ui/frontend/tests/e2e/execution-runner.spec.ts`

### Implementation for User Story 1

- [x] T020 [US1] Implement requestAnimationFrame message batching in `crates/ckrv-ui/frontend/src/components/ExecutionRunner.tsx` lines 536-566 (BUG-001)
- [x] T021 [US1] Replace direct state updates with batched updates using useTransition in `crates/ckrv-ui/frontend/src/components/ExecutionRunner.tsx` (BUG-001)
- [x] T022 [US1] Add 'reconnecting' status to ExecutionStatus type in `crates/ckrv-ui/frontend/src/components/ExecutionRunner.tsx` lines 38 (BUG-002)
- [x] T023 [US1] Integrate useWebSocketReconnect hook in ExecutionRunner connectWebSocket function in `crates/ckrv-ui/frontend/src/components/ExecutionRunner.tsx` lines 523-578 (BUG-002)
- [x] T024 [US1] Add visible countdown indicator for WebSocket retry in `crates/ckrv-ui/frontend/src/components/ExecutionRunner.tsx` (BUG-002)
- [x] T025 [US1] Fix terminal clear race condition with callback ref pattern in `crates/ckrv-ui/frontend/src/components/ExecutionRunner.tsx` lines 580-612 (BUG-003)
- [x] T026 [US1] Track setTimeout IDs in ref and cleanup on unmount in `crates/ckrv-ui/frontend/src/components/ExecutionRunner.tsx` lines 495-498 (BUG-005)
- [x] T027 [US1] Implement auto-scroll with user scroll detection in BatchLogPanel in `crates/ckrv-ui/frontend/src/components/ExecutionRunner.tsx` lines 180-273 (BUG-007)
- [x] T028 [US1] Add "scroll to bottom" button when not at bottom in BatchLogPanel in `crates/ckrv-ui/frontend/src/components/ExecutionRunner.tsx` (BUG-007)
- [x] T029 [US1] Fix useEffect dependency array and add ResizeObserver in `crates/ckrv-ui/frontend/src/components/LogTerminal.tsx` lines 89-94 (BUG-008)
- [x] T030 [US1] Debounce FitAddon.fit() calls during rapid resizes in `crates/ckrv-ui/frontend/src/components/LogTerminal.tsx` (BUG-008)

**Checkpoint**: ✅ Execution Runner is freeze-free and handles WebSocket failures gracefully

---

## Phase 4: User Story 2 - Consistent Visual Design Language (Priority: P2) ✅ COMPLETE

**Goal**: Ensure cohesive styling across all components with consistent animations

**Independent Test**: Navigate through all pages verifying consistent color schemes, typography, spacing, and animation timing.

**Bugs Addressed**: BUG-009 (Animation Durations), BUG-012 (Hardcoded Colors)

### E2E Tests for User Story 2

- [x] T031 [P] [US2] Visual regression test for all pages in `crates/ckrv-ui/frontend/tests/e2e/visual-consistency.spec.ts`
- [x] T032 [P] [US2] Test modal/panel animation timing consistency in `crates/ckrv-ui/frontend/tests/e2e/visual-consistency.spec.ts`

### Implementation for User Story 2

- [x] T033 [P] [US2] Audit and replace hardcoded transition durations in `crates/ckrv-ui/frontend/src/components/CommandPalette.tsx` (BUG-009)
- [x] T034 [P] [US2] Audit and replace hardcoded transition durations in `crates/ckrv-ui/frontend/src/components/AgentManager.tsx` (BUG-009)
- [x] T035 [P] [US2] Audit and replace hardcoded transition durations in `crates/ckrv-ui/frontend/src/components/SpecEditor.tsx` (BUG-009)
- [x] T036 [P] [US2] Audit and replace hardcoded transition durations in `crates/ckrv-ui/frontend/src/components/TaskEditor.tsx` (BUG-009)
- [x] T037 [P] [US2] Audit and replace hardcoded transition durations in `crates/ckrv-ui/frontend/src/components/PlanEditor.tsx` (BUG-009)
- [x] T038 [P] [US2] Audit and replace hardcoded transition durations in `crates/ckrv-ui/frontend/src/components/DiffViewer.tsx` (BUG-009)
- [x] T039 [US2] Create theme constants for XTerm.js derived from CSS variables in `crates/ckrv-ui/frontend/src/components/LogTerminal.tsx` (BUG-012)
- [x] T040 [US2] Use getComputedStyle to sync terminal theme with CSS variables in `crates/ckrv-ui/frontend/src/components/LogTerminal.tsx` (BUG-012)

**Checkpoint**: ✅ All components use consistent animation timing and theming

---

## Phase 5: User Story 3 - Error State Handling and Feedback (Priority: P2) ✅ COMPLETE

**Goal**: Display clear, actionable error messages for all failure scenarios

**Independent Test**: Trigger various error conditions and verify appropriate error messages are displayed.

**Bugs Addressed**: BUG-004 (Loading States), BUG-006 (Error Boundary) - already addressed in Phase 2

### E2E Tests for User Story 3

- [x] T041 [P] [US3] E2E test for API error handling in `crates/ckrv-ui/frontend/tests/e2e/error-handling.spec.ts`
- [x] T042 [P] [US3] E2E test for network timeout handling in `crates/ckrv-ui/frontend/tests/e2e/error-handling.spec.ts`
- [x] T043 [P] [US3] E2E test for form validation errors in `crates/ckrv-ui/frontend/tests/e2e/error-handling.spec.ts`

### Implementation for User Story 3

- [x] T044 [P] [US3] Replace loading buttons with LoadingButton component in `crates/ckrv-ui/frontend/src/components/CommandPalette.tsx` (BUG-004)
- [x] T045 [P] [US3] Replace loading buttons with LoadingButton component in `crates/ckrv-ui/frontend/src/components/AgentManager.tsx` (BUG-004)
- [x] T046 [P] [US3] Add LoadingOverlay to async fetch components in `crates/ckrv-ui/frontend/src/components/SpecEditor.tsx` (BUG-004)
- [x] T047 [P] [US3] Add LoadingOverlay to async fetch components in `crates/ckrv-ui/frontend/src/components/TaskEditor.tsx` (BUG-004)
- [x] T048 [P] [US3] Add LoadingOverlay to async fetch components in `crates/ckrv-ui/frontend/src/components/PlanEditor.tsx` (BUG-004)
- [x] T049 [US3] Add inline form validation error messages in `crates/ckrv-ui/frontend/src/components/AgentManager.tsx`
- [x] T050 [US3] Add timeout handling with user-friendly message for API calls in `crates/ckrv-ui/frontend/src/App.tsx`

**Checkpoint**: ✅ All errors display clear messages with retry options

---

## Phase 6: User Story 4 - Responsive Layout and Scrolling (Priority: P3) ✅ COMPLETE

**Goal**: Ensure dashboard adapts gracefully to different viewport sizes

**Independent Test**: Resize browser window to various dimensions and verify content remains accessible.

### E2E Tests for User Story 4

- [x] T051 [P] [US4] E2E test for 1280px viewport rendering in `crates/ckrv-ui/frontend/tests/e2e/responsive.spec.ts`
- [x] T052 [P] [US4] E2E test for panel minimize/maximize transitions in `crates/ckrv-ui/frontend/tests/e2e/responsive.spec.ts`

### Implementation for User Story 4

- [x] T053 [US4] Audit and fix horizontal overflow issues at 1280px in `crates/ckrv-ui/frontend/src/components/ExecutionRunner.tsx`
- [x] T054 [P] [US4] Ensure fixed headers during scroll in list components in `crates/ckrv-ui/frontend/src/components/TaskEditor.tsx`
- [x] T055 [P] [US4] Ensure fixed headers during scroll in list components in `crates/ckrv-ui/frontend/src/components/AgentManager.tsx`
- [x] T056 [US4] Review and fix scroll chaining in nested scrollable areas in all modal components

**Checkpoint**: ✅ Dashboard renders correctly on all standard developer screen sizes

---

## Phase 7: User Story 5 - Keyboard Navigation and Accessibility (Priority: P3) ✅ COMPLETE

**Goal**: Enable efficient keyboard navigation and screen reader support

**Independent Test**: Navigate entire dashboard using only keyboard (Tab, Enter, Escape) and verify all interactive elements are reachable.

**Bugs Addressed**: BUG-010 (Focus Trap), BUG-011 (ARIA Labels)

### E2E Tests for User Story 5

- [x] T057 [P] [US5] Accessibility audit test using axe-core in `crates/ckrv-ui/frontend/tests/e2e/accessibility.spec.ts`
- [x] T058 [P] [US5] E2E test for modal focus trap in `crates/ckrv-ui/frontend/tests/e2e/accessibility.spec.ts`
- [x] T059 [P] [US5] E2E test for keyboard navigation flow in `crates/ckrv-ui/frontend/tests/e2e/accessibility.spec.ts`

### Implementation for User Story 5

- [x] T060 [P] [US5] Add focus trap to modal in `crates/ckrv-ui/frontend/src/components/AgentManager.tsx` (BUG-010)
- [x] T061 [P] [US5] Add focus trap to modal in `crates/ckrv-ui/frontend/src/components/CommandPalette.tsx` (BUG-010)
- [x] T062 [P] [US5] Add focus trap to modal in `crates/ckrv-ui/frontend/src/components/TaskDetailModal.tsx` (BUG-010)
- [x] T063 [P] [US5] Add aria-label to all icon-only buttons in `crates/ckrv-ui/frontend/src/components/ExecutionRunner.tsx` (BUG-011)
- [x] T064 [P] [US5] Add aria-label to all icon-only buttons in `crates/ckrv-ui/frontend/src/components/CommandPalette.tsx` (BUG-011)
- [x] T065 [P] [US5] Add aria-label to all icon-only buttons in `crates/ckrv-ui/frontend/src/components/TaskEditor.tsx` (BUG-011)
- [x] T066 [P] [US5] Add aria-label to all icon-only buttons in `crates/ckrv-ui/frontend/src/components/SpecEditor.tsx` (BUG-011)
- [x] T067 [P] [US5] Add role and aria-* attributes to ProgressRing in `crates/ckrv-ui/frontend/src/components/ExecutionRunner.tsx` (BUG-011)
- [x] T068 [US5] Add visible focus indicators (outline) to all interactive elements in `crates/ckrv-ui/frontend/src/index.css`
- [x] T069 [US5] Ensure Escape key closes modals and returns focus to trigger element across all modals

**Checkpoint**: ✅ Dashboard is fully keyboard-navigable and screen reader compatible

---

## Phase 8: Polish & Final Validation ✅ COMPLETE

**Purpose**: Final cleanup and cross-cutting improvements

- [x] T070 [P] Run full E2E test suite and fix any failures
- [x] T071 [P] Run ESLint and fix all warnings in `crates/ckrv-ui/frontend/src/`
- [x] T072 [P] Run Prettier and format all files in `crates/ckrv-ui/frontend/src/`
- [x] T073 Validate all 12 bugs from bugs.md are fixed with regression tests
- [x] T074 Update bugs.md with resolution status for each bug
- [x] T075 Run quickstart.md validation script
- [x] T076 Final manual testing of spec → task → plan → run workflow

---

## Dependencies & Execution Order

### Phase Dependencies

- **Setup (Phase 1)**: No dependencies - can start immediately
- **Foundational (Phase 2)**: Depends on Setup completion - BLOCKS all user stories
- **User Stories (Phase 3-7)**: All depend on Foundational phase completion
  - User stories can proceed in parallel (if staffed)
  - Or sequentially in priority order (P1 → P2 → P3)
- **Polish (Phase 8)**: Depends on all user stories being complete

### User Story Dependencies

| Story | Priority | Depends On | Can Start After |
|-------|----------|------------|-----------------|
| US1 (Execution Runner) | P1 | Phase 2 | Foundational |
| US2 (Visual Design) | P2 | Phase 2 | Foundational |
| US3 (Error Handling) | P2 | Phase 2 | Foundational |
| US4 (Responsive Layout) | P3 | Phase 2 | Foundational |
| US5 (Accessibility) | P3 | Phase 2 | Foundational |

### Bug to User Story Mapping

| Bug ID | Bug Title | User Story | Phase |
|--------|-----------|------------|-------|
| BUG-001 | UI Freeze | US1 | 3 |
| BUG-002 | WebSocket Reconnection | US1 | 3 |
| BUG-003 | Terminal Reset | US1 | 3 |
| BUG-004 | Loading States | US3 | 5 |
| BUG-005 | Memory Leak | US1 | 3 |
| BUG-006 | Error Boundary | Foundational | 2 |
| BUG-007 | Log Scroll | US1 | 3 |
| BUG-008 | XTerm Race | US1 | 3 |
| BUG-009 | Animation Timing | US2 | 4 |
| BUG-010 | Focus Trap | US5 | 7 |
| BUG-011 | ARIA Labels | US5 | 7 |
| BUG-012 | Hardcoded Colors | US2 | 4 |

---

## Parallel Example: User Story 1

```bash
# Launch all E2E tests for User Story 1 together:
Task: T016 "E2E test for execution start/stop flow"
Task: T017 "E2E test for rapid log streaming"
Task: T018 "E2E test for WebSocket reconnection"
Task: T019 "E2E test for terminal reset"

# These must run sequentially after tests fail:
Task: T020-T030 (implementation tasks in order)
```

---

## Implementation Strategy

### MVP First (User Story 1 Only)

1. Complete Phase 1: Setup (T001-T006)
2. Complete Phase 2: Foundational (T007-T015)
3. Complete Phase 3: User Story 1 (T016-T030)
4. **STOP and VALIDATE**: Test Execution Runner independently
5. Deploy/demo if ready - Core workflow is now reliable

### Incremental Delivery

| Milestone | Tasks | Bugs Fixed | Value Delivered |
|-----------|-------|------------|-----------------|
| Foundation | T001-T015 | BUG-006 | Error boundary, reusable hooks |
| MVP (US1) | T016-T030 | BUG-001-003, 005, 007-008 | Reliable execution runner |
| Visual Polish | T031-T040 | BUG-009, 012 | Consistent styling |
| Error Handling | T041-T050 | BUG-004 | Clear error feedback |
| Responsiveness | T051-T056 | - | Multi-screen support |
| Accessibility | T057-T069 | BUG-010-011 | Keyboard/screen reader |
| Final | T070-T076 | Validation | Production ready |

---

## Summary

| Metric | Value |
|--------|-------|
| Total Tasks | 76 |
| Setup Phase | 6 tasks |
| Foundational Phase | 9 tasks |
| User Story 1 (P1) | 15 tasks |
| User Story 2 (P2) | 10 tasks |
| User Story 3 (P2) | 10 tasks |
| User Story 4 (P3) | 6 tasks |
| User Story 5 (P3) | 13 tasks |
| Polish Phase | 7 tasks |
| Bugs Addressed | 12 |
| Parallel Opportunities | 45 tasks marked [P] |
| MVP Scope | Phases 1-3 (30 tasks) |

---

## Notes

- [P] tasks = different files, no dependencies
- [Story] label maps task to specific user story for traceability
- Each user story should be independently completable and testable
- **CRITICAL**: All tests run in temporary directories (TR-007)
- Commit after each task or logical group
- Stop at any checkpoint to validate story independently
- Verify tests fail before implementing bug fixes
