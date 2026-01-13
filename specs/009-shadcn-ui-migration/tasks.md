# Tasks: shadcn/ui Migration

**Input**: Design documents from `/specs/009-shadcn-ui-migration/`  
**Prerequisites**: plan.md (required), spec.md (required), research.md, data-model.md, contracts/, quickstart.md

**Tests**: No explicit test tasks requested. E2E tests exist and will be verified post-migration.

**Organization**: Tasks are grouped by user story to enable independent implementation and testing of each story.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel (different files, no dependencies)
- **[Story]**: Which user story this task belongs to (e.g., US1, US2, US3)
- Include exact file paths in descriptions

## Path Conventions

All paths are relative to `crates/ckrv-ui/frontend/`

---

## Phase 1: Setup (Shared Infrastructure)

**Purpose**: Project initialization, shadcn/ui setup, and configuration

- [x] T001 Install dependencies: `npm install clsx tailwind-merge tw-animate-css` in crates/ckrv-ui/frontend/
- [x] T002 Update tsconfig.app.json with path alias: `"baseUrl": "."` and `"paths": { "@/*": ["./src/*"] }` in crates/ckrv-ui/frontend/tsconfig.app.json
- [x] T003 Update vite.config.ts with path alias: add `resolve.alias` for `@` → `./src` in crates/ckrv-ui/frontend/vite.config.ts
- [x] T004 Initialize shadcn/ui: run `npx shadcn@latest init` with New York style, slate base, CSS vars enabled in crates/ckrv-ui/frontend/
- [x] T005 Create cn() utility function in crates/ckrv-ui/frontend/src/lib/utils.ts
- [x] T006 Update index.css with shadcn CSS variables mapping to existing design tokens in crates/ckrv-ui/frontend/src/index.css

---

## Phase 2: Foundational (Blocking Prerequisites)

**Purpose**: Install all shadcn components needed across user stories - MUST complete before any component migration

**⚠️ CRITICAL**: No user story work can begin until this phase is complete

- [x] T007 [P] Install shadcn button component: `npx shadcn@latest add button` in crates/ckrv-ui/frontend/
- [x] T008 [P] Install shadcn card component: `npx shadcn@latest add card` in crates/ckrv-ui/frontend/
- [x] T009 [P] Install shadcn badge component: `npx shadcn@latest add badge` in crates/ckrv-ui/frontend/
- [x] T010 [P] Install shadcn tooltip component: `npx shadcn@latest add tooltip` in crates/ckrv-ui/frontend/
- [x] T011 [P] Install shadcn input component: `npx shadcn@latest add input` in crates/ckrv-ui/frontend/
- [x] T012 [P] Install shadcn select component: `npx shadcn@latest add select` in crates/ckrv-ui/frontend/
- [x] T013 [P] Install shadcn dialog component: `npx shadcn@latest add dialog` in crates/ckrv-ui/frontend/
- [x] T014 [P] Install shadcn tabs component: `npx shadcn@latest add tabs` in crates/ckrv-ui/frontend/
- [x] T015 [P] Install shadcn collapsible component: `npx shadcn@latest add collapsible` in crates/ckrv-ui/frontend/
- [x] T016 [P] Install shadcn scroll-area component: `npx shadcn@latest add scroll-area` in crates/ckrv-ui/frontend/
- [x] T017 [P] Install shadcn dropdown-menu component: `npx shadcn@latest add dropdown-menu` in crates/ckrv-ui/frontend/
- [x] T018 [P] Install shadcn progress component: `npx shadcn@latest add progress` in crates/ckrv-ui/frontend/
- [x] T019 [P] Install shadcn skeleton component: `npx shadcn@latest add skeleton` in crates/ckrv-ui/frontend/
- [x] T020 [P] Install shadcn alert component: `npx shadcn@latest add alert` in crates/ckrv-ui/frontend/
- [x] T021 [P] Install shadcn separator component: `npx shadcn@latest add separator` in crates/ckrv-ui/frontend/
- [x] T022 Add custom badge variants (success, warning, info) to crates/ckrv-ui/frontend/src/components/ui/badge.tsx
- [x] T023 Create TooltipProvider wrapper in App.tsx for global tooltip support in crates/ckrv-ui/frontend/src/App.tsx

**Checkpoint**: All shadcn components installed and configured - user story implementation can now begin

---

## Phase 3: User Story 1 - Core Navigation & Layout Polish (Priority: P1) 🎯 MVP

**Goal**: Migrate Dashboard layout, sidebar navigation, and status indicators to shadcn components

**Independent Test**: Navigate through all pages and verify consistent sidebar styling, smooth hover/active states, and proper status indicator updates

### Implementation for User Story 1

- [x] T024 [US1] Migrate NavIcon component to use shadcn Button variant="ghost" + Tooltip in crates/ckrv-ui/frontend/src/layouts/Dashboard.tsx
- [x] T025 [US1] Migrate ConnectionIndicator to use shadcn Badge + Tooltip in crates/ckrv-ui/frontend/src/layouts/Dashboard.tsx
- [x] T026 [US1] Migrate DockerIndicator to use shadcn Badge + Tooltip in crates/ckrv-ui/frontend/src/layouts/Dashboard.tsx
- [x] T027 [US1] Migrate CloudIndicator to use shadcn Badge + Tooltip in crates/ckrv-ui/frontend/src/layouts/Dashboard.tsx
- [x] T028 [US1] Update sidebar layout spacing to use shadcn design tokens in crates/ckrv-ui/frontend/src/layouts/Dashboard.tsx
- [x] T029 [US1] Verify keyboard navigation works for sidebar icons (tab focus) in crates/ckrv-ui/frontend/src/layouts/Dashboard.tsx

**Checkpoint**: Dashboard layout fully migrated - navigation and status indicators use shadcn components

---

## Phase 4: User Story 2 - Form Inputs & Buttons Consistency (Priority: P1)

**Goal**: Migrate all buttons and form inputs to shadcn components for consistent interaction patterns

**Independent Test**: Click all buttons, interact with all form fields, verify consistent focus states and loading behaviors

### Implementation for User Story 2

- [x] T030 [US2] Delete LoadingButton.tsx and replace usages with shadcn Button + loading pattern in crates/ckrv-ui/frontend/src/components/ui/LoadingButton.tsx
- [x] T031 [US2] Migrate StatusWidget to use shadcn Card and Badge in crates/ckrv-ui/frontend/src/components/StatusWidget.tsx
- [x] T032 [P] [US2] Migrate StatusRow component to use consistent spacing in crates/ckrv-ui/frontend/src/components/StatusWidget.tsx
- [x] T033 [US2] Migrate CommandPalette container to use shadcn Card in crates/ckrv-ui/frontend/src/components/CommandPalette.tsx
- [x] T034 [US2] Migrate CommandPalette buttons to use shadcn Button variants in crates/ckrv-ui/frontend/src/components/CommandPalette.tsx
- [x] T035 [US2] Migrate CommandPalette badges to use shadcn Badge in crates/ckrv-ui/frontend/src/components/CommandPalette.tsx
- [x] T036 [US2] Add Skeleton loading states to CommandPalette for spec/task loading in crates/ckrv-ui/frontend/src/components/CommandPalette.tsx

**Checkpoint**: All buttons and status widgets use shadcn components consistently

---

## Phase 5: User Story 3 - Modal & Dialog Interactions (Priority: P2)

**Goal**: Migrate all modals to shadcn Dialog with proper accessibility (Escape to close, focus trap)

**Independent Test**: Open each modal type, verify Escape closes, click outside closes, and keyboard navigation works within

### Implementation for User Story 3

- [x] T037 [US3] Migrate AgentModal to use shadcn Dialog, DialogHeader, DialogContent, DialogFooter in crates/ckrv-ui/frontend/src/components/AgentManager.tsx
- [x] T038 [US3] Migrate form inputs in AgentModal to use shadcn Input and Select in crates/ckrv-ui/frontend/src/components/AgentManager.tsx
- [x] T039 [US3] Migrate AgentCliModal to use shadcn Dialog in crates/ckrv-ui/frontend/src/components/AgentCliModal.tsx
- [x] T040 [US3] Wrap xterm terminal in shadcn Card within AgentCliModal in crates/ckrv-ui/frontend/src/components/AgentCliModal.tsx
- [x] T041 [US3] Verify Escape key closes all modals in crates/ckrv-ui/frontend/src/components/AgentManager.tsx and AgentCliModal.tsx

**Checkpoint**: All modals use shadcn Dialog with proper accessibility features

---

## Phase 6: User Story 4 - Cards & Data Display (Priority: P2)

**Goal**: Migrate all card components to shadcn Card for consistent styling

**Independent Test**: View agent cards, spec cards, task cards, batch cards - verify consistent hover, borders, and badge styling

### Implementation for User Story 4

- [x] T042 [US4] Migrate AgentCard to use shadcn Card, CardHeader, CardContent in crates/ckrv-ui/frontend/src/components/AgentManager.tsx
- [x] T043 [US4] Migrate agent type badges and status badges to shadcn Badge in crates/ckrv-ui/frontend/src/components/AgentManager.tsx
- [x] T044 [US4] Migrate AgentManager tabs (if any view toggles) to shadcn Tabs in crates/ckrv-ui/frontend/src/components/AgentManager.tsx
- [x] T045 [P] [US4] Migrate SpecEditor Section component to use shadcn Collapsible in crates/ckrv-ui/frontend/src/components/SpecEditor.tsx
- [x] T046 [P] [US4] Migrate SpecEditor ViewToggle to use shadcn Tabs in crates/ckrv-ui/frontend/src/components/SpecEditor.tsx
- [x] T047 [P] [US4] Migrate SpecEditor PriorityBadge to use shadcn Badge in crates/ckrv-ui/frontend/src/components/SpecEditor.tsx
- [x] T048 [P] [US4] Migrate SpecListView cards to use shadcn Card in crates/ckrv-ui/frontend/src/components/SpecEditor.tsx
- [x] T049 [P] [US4] Migrate TaskEditor TaskCard to use shadcn Card in crates/ckrv-ui/frontend/src/components/TaskEditor.tsx
- [x] T050 [P] [US4] Migrate TaskEditor badges (RiskBadge, ModelTierBadge, StatusBadge) to shadcn Badge in crates/ckrv-ui/frontend/src/components/TaskEditor.tsx
- [x] T051 [P] [US4] Migrate TaskEditor PhaseGroup to use shadcn Collapsible in crates/ckrv-ui/frontend/src/components/TaskEditor.tsx
- [x] T052 [P] [US4] Migrate TaskEditor ViewToggle to use shadcn Tabs in crates/ckrv-ui/frontend/src/components/TaskEditor.tsx
- [x] T053 [P] [US4] Migrate TaskEditor FilterBar selects to use shadcn Select in crates/ckrv-ui/frontend/src/components/TaskEditor.tsx
- [x] T054 [P] [US4] Migrate PlanEditor BatchCard to use shadcn Card in crates/ckrv-ui/frontend/src/components/PlanEditor.tsx
- [x] T055 [P] [US4] Migrate PlanEditor badges (ModelBadge, StrategyBadge) to shadcn Badge in crates/ckrv-ui/frontend/src/components/PlanEditor.tsx
- [x] T056 [P] [US4] Migrate PlanEditor view toggles to shadcn Tabs in crates/ckrv-ui/frontend/src/components/PlanEditor.tsx
- [x] T057 [P] [US4] Migrate WorkflowPanel PipelineStage to use shadcn Card in crates/ckrv-ui/frontend/src/components/WorkflowPanel.tsx
- [x] T058 [P] [US4] Migrate WorkflowPanel badges to shadcn Badge in crates/ckrv-ui/frontend/src/components/WorkflowPanel.tsx

**Checkpoint**: All cards and data display components use shadcn Card and Badge consistently

---

## Phase 7: User Story 5 - Execution & Real-time Feedback (Priority: P3)

**Goal**: Migrate ExecutionRunner and related real-time components to shadcn

**Independent Test**: Run an execution, verify progress updates, batch status changes, and log panels display correctly

### Implementation for User Story 5

- [x] T059 [US5] Migrate ExecutionRunner SpecListView to use shadcn Card in crates/ckrv-ui/frontend/src/components/ExecutionRunner.tsx
- [x] T060 [US5] Migrate ExecutionRunner header buttons to shadcn Button in crates/ckrv-ui/frontend/src/components/ExecutionRunner.tsx
- [x] T061 [US5] Migrate ExecutionRunner batch status badges to shadcn Badge in crates/ckrv-ui/frontend/src/components/ExecutionRunner.tsx
- [x] T062 [US5] Migrate BatchLogPanel container to use shadcn Card in crates/ckrv-ui/frontend/src/components/ExecutionRunner.tsx
- [x] T063 [US5] Migrate BatchLogPanel scroll container to use shadcn ScrollArea in crates/ckrv-ui/frontend/src/components/ExecutionRunner.tsx
- [x] T064 [US5] Update ProgressRing to use design token colors (keep SVG implementation) in crates/ckrv-ui/frontend/src/components/ExecutionRunner.tsx
- [x] T065 [US5] Migrate MergeBranchesPanel to use shadcn Card and Button in crates/ckrv-ui/frontend/src/components/ExecutionRunner.tsx
- [x] T066 [P] [US5] Migrate CompletionSummary to use shadcn Card and Badge in crates/ckrv-ui/frontend/src/components/CompletionSummary.tsx
- [x] T067 [P] [US5] Migrate RunHistoryPanel to use shadcn Card, Badge, ScrollArea in crates/ckrv-ui/frontend/src/components/RunHistoryPanel.tsx
- [x] T068 [P] [US5] Migrate TaskDetailModal to use shadcn Dialog in crates/ckrv-ui/frontend/src/components/TaskDetailModal.tsx
- [x] T069 [P] [US5] Migrate TaskDetailModal AgentSelector to use shadcn Select in crates/ckrv-ui/frontend/src/components/TaskDetailModal.tsx
- [x] T070 [P] [US5] Migrate TaskDetailModal badges to shadcn Badge in crates/ckrv-ui/frontend/src/components/TaskDetailModal.tsx
- [x] T071 [P] [US5] Wrap EmbeddedTerminal in shadcn Card in crates/ckrv-ui/frontend/src/components/TaskDetailModal.tsx

**Checkpoint**: ExecutionRunner and real-time feedback components fully migrated

---

## Phase 8: User Story 6 - Editors & Complex Views (Priority: P3)

**Goal**: Migrate LogViewer, DiffViewer, and remaining editor components

**Independent Test**: Use editors, view logs, compare diffs - verify collapsible sections, scroll areas, and select dropdowns work correctly

### Implementation for User Story 6

- [x] T072 [P] [US6] Migrate LogViewer container to use shadcn Card in crates/ckrv-ui/frontend/src/components/LogViewer.tsx
- [x] T073 [P] [US6] Migrate LogViewer toolbar buttons to shadcn Button variant="ghost" in crates/ckrv-ui/frontend/src/components/LogViewer.tsx
- [x] T074 [P] [US6] Migrate LogViewer scroll container to shadcn ScrollArea in crates/ckrv-ui/frontend/src/components/LogViewer.tsx
- [x] T075 [P] [US6] Migrate DiffViewer BranchSelector to use shadcn Select in crates/ckrv-ui/frontend/src/components/DiffViewer.tsx
- [x] T076 [P] [US6] Migrate DiffViewer FileDiffView to use shadcn Card + Collapsible in crates/ckrv-ui/frontend/src/components/DiffViewer.tsx
- [x] T077 [P] [US6] Migrate DiffViewer stats badges to shadcn Badge in crates/ckrv-ui/frontend/src/components/DiffViewer.tsx
- [x] T078 [P] [US6] Migrate ErrorBoundary error display to use shadcn Alert variant="destructive" in crates/ckrv-ui/frontend/src/components/ErrorBoundary.tsx
- [x] T079 [P] [US6] Wrap LogTerminal (xterm) in shadcn Card container in crates/ckrv-ui/frontend/src/components/LogTerminal.tsx

**Checkpoint**: All editors and complex views migrated to shadcn components

---

## Phase 9: Polish & Cleanup

**Purpose**: Remove old dependencies, verify consistency, finalize migration

- [x] T080 Update App.tsx to remove Theme wrapper from @radix-ui/themes in crates/ckrv-ui/frontend/src/App.tsx
- [x] T081 Remove @radix-ui/themes import and usage in crates/ckrv-ui/frontend/src/App.tsx
- [x] T082 Uninstall @radix-ui/themes: `npm uninstall @radix-ui/themes` in crates/ckrv-ui/frontend/
- [x] T083 [P] Verify no remaining @radix-ui/themes imports: `grep -r "@radix-ui/themes" src/` in crates/ckrv-ui/frontend/
- [x] T084 [P] Run type check: `npm run build` to verify no TypeScript errors in crates/ckrv-ui/frontend/
- [x] T085 [P] Run E2E tests to verify no functional regression in crates/ckrv-ui/frontend/
- [x] T086 Visual verification: navigate all pages and verify consistent styling

---

## Dependencies & Execution Order

### Phase Dependencies

- **Phase 1 (Setup)**: No dependencies - can start immediately
- **Phase 2 (Foundational)**: Depends on Phase 1 - BLOCKS all user stories
- **Phase 3-8 (User Stories)**: All depend on Phase 2 completion
  - User stories can proceed in priority order (P1 → P2 → P3)
  - Some tasks within phases marked [P] can run in parallel
- **Phase 9 (Polish)**: Depends on all user stories being complete

### User Story Dependencies

- **User Story 1 (P1)**: Can start after Phase 2 - No dependencies on other stories
- **User Story 2 (P1)**: Can start after Phase 2 - No dependencies on other stories
- **User Story 3 (P2)**: Can start after Phase 2 - No strict dependencies
- **User Story 4 (P2)**: Can start after Phase 2 - No strict dependencies
- **User Story 5 (P3)**: Can start after Phase 2 - No strict dependencies
- **User Story 6 (P3)**: Can start after Phase 2 - No strict dependencies

### Within Each User Story

- Card/layout migrations before badge migrations
- Container components before child components
- Core components before utility components

### Parallel Opportunities

- All component installations in Phase 2 can run in parallel
- SpecEditor, TaskEditor, PlanEditor, WorkflowPanel migrations can run in parallel (different files)
- LogViewer, DiffViewer, ErrorBoundary, LogTerminal migrations can run in parallel

---

## Implementation Strategy

### MVP First (User Stories 1 + 2)

1. Complete Phase 1: Setup
2. Complete Phase 2: Install all shadcn components
3. Complete Phase 3: Dashboard navigation (US1)
4. Complete Phase 4: Buttons and StatusWidget (US2)
5. **STOP and VALIDATE**: Test navigation and core interactions
6. Deploy/demo if ready

### Incremental Delivery

1. Setup + Foundational → All components available
2. User Story 1 + 2 → Core navigation and buttons polished (MVP!)
3. User Story 3 + 4 → Modals and cards polished
4. User Story 5 + 6 → Execution and editors polished
5. Cleanup → Remove old dependencies

---

## Summary

| Metric | Count |
|--------|-------|
| **Total Tasks** | 86 |
| **Phase 1 (Setup)** | 6 tasks |
| **Phase 2 (Foundational)** | 17 tasks |
| **User Story 1 (P1)** | 6 tasks |
| **User Story 2 (P1)** | 7 tasks |
| **User Story 3 (P2)** | 5 tasks |
| **User Story 4 (P2)** | 17 tasks |
| **User Story 5 (P3)** | 13 tasks |
| **User Story 6 (P3)** | 8 tasks |
| **Phase 9 (Polish)** | 7 tasks |
| **Parallel Opportunities** | 45 tasks marked [P] |

---

## Notes

- [P] tasks = different files, no dependencies
- [Story] label maps task to specific user story for traceability
- Each user story should be independently completable and testable
- Commit after each phase or logical group
- Stop at any checkpoint to validate independently
- Avoid: vague tasks, same file conflicts, cross-story dependencies
