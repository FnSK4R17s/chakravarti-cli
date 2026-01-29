# Tasks: Unified Code Page

**Input**: Design documents from `/specs/015-unified-code-page/`  
**Prerequisites**: plan.md ✅, spec.md ✅, research.md ✅, data-model.md ✅, contracts/ ✅

**Tests**: E2E tests are included per plan.md specification (Playwright).

**Organization**: Tasks are grouped by user story to enable independent implementation and testing of each story.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel (different files, no dependencies)
- **[Story]**: Which user story this task belongs to (e.g., US1, US2, US3)
- Include exact file paths in descriptions

## Path Conventions

- **Web app**: `crates/ckrv-ui/frontend/src/` (React frontend)
- **Tests**: `crates/ckrv-ui/frontend/tests/e2e/` (Playwright E2E)

---

## Phase 1: Setup ✅ COMPLETE

**Purpose**: Define types and prepare for CodePage implementation

- [x] T001 Define CodeTabType and update PageType in `crates/ckrv-ui/frontend/src/types.ts`
- [x] T002 [P] Export CODE_TABS constant array with tab metadata (id, label, icon) in `crates/ckrv-ui/frontend/src/types.ts`

---

## Phase 2: Foundational (Core Implementation) ✅ COMPLETE

**Purpose**: Create the CodePage component and wire up navigation - blocks all user stories

**⚠️ CRITICAL**: User story validation cannot begin until this phase is complete

- [x] T003 Create CodePage component with Radix Tabs structure in `crates/ckrv-ui/frontend/src/components/CodePage.tsx`
- [x] T004 Import and render SpecEditor, TaskEditor, PlanEditor, ExecutionRunner as tab content in `crates/ckrv-ui/frontend/src/components/CodePage.tsx`
- [x] T005 Add local useState for activeTab with 'spec' as default in `crates/ckrv-ui/frontend/src/components/CodePage.tsx`
- [x] T006 Update PageType union in NavigationContext removing specs/tasks/plan/runner, adding 'code' in `crates/ckrv-ui/frontend/src/App.tsx`
- [x] T007 Replace 4 page conditions (specs, tasks, plan, runner) with single CodePage condition in `crates/ckrv-ui/frontend/src/App.tsx`
- [x] T008 Update sidebar navigation from 9 items to 5 items (Dashboard, Agents, Code, Test, QA) in `crates/ckrv-ui/frontend/src/layouts/Dashboard.tsx`
- [x] T009 Update pageTitles object to remove old pages, add 'code' title in `crates/ckrv-ui/frontend/src/layouts/Dashboard.tsx`

**Checkpoint**: Navigation works - Code page displays with tabs, all 4 sub-views accessible ✅

---

## Phase 3: User Story 1 - View Code Workflow in Single Page (Priority: P1) 🎯 MVP ✅ COMPLETE

**Goal**: Access all 4 workflow stages (Spec, Tasks, Plan, Run) in a single tabbed page without context switching

**Independent Test**: Navigate to Code page, click each of the 4 tabs, verify each shows the correct component and sidebar shows 5 items

### E2E Tests for User Story 1

- [x] T010 [P] [US1] Create E2E test file with navigation to Code page test in `crates/ckrv-ui/frontend/tests/e2e/code-page.spec.ts`
- [x] T011 [P] [US1] Add E2E test: clicking each tab shows correct content in `crates/ckrv-ui/frontend/tests/e2e/code-page.spec.ts`
- [x] T012 [P] [US1] Add E2E test: sidebar shows exactly 5 navigation items in `crates/ckrv-ui/frontend/tests/e2e/code-page.spec.ts`

### Implementation for User Story 1

- [x] T013 [US1] Style tab bar with icons (FileText, ListTodo, Workflow, Rocket) from lucide-react in `crates/ckrv-ui/frontend/src/components/CodePage.tsx`
- [x] T014 [US1] Ensure keyboard navigation works (arrow keys between tabs) - Radix handles this by default, verify in `crates/ckrv-ui/frontend/src/components/CodePage.tsx`
- [x] T015 [US1] Update page header to show "Code - [Tab Name]" based on active tab in `crates/ckrv-ui/frontend/src/components/CodePage.tsx`
- [x] T016 [US1] Remove DiffPage from sidebar navigation (moved to Run tab context) in `crates/ckrv-ui/frontend/src/layouts/Dashboard.tsx`
- [x] T017 [US1] Clean up unused page imports (SpecsPage, TasksPage, PlanPage, RunnerPage, DiffPage) from `crates/ckrv-ui/frontend/src/App.tsx`

**Checkpoint**: User Story 1 complete - Single Code page with 4 working tabs, sidebar reduced to 5 items ✅

---

## Phase 4: User Story 2 - Persist Active Tab State (Priority: P2) ✅ COMPLETE

**Goal**: Remember which tab user was on when navigating away and back

**Independent Test**: Select Plan tab, navigate to Dashboard, return to Code, verify Plan tab is still active

### E2E Tests for User Story 2

- [x] T018 [P] [US2] Add E2E test: tab state persists after navigating away and back in `crates/ckrv-ui/frontend/tests/e2e/code-page.spec.ts`
- [x] T019 [P] [US2] Add E2E test: first visit defaults to Spec tab in `crates/ckrv-ui/frontend/tests/e2e/code-page.spec.ts`

### Implementation for User Story 2

- [x] T020 [US2] Create useCodeTab custom hook with session storage persistence in `crates/ckrv-ui/frontend/src/hooks/useCodeTab.ts`
- [x] T021 [US2] Replace useState with useCodeTab hook in CodePage in `crates/ckrv-ui/frontend/src/components/CodePage.tsx`
- [x] T022 [US2] Add sessionStorage key constant for tab state in `crates/ckrv-ui/frontend/src/hooks/useCodeTab.ts`

**Checkpoint**: User Story 2 complete - Tab state persists within session ✅

---

## Phase 5: User Story 3 - Visual Workflow Progress Indicator (Priority: P3) ✅ COMPLETE

**Goal**: Show visual indicators on tabs to indicate completion status of each workflow stage

**Independent Test**: Complete spec actions, verify Spec tab shows completion indicator; verify incomplete stages show pending styling

### E2E Tests for User Story 3

- [x] T023 [P] [US3] Add E2E test: completed stage shows checkmark indicator in `crates/ckrv-ui/frontend/tests/e2e/code-page.spec.ts`
- [x] T024 [P] [US3] Add E2E test: pending stages show neutral styling in `crates/ckrv-ui/frontend/tests/e2e/code-page.spec.ts`

### Implementation for User Story 3

- [x] T025 [US3] Create useWorkflowProgress hook that fetches spec/tasks/plan status from existing APIs in `crates/ckrv-ui/frontend/src/hooks/useWorkflowProgress.ts`
- [x] T026 [US3] Define WorkflowStage interface with status: 'pending' | 'complete' in `crates/ckrv-ui/frontend/src/types.ts`
- [x] T027 [US3] Add completion indicator (CheckCircle2 icon) to tab triggers for completed stages in `crates/ckrv-ui/frontend/src/components/CodePage.tsx`
- [x] T028 [US3] Style pending tabs with muted appearance, completed tabs with accent color in `crates/ckrv-ui/frontend/src/components/CodePage.tsx`

**Checkpoint**: User Story 3 complete - Tabs show visual progress indicators ✅

---

## Phase 6: Polish & Cross-Cutting Concerns ✅ COMPLETE

**Purpose**: Final cleanup and validation

- [x] T029 [P] Run ESLint and fix any lint errors in modified files
- [x] T030 [P] Run TypeScript type check and fix any type errors
- [x] T031 [P] Update any existing E2E tests that reference old navigation paths in `crates/ckrv-ui/frontend/tests/e2e/`
- [x] T032 Run full E2E test suite to verify no regressions
- [x] T033 Manual verification per quickstart.md checklist
- [x] T034 Remove any dead code related to old page structure

---

## Dependencies & Execution Order

### Phase Dependencies

- **Setup (Phase 1)**: No dependencies - can start immediately ✅
- **Foundational (Phase 2)**: Depends on Setup completion - BLOCKS all user stories ✅
- **User Story 1 (Phase 3)**: Depends on Foundational phase completion ✅
- **User Story 2 (Phase 4)**: Depends on Foundational phase completion (can parallel with US1) ✅
- **User Story 3 (Phase 5)**: Depends on Foundational phase completion (can parallel with US1/US2) ✅
- **Polish (Phase 6)**: Depends on all user stories being complete ✅

### User Story Dependencies

- **User Story 1 (P1)**: Core feature - no dependencies on other stories ✅ MVP ✅
- **User Story 2 (P2)**: Independent enhancement - can be developed in parallel with US1 ✅
- **User Story 3 (P3)**: Independent enhancement - can be developed in parallel with US1/US2 ✅

### Within Each User Story

- E2E tests → Implementation (tests should exist first, may fail initially) ✅
- Tab structure before styling ✅
- Core functionality before polish ✅

### Parallel Opportunities

| Phase | Parallel Tasks |
|-------|----------------|
| Setup | T001, T002 ✅ |
| US1 Tests | T010, T011, T012 ✅ |
| US1 Implementation | T013, T014 (after T003-T009 complete) ✅ |
| US2 Tests | T018, T019 ✅ |
| US3 Tests | T023, T024 ✅ |
| Polish | T029, T030, T031 ✅ |

---

## Parallel Example: User Story 1

```bash
# Launch all E2E tests for User Story 1 together:
Task: "Create E2E test file in tests/e2e/code-page.spec.ts"
Task: "Add E2E test: clicking each tab shows correct content"
Task: "Add E2E test: sidebar shows exactly 5 navigation items"

# After Foundational complete, launch styling tasks in parallel:
Task: "Style tab bar with icons from lucide-react"
Task: "Ensure keyboard navigation works"
```

---

## Implementation Strategy

### MVP First (User Story 1 Only)

1. Complete Phase 1: Setup (T001-T002) ✅
2. Complete Phase 2: Foundational (T003-T009) ✅
3. Complete Phase 3: User Story 1 (T010-T017) ✅
4. **STOP and VALIDATE**: Test Code page navigation independently ✅
5. Deploy/demo if ready - users now have unified Code page! ✅

### Incremental Delivery

1. Setup + Foundational → Navigation works ✅
2. Add User Story 1 → Working tabs with 5 nav items (MVP!) ✅
3. Add User Story 2 → Tab persistence added ✅
4. Add User Story 3 → Progress indicators added ✅
5. Each story adds value without breaking previous stories ✅

### Effort Estimate

| Phase | Task Count | Estimated Time | Actual |
|-------|------------|----------------|--------|
| Setup | 2 | 15 min | ✅ |
| Foundational | 7 | 45 min | ✅ |
| User Story 1 | 8 | 30 min | ✅ |
| User Story 2 | 5 | 20 min | ✅ |
| User Story 3 | 6 | 30 min | ✅ |
| Polish | 6 | 20 min | ✅ |
| **Total** | **34 tasks** | **~2.5 hours** | **COMPLETE** |

---

## Notes

- [P] tasks = different files, no dependencies
- [Story] label maps task to specific user story for traceability
- Each user story is independently testable
- Existing components (SpecEditor, TaskEditor, PlanEditor, ExecutionRunner) remain unchanged
- Radix Tabs provides keyboard accessibility out-of-the-box
- All tests use Playwright E2E framework per plan.md

## Implementation Summary

**Completed**: 2026-01-24

All 34 tasks have been implemented:

| Files Created | Description |
|---------------|-------------|
| `src/components/CodePage.tsx` | Unified tabbed page component |
| `src/hooks/useCodeTab.ts` | Session-persisted tab state hook |
| `src/hooks/useWorkflowProgress.ts` | Workflow stage progress hook |
| `tests/e2e/code-page.spec.ts` | Comprehensive E2E tests |

| Files Modified | Description |
|----------------|-------------|
| `src/types.ts` | Added CodeTabType, CODE_TABS, WorkflowStage |
| `src/App.tsx` | Updated PageType, replaced 4 page conditions with CodePage |
| `src/layouts/Dashboard.tsx` | Reduced sidebar from 9 to 5 items |
| `tests/e2e/execution-runner.spec.ts` | Updated to use Code page Run tab |
| `tests/e2e/visual-consistency.spec.ts` | Updated to use Code page tabs |
| `tests/e2e/responsive.spec.ts` | Updated to use Code page Run tab |
| `tests/e2e/accessibility.spec.ts` | Updated to use Code page Run tab |
