# Implementation Plan: Unified Code Page

**Branch**: `015-unified-code-page` | **Date**: 2026-01-24 | **Spec**: [spec.md](./spec.md)
**Input**: Feature specification from `/specs/015-unified-code-page/spec.md`

## Summary

Consolidate 4 separate navigation pages (Specs, Tasks, Plan, Runner) into a single "Code" page with a tabbed interface. This reduces navigation complexity from 9 sidebar items to 5 while maintaining all existing functionality. The existing components (SpecEditor, TaskEditor, PlanEditor, ExecutionRunner) will be rendered as tab content without modification.

## Technical Context

**Language/Version**: TypeScript 5.9, React 19  
**Primary Dependencies**: React, Radix UI (Tabs), TanStack Query, Tailwind CSS 4  
**Storage**: N/A (frontend only, uses existing API endpoints)  
**Testing**: Playwright E2E tests  
**Target Platform**: Web browser (React SPA served by Axum backend)  
**Project Type**: Web application (monorepo: Rust backend + React frontend)  
**Performance Goals**: Tab switching < 100ms (no full page reload)  
**Constraints**: Must preserve all existing component functionality  
**Scale/Scope**: 4 existing components consolidated into 1 container

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

| Principle | Requirement | Status |
|-----------|-------------|--------|
| I. Code Quality Excellence | Full typing, zero lint errors, single responsibility | ✅ TypeScript strict mode, existing components well-typed |
| II. Testing Standards | TDD approach planned, coverage targets defined | ✅ E2E tests will verify tab navigation |
| III. Reliability First | Error handling strategy, idempotency considered | ✅ ErrorBoundary already wraps pages, tabs are stateless |
| IV. Security by Default | No hardcoded secrets, input validation planned | ✅ N/A for frontend navigation change |
| V. Deterministic CLI Behavior | Machine-readable output, explicit exit codes | ✅ N/A for UI-only change |

## Project Structure

### Documentation (this feature)

```text
specs/015-unified-code-page/
├── plan.md              # This file
├── research.md          # Phase 0 output (minimal for this feature)
├── data-model.md        # Phase 1 output (tab state model)
├── quickstart.md        # Phase 1 output (dev setup)
├── contracts/           # Phase 1 output (no API changes)
└── tasks.md             # Phase 2 output (/speckit.tasks command)
```

### Source Code (repository root)

```text
crates/ckrv-ui/frontend/
├── src/
│   ├── App.tsx                    # UPDATE: Remove individual page routes, add CodePage
│   ├── layouts/
│   │   └── Dashboard.tsx          # UPDATE: Reduce nav items from 9 to 5
│   └── components/
│       ├── CodePage.tsx           # NEW: Unified tabbed container
│       ├── SpecEditor.tsx         # UNCHANGED (rendered in tab)
│       ├── TaskEditor.tsx         # UNCHANGED (rendered in tab)
│       ├── PlanEditor.tsx         # UNCHANGED (rendered in tab)
│       └── ExecutionRunner.tsx    # UNCHANGED (rendered in tab)
└── tests/
    └── e2e/
        └── code-page.spec.ts      # NEW: Tab navigation tests
```

**Structure Decision**: Minimal changes to existing architecture. New `CodePage.tsx` component acts as a container that renders existing components based on active tab.

## Complexity Tracking

> No violations - this is a straightforward refactoring that simplifies navigation.

| Violation | Why Needed | Simpler Alternative Rejected Because |
|-----------|------------|-------------------------------------|
| N/A | N/A | N/A |
