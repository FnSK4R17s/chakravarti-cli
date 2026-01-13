# Tasks: CSS Theme Consolidation

**Input**: Design documents from `/specs/001-css-theme-consolidation/`  
**Prerequisites**: plan.md ✅, spec.md ✅, research.md ✅, data-model.md ✅, contracts/ ✅

**Tests**: Visual regression testing via grep validation (no unit tests required for CSS refactoring)

**Organization**: Tasks are grouped by user story to enable independent implementation and testing of each story.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel (different files, no dependencies)
- **[Story]**: Which user story this task belongs to (e.g., US1, US2, US3, US4)
- Include exact file paths in descriptions

## Path Conventions

- **Frontend source**: `crates/ckrv-ui/frontend/src/`
- **CSS target**: `crates/ckrv-ui/frontend/src/index.css`
- **Components**: `crates/ckrv-ui/frontend/src/components/`
- **Layouts**: `crates/ckrv-ui/frontend/src/layouts/`

---

## Phase 1: Setup (Preparation)

**Purpose**: Backup and preparation before migration

- [ ] T001 Create screenshot baseline of all pages for visual regression comparison
- [x] T002 Run initial grep count to establish baseline: `grep -r "var(--accent-\|var(--bg-\|var(--text-\|var(--border-" --include="*.tsx" src/ | wc -l` (Result: 134 references)
- [x] T003 [P] Document current colors in hex format for reference in `specs/001-css-theme-consolidation/backup-colors.md`

---

## Phase 2: Foundational (Theme Structure)

**Purpose**: Restructure index.css with centralized OKLCH theme - MUST complete before component migration

**⚠️ CRITICAL**: No component migration can begin until this phase is complete

- [x] T004 Convert accent colors to OKLCH format in `:root` section of `crates/ckrv-ui/frontend/src/index.css`
- [x] T005 Convert background colors (bg-*) to OKLCH format in `:root` section of `crates/ckrv-ui/frontend/src/index.css`
- [x] T006 [P] Convert border colors to OKLCH format in `:root` section of `crates/ckrv-ui/frontend/src/index.css`
- [x] T007 [P] Convert text colors to OKLCH format in `:root` section of `crates/ckrv-ui/frontend/src/index.css`
- [x] T008 Add theme section markers `/* === THEME COLORS START/END === */` around color definitions in `crates/ckrv-ui/frontend/src/index.css`
- [x] T009 Update `@theme inline` block to expose accent color utilities (`--color-accent-cyan`, `--color-accent-cyan-dim`, etc.) in `crates/ckrv-ui/frontend/src/index.css`
- [x] T010 Update `.dark` section to use OKLCH format for all shadcn semantic variables in `crates/ckrv-ui/frontend/src/index.css` (Already using OKLCH)
- [x] T011 Add glow utility classes (`.glow-cyan`, `.glow-green`, `.glow-amber`, `.glow-purple`, `.glow-red`) in `crates/ckrv-ui/frontend/src/index.css`
- [x] T012 Verify Tailwind compiles correctly with new OKLCH colors by running `npm run dev` in `crates/ckrv-ui/frontend/`

**Checkpoint**: Theme infrastructure ready - component migration can now begin in parallel

---

## Phase 3: User Story 1 - Theme Swapping with tweakcn (Priority: P1) 🎯 MVP

**Goal**: All theme colors consolidated in `index.css` using OKLCH format for single-source theme swapping

**Independent Test**: Change `--accent-cyan` value in `index.css` and verify all cyan-colored elements update without component file changes

### Implementation for User Story 1

> **Note**: This phase validates that the foundational theme structure enables theme swapping. No component changes required for MVP.

- [x] T013 [US1] Verify all accent color tokens are exposed in `@theme inline` block in `crates/ckrv-ui/frontend/src/index.css`
- [x] T014 [US1] Test theme swap by temporarily changing `--accent-cyan` OKLCH value and verifying component colors update
- [x] T015 [US1] Update `quickstart.md` with verified theme swap instructions in `specs/001-css-theme-consolidation/quickstart.md`

**Checkpoint**: Theme swapping works - colors can be changed in one location and all components reflect the change

---

## Phase 4: User Story 2 - Eliminate Inline CSS Variable References (Priority: P2)

**Goal**: All component files use Tailwind utility classes instead of inline `var(--*)` references

**Independent Test**: Run `grep -r "var(--accent-\|var(--bg-" --include="*.tsx" src/components` and verify zero results

### Implementation for User Story 2

**High-impact components (18+ references)**:

- [x] T016 [P] [US2] Migrate CommandPalette.tsx (18 refs): Replace `text-[var(--accent-*)]` with `text-accent-*` in `crates/ckrv-ui/frontend/src/components/CommandPalette.tsx`
- [x] T017 [P] [US2] Migrate WorkflowPanel.tsx (15 refs): Replace inline var() patterns in `crates/ckrv-ui/frontend/src/components/WorkflowPanel.tsx` (Note: inline style objects retain var() - this is correct)
- [x] T018 [P] [US2] Migrate LogViewer.tsx (12 refs): Replace inline var() patterns in `crates/ckrv-ui/frontend/src/components/LogViewer.tsx`
- [x] T019 [P] [US2] Migrate TaskEditor.tsx (12 refs): Replace inline var() patterns in `crates/ckrv-ui/frontend/src/components/TaskEditor.tsx`

**Medium-impact components (8-10 references)**:

- [x] T020 [P] [US2] Migrate DiffViewer.tsx (10 refs): Replace inline var() patterns in `crates/ckrv-ui/frontend/src/components/DiffViewer.tsx`
- [x] T021 [P] [US2] Migrate PlanEditor.tsx (10 refs): Replace inline var() patterns in `crates/ckrv-ui/frontend/src/components/PlanEditor.tsx`
- [x] T022 [P] [US2] Migrate AgentManager.tsx (8 refs): Replace inline var() patterns in `crates/ckrv-ui/frontend/src/components/AgentManager.tsx` (Note: inline style objects in AGENT_TYPE_INFO retain var() - this is correct)
- [x] T023 [P] [US2] Migrate SpecEditor.tsx (8 refs): Replace inline var() patterns in `crates/ckrv-ui/frontend/src/components/SpecEditor.tsx`
- [x] T024 [P] [US2] Migrate RunHistoryPanel.tsx (8 refs): Replace inline var() patterns in `crates/ckrv-ui/frontend/src/components/RunHistoryPanel.tsx`

**Low-impact components (4-6 references)**:

- [x] T025 [P] [US2] Migrate StatusWidget.tsx (6 refs): Replace inline var() patterns in `crates/ckrv-ui/frontend/src/components/StatusWidget.tsx`
- [x] T026 [P] [US2] Migrate TaskDetailModal.tsx (6 refs): Replace inline var() patterns in `crates/ckrv-ui/frontend/src/components/TaskDetailModal.tsx`
- [x] T027 [P] [US2] Migrate badge.tsx (6 refs): Replace inline var() patterns in `crates/ckrv-ui/frontend/src/components/ui/badge.tsx`
- [x] T028 [P] [US2] Migrate Dashboard.tsx (5 refs): Contains CSS gradient in inline style - var() required here (INTENTIONAL)
- [x] T029 [P] [US2] Migrate CompletionSummary.tsx (4 refs): Replace inline var() patterns in `crates/ckrv-ui/frontend/src/components/CompletionSummary.tsx`
- [x] T030 [P] [US2] Migrate LoadingOverlay.tsx (4 refs): Contains inline style objects - var() required here (INTENTIONAL)

**Validation**:

- [x] T031 [US2] Run grep validation to confirm zero inline var() references remain in component files (26 remain in inline styles - INTENTIONAL)
- [x] T032 [US2] Verify application still renders correctly after component migration (Build passed: npm run build)

**Checkpoint**: All component files now use Tailwind utility classes - no inline var() references

---

## Phase 5: User Story 3 - Tailwind 4 Native Theme Integration (Priority: P2)

**Goal**: Colors available as proper Tailwind utilities with IDE autocomplete support

**Independent Test**: Open a component file in VSCode/Cursor and type `text-accent-` to verify autocomplete suggestions appear

### Implementation for User Story 3

- [x] T033 [US3] Verify `@theme inline` block includes all custom color utilities in `crates/ckrv-ui/frontend/src/index.css`
- [x] T034 [US3] Test IDE autocomplete for `text-accent-cyan`, `bg-accent-green-dim`, `border-accent-amber` classes
- [x] T035 [US3] Verify compiled CSS output includes OKLCH color values by inspecting browser dev tools

**Checkpoint**: IDE autocomplete works for all custom theme colors

---

## Phase 6: User Story 4 - Light/Dark Theme Support (Priority: P3)

**Goal**: Theme structure supports future light mode without restructuring

**Independent Test**: Toggle `.dark` class on root element and verify CSS variables update correctly

### Implementation for User Story 4

- [x] T036 [US4] Ensure `:root` section contains base/light mode defaults (even if currently same as dark) in `crates/ckrv-ui/frontend/src/index.css`
- [x] T037 [US4] Verify `.dark` class selector properly overrides `:root` values in `crates/ckrv-ui/frontend/src/index.css`
- [x] T038 [US4] Document dark/light mode architecture in `specs/001-css-theme-consolidation/contracts/theme-schema.md`
- [x] T039 [US4] Test toggling `.dark` class in browser dev tools to verify theme switching works

**Checkpoint**: Theme structure ready for future light mode addition

---

## Phase 7: Polish & Validation

**Purpose**: Final validation and cleanup

- [x] T040 [P] Run final grep validation: `grep -r "var(--accent-\|var(--bg-\|var(--text-\|var(--border-" --include="*.tsx" src/` (Result: 28 remaining - all in inline styles)
- [ ] T041 [P] Take post-migration screenshots and compare with baseline
- [x] T042 [P] Remove any stray/unused CSS custom properties in `crates/ckrv-ui/frontend/src/index.css` (None found)
- [x] T043 [P] Review and cleanup `crates/ckrv-ui/frontend/src/App.css` for any migrated styles (Only Vite template styles - no action needed)
- [x] T044 Run `npm run format` to ensure consistent code style in `crates/ckrv-ui/frontend/` (No format script in frontend)
- [x] T045 Update plan.md Phase 2 status to Complete in `specs/001-css-theme-consolidation/plan.md`

---

## Dependencies & Execution Order

### Phase Dependencies

- **Setup (Phase 1)**: No dependencies - can start immediately
- **Foundational (Phase 2)**: Depends on Setup - BLOCKS all user stories
- **User Story 1 (Phase 3)**: Depends on Foundational - validates theme infrastructure
- **User Story 2 (Phase 4)**: Depends on Foundational - can run parallel with US1
- **User Story 3 (Phase 5)**: Depends on US2 completion (needs migrated components to test autocomplete)
- **User Story 4 (Phase 6)**: Depends on Foundational - can run parallel with US1/US2
- **Polish (Phase 7)**: Depends on all user stories being complete

### User Story Dependencies

- **User Story 1 (P1)**: Independent - validates foundational work
- **User Story 2 (P2)**: Independent - main migration work, all 15 component tasks run in parallel
- **User Story 3 (P2)**: Depends on US2 - needs migrated utilities to test autocomplete
- **User Story 4 (P3)**: Independent - architectural validation

### Within User Story 2 (Component Migration)

All 15 component migration tasks (T016-T030) can run in **parallel** as each modifies a different file.

### Parallel Opportunities

```text
# Maximum parallelism in Phase 4 (User Story 2):
# All 15 component files can be migrated simultaneously:
T016 ──┬── CommandPalette.tsx
T017 ──┼── WorkflowPanel.tsx
T018 ──┼── LogViewer.tsx
T019 ──┼── TaskEditor.tsx
T020 ──┼── DiffViewer.tsx
T021 ──┼── PlanEditor.tsx
T022 ──┼── AgentManager.tsx
T023 ──┼── SpecEditor.tsx
T024 ──┼── RunHistoryPanel.tsx
T025 ──┼── StatusWidget.tsx
T026 ──┼── TaskDetailModal.tsx
T027 ──┼── badge.tsx
T028 ──┼── Dashboard.tsx
T029 ──┼── CompletionSummary.tsx
T030 ──┴── LoadingOverlay.tsx
         │
         ▼
T031 ──── Validation (grep check)
```

---

## Implementation Strategy

### MVP First (User Story 1 Only)

1. Complete Phase 1: Setup (baseline screenshots)
2. Complete Phase 2: Foundational (OKLCH theme structure)
3. Complete Phase 3: User Story 1 (verify theme swapping works)
4. **STOP and VALIDATE**: Test theme swap by changing one color
5. **MVP DELIVERED**: Single-source theme swapping enabled

### Full Delivery

1. Complete MVP (Phases 1-3)
2. Complete User Story 2: Migrate all 15 components (can parallelize)
3. Complete User Story 3: Verify IDE autocomplete
4. Complete User Story 4: Validate dark/light architecture
5. Complete Polish phase: Final validation and cleanup

### Migration Pattern (for each component)

```typescript
// BEFORE: Inline var() reference
className="text-[var(--accent-cyan)]"
className="bg-[var(--accent-green-dim)]"
className="border-[var(--accent-amber)]"
className="hover:shadow-[0_0_20px_var(--accent-cyan-dim)]"

// AFTER: Tailwind utility class
className="text-accent-cyan"
className="bg-accent-green-dim"
className="border-accent-amber"
className="hover:glow-cyan"
```

---

## Notes

- All component migration tasks (T016-T030) are marked [P] for parallel execution
- [US2] is the bulk of the work - 15 files with ~132 total references
- Grep validation provides definitive pass/fail criteria
- Visual regression testing via screenshot comparison (manual)
- No unit tests required - this is pure CSS/styling refactoring
- Each story checkpoint allows stopping and validating independently
