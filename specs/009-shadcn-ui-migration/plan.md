# Implementation Plan: shadcn/ui Migration

**Branch**: `009-shadcn-ui-migration` | **Date**: 2026-01-11 | **Spec**: [spec.md](./spec.md)  
**Input**: Feature specification from `/specs/009-shadcn-ui-migration/spec.md`

## Summary

Migrate the Chakravarti UI frontend from custom Tailwind CSS components to shadcn/ui for a polished, accessible, production-ready customer-facing interface. This involves:

1. Installing and configuring shadcn/ui with Tailwind CSS v4
2. Installing 15 shadcn components (button, card, badge, dialog, etc.)
3. Migrating 16 component files (~340KB of React code)
4. Removing the unused `@radix-ui/themes` dependency
5. Preserving custom visualizations (DAG view, progress ring, xterm terminal)

---

## Technical Context

**Language/Version**: TypeScript 5.9.3, React 19.2.0  
**Primary Dependencies**: Vite 7.2.4, Tailwind CSS 4.1.18, shadcn/ui (latest), @tanstack/react-query, lucide-react  
**Storage**: N/A (frontend-only)  
**Testing**: Playwright (E2E), Vitest (if added)  
**Target Platform**: Modern browsers (Chrome, Firefox, Safari, Edge)  
**Project Type**: Web frontend (React SPA served by Rust backend)  
**Performance Goals**: <100ms initial render after hydration, 60fps animations  
**Constraints**: Must preserve existing functionality, no breaking changes to E2E tests  
**Scale/Scope**: 16 component files, ~7,000 lines of TSX code

---

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

| Principle | Requirement | Status |
|-----------|-------------|--------|
| I. Code Quality Excellence | Full typing, zero lint errors, single responsibility | ✅ shadcn components are TypeScript-first with proper types |
| II. Testing Standards | TDD approach planned, coverage targets defined | ✅ E2E tests preserved; visual verification during migration |
| III. Reliability First | Error handling strategy, idempotency considered | ✅ No backend changes; component-by-component migration allows rollback |
| IV. Security by Default | No hardcoded secrets, input validation planned | ✅ N/A for UI component migration |
| V. Deterministic CLI Behavior | Machine-readable output, explicit exit codes | ✅ N/A for frontend changes |

---

## Project Structure

### Documentation (this feature)

```text
specs/009-shadcn-ui-migration/
├── spec.md              # Feature specification
├── plan.md              # This file
├── research.md          # Phase 0 output - technology research
├── data-model.md        # Phase 1 output - component hierarchy
├── quickstart.md        # Phase 1 output - installation guide
├── contracts/           # Phase 1 output - component interfaces
│   └── components.md    # TypeScript interfaces for shadcn components
├── checklists/
│   └── requirements.md  # Specification quality checklist
└── tasks.md             # Phase 2 output (created by /speckit.tasks)
```

### Source Code (repository root)

```text
crates/ckrv-ui/frontend/
├── src/
│   ├── components/
│   │   ├── ui/                    # NEW: shadcn/ui components
│   │   │   ├── button.tsx
│   │   │   ├── card.tsx
│   │   │   ├── badge.tsx
│   │   │   ├── dialog.tsx
│   │   │   ├── input.tsx
│   │   │   ├── select.tsx
│   │   │   ├── tabs.tsx
│   │   │   ├── tooltip.tsx
│   │   │   ├── collapsible.tsx
│   │   │   ├── scroll-area.tsx
│   │   │   ├── dropdown-menu.tsx
│   │   │   ├── progress.tsx
│   │   │   ├── skeleton.tsx
│   │   │   ├── alert.tsx
│   │   │   └── separator.tsx
│   │   ├── AgentManager.tsx       # Migrated
│   │   ├── AgentCliModal.tsx      # Migrated
│   │   ├── CommandPalette.tsx     # Migrated
│   │   ├── CompletionSummary.tsx  # Migrated
│   │   ├── DiffViewer.tsx         # Migrated
│   │   ├── ErrorBoundary.tsx      # Migrated
│   │   ├── ExecutionRunner.tsx    # Migrated
│   │   ├── LogTerminal.tsx        # Partially migrated (wrapper)
│   │   ├── LogViewer.tsx          # Migrated
│   │   ├── PlanEditor.tsx         # Migrated
│   │   ├── RunHistoryPanel.tsx    # Migrated
│   │   ├── SpecEditor.tsx         # Migrated
│   │   ├── StatusWidget.tsx       # Migrated
│   │   ├── TaskDetailModal.tsx    # Migrated
│   │   ├── TaskEditor.tsx         # Migrated
│   │   └── WorkflowPanel.tsx      # Migrated
│   ├── layouts/
│   │   └── Dashboard.tsx          # Migrated
│   ├── lib/
│   │   └── utils.ts               # NEW: cn() utility
│   ├── index.css                  # Updated with shadcn variables
│   └── App.tsx                    # Theme provider removed
├── components.json                 # NEW: shadcn configuration
├── tsconfig.app.json              # Updated with path aliases
├── vite.config.ts                 # Updated with path aliases
└── package.json                   # Dependencies updated
```

**Structure Decision**: This is a frontend migration within the existing project structure. All changes are contained within `crates/ckrv-ui/frontend/`. The new `components/ui/` directory follows shadcn/ui conventions.

---

## Migration Phases

### Phase 1: Foundation Setup
**Effort**: ~2 hours  
**Files Changed**: 5

1. Install dependencies (clsx, tailwind-merge, tw-animate-css)
2. Configure path aliases (tsconfig, vite.config)
3. Initialize shadcn/ui
4. Create `lib/utils.ts`
5. Update `index.css` with shadcn variables

### Phase 2: Core Components (P1)
**Effort**: ~4 hours  
**Files Changed**: 4

1. Install Button, Card, Badge, Tooltip components
2. Migrate `Dashboard.tsx` (sidebar navigation)
3. Migrate `StatusWidget.tsx`
4. Migrate `CommandPalette.tsx`
5. Remove `LoadingButton.tsx` (replaced by Button + loading pattern)

### Phase 3: Form Components
**Effort**: ~2 hours  
**Files Changed**: 2

1. Install Input, Select components
2. Create patterns for form fields
3. Update modal form patterns preparation

### Phase 4: Modal Components (P2)
**Effort**: ~4 hours  
**Files Changed**: 3

1. Install Dialog, DropdownMenu components
2. Migrate `AgentManager.tsx` (complex with modal)
3. Migrate `AgentCliModal.tsx`
4. Migrate `TaskDetailModal.tsx`

### Phase 5: Editor Pages (P2)
**Effort**: ~6 hours  
**Files Changed**: 3

1. Install Tabs, Collapsible, Separator, ScrollArea components
2. Migrate `SpecEditor.tsx`
3. Migrate `TaskEditor.tsx`
4. Migrate `PlanEditor.tsx`

### Phase 6: Execution & Viewer Pages (P3)
**Effort**: ~6 hours  
**Files Changed**: 6

1. Install Progress, Skeleton, Alert components
2. Migrate `ExecutionRunner.tsx`
3. Migrate `WorkflowPanel.tsx`
4. Migrate `LogViewer.tsx`
5. Migrate `DiffViewer.tsx`
6. Update `LogTerminal.tsx` (wrapper only)
7. Migrate `CompletionSummary.tsx`
8. Migrate `RunHistoryPanel.tsx`
9. Migrate `ErrorBoundary.tsx`

### Phase 7: Cleanup
**Effort**: ~1 hour  
**Files Changed**: 2

1. Update `App.tsx` (remove Theme wrapper)
2. Remove `@radix-ui/themes` dependency
3. Final verification and testing

---

## Component Migration Priority

| Priority | Component | Effort | Dependencies |
|----------|-----------|--------|--------------|
| **P1** | Dashboard.tsx | Medium | Button, Tooltip, Badge |
| **P1** | StatusWidget.tsx | Low | Card, Badge, Tooltip |
| **P1** | CommandPalette.tsx | High | Card, Button, Badge, Skeleton |
| **P2** | AgentManager.tsx | High | Card, Button, Badge, Dialog, Input, Select, Tabs, Collapsible |
| **P2** | SpecEditor.tsx | High | Card, Tabs, Collapsible, Input, Badge |
| **P2** | TaskEditor.tsx | High | Card, Tabs, Badge, Collapsible, Select |
| **P2** | PlanEditor.tsx | Medium | Card, Tabs, Badge, Select |
| **P3** | ExecutionRunner.tsx | Very High | Card, Button, Badge, Progress, ScrollArea, Skeleton |
| **P3** | TaskDetailModal.tsx | High | Dialog, Button, Badge, Select, Tabs |
| **P3** | WorkflowPanel.tsx | Medium | Card, Badge |
| **P3** | LogViewer.tsx | Medium | Card, Button, ScrollArea |
| **P3** | DiffViewer.tsx | Medium | Card, Select, Collapsible |
| **P3** | AgentCliModal.tsx | Low | Dialog, Button |
| **P3** | CompletionSummary.tsx | Low | Card, Badge |
| **P3** | RunHistoryPanel.tsx | Low | Card, Badge, ScrollArea |
| **P3** | ErrorBoundary.tsx | Low | Alert |
| **P3** | LogTerminal.tsx | Low | Card (wrapper only) |

---

## Risk Mitigation

| Risk | Impact | Mitigation |
|------|--------|------------|
| Tailwind v4 incompatibility | High | Research confirmed compatibility; follow upgrade guide |
| React 19 issues | Medium | Components updated for React 19; forwardRef removed |
| Broken E2E tests | Medium | Run tests after each phase; component-by-component approach |
| Style regressions | Low | Visual verification step after each migration |
| Bundle size increase | Low | Tree-shaking; only install needed components |

---

## Complexity Tracking

> No Constitution violations identified. This is a frontend-only migration with no backend changes.

---

## Success Metrics

- [ ] All 15 shadcn components installed
- [ ] All 16 component files migrated
- [ ] `@radix-ui/themes` removed
- [ ] E2E tests pass
- [ ] Visual consistency verified across all pages
- [ ] Focus indicators visible on all interactive elements
- [ ] Keyboard navigation works (Escape closes modals, Tab focuses elements)

---

## Generated Artifacts

| Artifact | Path | Purpose |
|----------|------|---------|
| Research | `research.md` | Technology decisions and rationale |
| Data Model | `data-model.md` | Component hierarchy and token mapping |
| Contracts | `contracts/components.md` | TypeScript interfaces for components |
| Quickstart | `quickstart.md` | Step-by-step installation guide |
