# Implementation Plan: CSS Theme Consolidation

**Branch**: `001-css-theme-consolidation` | **Date**: 2026-01-12 | **Spec**: [spec.md](./spec.md)
**Input**: Feature specification from `/specs/001-css-theme-consolidation/spec.md`

## Summary

Consolidate all disorganized CSS style references from 15 component files (~132 inline variable references) into a centralized theme storage in `index.css` using Tailwind 4's `@theme` directive and OKLCH color format. This enables rapid theme swapping with tweakcn by creating a single source of truth for all design tokens.

## Technical Context

**Language/Version**: TypeScript 5.x, CSS (Tailwind 4)  
**Primary Dependencies**: Tailwind CSS v4, React 18, Vite  
**Storage**: N/A (CSS files only)  
**Testing**: Visual regression testing (screenshot comparison), grep-based validation  
**Target Platform**: Web (modern browsers with OKLCH support)  
**Project Type**: Web application (React frontend)  
**Performance Goals**: N/A (CSS-only changes, no runtime impact)  
**Constraints**: Zero visual regressions, maintain all existing design tokens  
**Scale/Scope**: 15 component files, ~132 inline CSS variable references, 1 CSS file restructure

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

| Principle | Requirement | Status |
|-----------|-------------|--------|
| I. Code Quality Excellence | Full typing, zero lint errors, single responsibility | ✅ CSS-only changes; Tailwind handles type-safe class names |
| II. Testing Standards | TDD approach planned, coverage targets defined | ✅ Grep-based validation + visual regression testing |
| III. Reliability First | Error handling strategy, idempotency considered | ✅ CSS changes are idempotent; no runtime errors possible |
| IV. Security by Default | No hardcoded secrets, input validation planned | ✅ N/A - no secrets or user input in CSS theming |
| V. Deterministic CLI Behavior | Machine-readable output, explicit exit codes | ✅ N/A - frontend styling only |

## Project Structure

### Documentation (this feature)

```text
specs/001-css-theme-consolidation/
├── plan.md              # This file
├── research.md          # Phase 0: Tailwind 4 @theme best practices
├── data-model.md        # Phase 1: Color token inventory
├── quickstart.md        # Phase 1: How to swap themes
├── contracts/           # Phase 1: Theme variable schema
└── tasks.md             # Phase 2: Implementation tasks
```

### Source Code (feature scope)

```text
crates/ckrv-ui/frontend/
├── src/
│   ├── index.css              # PRIMARY: Theme consolidation target
│   ├── App.css                # Review for stray variables
│   ├── layouts/
│   │   └── Dashboard.tsx      # 2 inline var() references
│   └── components/            # 14 files with inline var() references
│       ├── AgentManager.tsx
│       ├── CommandPalette.tsx
│       ├── CompletionSummary.tsx
│       ├── DiffViewer.tsx
│       ├── LogViewer.tsx
│       ├── PlanEditor.tsx
│       ├── RunHistoryPanel.tsx
│       ├── SpecEditor.tsx
│       ├── StatusWidget.tsx
│       ├── TaskDetailModal.tsx
│       ├── TaskEditor.tsx
│       ├── WorkflowPanel.tsx
│       └── ui/
│           ├── badge.tsx
│           └── LoadingOverlay.tsx
```

**Structure Decision**: Web application pattern. Changes are isolated to the frontend CSS and component styling. No backend changes required.

## Complexity Tracking

> No Constitution Check violations. This is a pure refactoring task with no new functionality.

| Aspect | Complexity | Notes |
|--------|------------|-------|
| Theme Token Count | Medium | ~25 distinct color tokens to map |
| Component Count | Medium | 15 files to update |
| Migration Risk | Low | Find-replace pattern, easily reversible |
| Visual Regression | Low | OKLCH produces identical visual output |

## Phase Completion Status

| Phase | Status | Output |
|-------|--------|--------|
| Phase 0: Research | ✅ Complete | [research.md](./research.md) |
| Phase 1: Design | ✅ Complete | [data-model.md](./data-model.md), [contracts/](./contracts/), [quickstart.md](./quickstart.md) |
| Phase 2: Tasks | ✅ Complete | [tasks.md](./tasks.md) |
| Phase 3: Implementation | ✅ Complete | OKLCH theme centralized, 79% inline var() reduction (134→28), all 4 user stories delivered |

## Implementation Summary

- **Baseline**: 134 inline `var(--*)` references in 15 component files
- **Final Count**: 28 remaining (all in inline style objects - intentional)
- **Reduction**: 79% of inline CSS variable references eliminated
- **Build Status**: ✅ Passing (`npm run build` successful)
- **Theme Utilities**: 112 usages of semantic Tailwind classes across 16 components

## Next Steps

Implementation complete. This feature is ready for visual verification and deployment.

