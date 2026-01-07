# Implementation Plan: Execution State Synchronization Fix

**Branch**: `007-execution-state-sync` | **Date**: 2026-01-06 | **Spec**: [spec.md](./spec.md)
**Input**: Feature specification from `/specs/007-execution-state-sync/spec.md`

## Summary

Fix the state synchronization issue between the Rust backend and React frontend in the Execution Runner. The backend sends log messages with `type_: "start"`, `"batch_complete"`, and `"success"` but the frontend expects `{ type: "status", status: "running/completed" }` format. This mismatch causes the timer to never start and batch completion to not be tracked.

**Solution**: Modify both backend and frontend:
1. Backend: Add explicit status messages alongside existing log messages
2. Frontend: Add pattern matching for backend's actual message formats as fallback

## Technical Context

**Language/Version**: Rust 1.75+ (backend), TypeScript 5.x (frontend)  
**Primary Dependencies**: axum, tokio (backend); React 18, XTerm.js (frontend)  
**Storage**: N/A (stateless WebSocket communication)  
**Testing**: cargo test (backend), vitest/playwright (frontend)  
**Target Platform**: Desktop browsers, Linux/macOS  
**Project Type**: Web application (Rust backend + React frontend)  
**Performance Goals**: UI updates within 100ms of WebSocket message receipt  
**Constraints**: Backward compatible with existing message formats  
**Scale/Scope**: Single feature bug fix affecting ~2 files

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

| Principle | Requirement | Status |
|-----------|-------------|--------|
| I. Code Quality Excellence | Full typing, zero lint errors, single responsibility | ☑ TypeScript strict mode, Rust clippy |
| II. Testing Standards | TDD approach planned, coverage targets defined | ☑ Unit tests for message parsing |
| III. Reliability First | Error handling strategy, idempotency considered | ☑ Graceful handling of unknown message types |
| IV. Security by Default | No hardcoded secrets, input validation planned | ☑ N/A - no security changes |
| V. Deterministic CLI Behavior | Machine-readable output, explicit exit codes | ☑ N/A - UI feature only |

## Project Structure

### Documentation (this feature)

```text
specs/007-execution-state-sync/
├── plan.md              # This file
├── research.md          # Root cause analysis
├── data-model.md        # Message format definitions
├── quickstart.md        # Testing instructions
├── contracts/           # WebSocket message contracts
└── tasks.md             # Implementation tasks
```

### Source Code (repository root)

```text
# Files to modify:

crates/ckrv-ui/src/
├── api/
│   └── execution.rs          # Add status message sends
└── services/
    └── engine.rs             # Add status messages to log stream

crates/ckrv-ui/frontend/src/
├── components/
│   └── ExecutionRunner.tsx   # Enhanced message parsing
└── types/
    └── websocket.ts          # Message type definitions (new)

crates/ckrv-ui/frontend/tests/
└── unit/
    └── message-parsing.test.ts  # Unit tests for parsing
```

**Structure Decision**: Minimal changes to existing files. Add new type definitions and tests.

## Complexity Tracking

> No constitution violations - this is a straightforward bug fix.

| Violation | Why Needed | Simpler Alternative Rejected Because |
|-----------|------------|-------------------------------------|
| N/A | N/A | N/A |
