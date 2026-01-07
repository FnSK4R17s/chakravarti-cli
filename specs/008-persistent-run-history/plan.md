# Implementation Plan: Persistent Run History

**Branch**: `008-persistent-run-history` | **Date**: 2026-01-07 | **Spec**: [spec.md](./spec.md)
**Input**: Feature specification from `/specs/008-persistent-run-history/spec.md`

## Summary

Add persistent run history to the Execution Runner that survives page refreshes and tab switches. Run data is stored in YAML files within each spec directory (`runs.yaml`). The UI will also be updated for consistency with other pages and to show completion summaries.

**Key Components**:
1. Backend: YAML-based run history storage and retrieval APIs
2. Frontend: History panel, completion summary, UI consistency updates
3. Real-time updates: WebSocket integration for live run status persistence

## Technical Context

**Language/Version**: Rust 1.75+ (backend), TypeScript 5.x (frontend)  
**Primary Dependencies**: axum, tokio, serde_yaml (backend); React 18, @tanstack/react-query (frontend)  
**Storage**: YAML files in `.specs/<spec-name>/runs.yaml`  
**Testing**: cargo test (backend), vitest/playwright (frontend)  
**Target Platform**: Desktop browsers, Linux/macOS  
**Project Type**: Web application (Rust backend + React frontend)  
**Performance Goals**: Run history loads < 1 second for 100 runs  
**Constraints**: No external database, graceful degradation on file errors  
**Scale/Scope**: Medium feature affecting 3-5 files per component

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

| Principle | Requirement | Status |
|-----------|-------------|--------|
| I. Code Quality Excellence | Full typing, zero lint errors, single responsibility | ☑ TypeScript strict mode, Rust clippy, modular design |
| II. Testing Standards | TDD approach planned, coverage targets defined | ☑ Unit tests for YAML serialization, E2E for history display |
| III. Reliability First | Error handling strategy, idempotency considered | ☑ Graceful degradation on file errors, atomic writes |
| IV. Security by Default | No hardcoded secrets, input validation planned | ☑ YAML input validation, path sanitization |
| V. Deterministic CLI Behavior | Machine-readable output, explicit exit codes | ☑ JSON API responses, consistent state |

## Project Structure

### Documentation (this feature)

```text
specs/008-persistent-run-history/
├── plan.md              # This file
├── research.md          # Storage format decisions
├── data-model.md        # Run and BatchResult entities
├── quickstart.md        # Testing instructions
├── contracts/           # API specifications
└── tasks.md             # Implementation tasks
```

### Source Code (repository root)

```text
# Backend (Rust)
crates/ckrv-ui/src/
├── api/
│   ├── mod.rs            # Add history routes
│   └── history.rs        # NEW: Run history API handlers
├── services/
│   ├── engine.rs         # Integrate history persistence
│   └── history.rs        # NEW: YAML storage service
└── models/
    └── history.rs        # NEW: Run, BatchResult types

# Frontend (TypeScript/React)
crates/ckrv-ui/frontend/src/
├── components/
│   ├── ExecutionRunner.tsx   # Add history panel, completion summary
│   ├── RunHistoryPanel.tsx   # NEW: History list component
│   └── CompletionSummary.tsx # NEW: Run completion summary
├── hooks/
│   └── useRunHistory.ts      # NEW: History data fetching hook
└── types/
    └── history.ts            # NEW: Run, BatchResult types
```

**Structure Decision**: Add new files for history functionality, integrate with existing ExecutionRunner and engine.

## Complexity Tracking

> No constitution violations - this follows established patterns.

| Violation | Why Needed | Simpler Alternative Rejected Because |
|-----------|------------|-------------------------------------|
| N/A | N/A | N/A |
