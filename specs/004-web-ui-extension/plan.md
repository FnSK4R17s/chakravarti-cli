# Implementation Plan: [FEATURE]

**Branch**: `[###-feature-name]` | **Date**: [DATE] | **Spec**: [link]
**Input**: Feature specification from `/specs/[###-feature-name]/spec.md`

**Note**: This template is filled in by the `/speckit.plan` command. See `.specify/templates/commands/plan.md` for the execution workflow.

## Summary

Implement a standalone Web UI for Chakravarti to monitor status and trigger commands (`init`, `spec`, `run`, `diff`, `promote`). The solution involves a new Rust crate `ckrv-ui` hosting an `axum` web server that APIs and serves a compiled React/TypeScript frontend (embedded via `rust-embed`).

## Technical Context

**Language/Version**: Rust 1.75+ (Backend), Node.js/TypeScript (Frontend Build)
**Primary Dependencies**: 
- Backend: `axum` (Web Server), `tokio` (Async Runtime), `rust-embed` (Asset Embedding), `serde` (Serialization)
- Frontend: `React`, `Vite`, `TanStack Query`, `TailwindCSS`
**Storage**: N/A (Uses existing file system / git worktrees via `ckrv-core`)
**Testing**: `cargo test` (Backend), `vitest` (Frontend)
**Target Platform**: Runs locally on user's machine (Linux/macOS/Windows) via CLI
**Project Type**: Hybrid (Rust CLI + Embedded Web App)
**Performance Goals**: UI loads < 200ms, API response < 50ms
**Constraints**: Must be distributable as a single binary (no separate frontend server requirement at runtime)
**Scale/Scope**: Single-user, local-only

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

| Principle | Requirement | Status |
|-----------|-------------|--------|
| I. Code Quality Excellence | Full typing, zero lint errors, single responsibility | ☑ |
| II. Testing Standards | TDD approach planned, coverage targets defined | ☑ |
| III. Reliability First | Error handling strategy, idempotency considered | ☑ |
| IV. Security by Default | No hardcoded secrets, input validation planned | ☑ |
| V. Deterministic CLI Behavior | Machine-readable output, explicit exit codes | ☑ |

## Project Structure

### Documentation (this feature)

```text
specs/[###-feature]/
├── plan.md              # This file (/speckit.plan command output)
├── research.md          # Phase 0 output (/speckit.plan command)
├── data-model.md        # Phase 1 output (/speckit.plan command)
├── quickstart.md        # Phase 1 output (/speckit.plan command)
├── contracts/           # Phase 1 output (/speckit.plan command)
└── tasks.md             # Phase 2 output (/speckit.tasks command - NOT created by /speckit.plan)
```

### Source Code (repository root)
<!--
  ACTION REQUIRED: Replace the placeholder tree below with the concrete layout
  for this feature. Delete unused options and expand the chosen structure with
  real paths (e.g., apps/admin, packages/something). The delivered plan must
  not include Option labels.
-->

```text
# [REMOVE IF UNUSED] Option 1: Single project (DEFAULT)
src/
├── models/
├── services/
├── cli/
└── lib/

tests/
├── contract/
├── integration/
└── unit/

# [REMOVE IF UNUSED] Option 2: Web application (when "frontend" + "backend" detected)
backend/
├── src/
│   ├── models/
│   ├── services/
│   └── api/
└── tests/

frontend/
├── src/
│   ├── components/
│   ├── pages/
│   └── services/
└── tests/

# [REMOVE IF UNUSED] Option 3: Mobile + API (when "iOS/Android" detected)
api/
└── [same as backend above]

ios/ or android/
└── [platform-specific structure: feature modules, UI flows, platform tests]
```

**Structure Decision**: [Document the selected structure and reference the real
directories captured above]

## Complexity Tracking

> **Fill ONLY if Constitution Check has violations that must be justified**

| Violation | Why Needed | Simpler Alternative Rejected Because |
|-----------|------------|-------------------------------------|
| [e.g., 4th project] | [current need] | [why 3 projects insufficient] |
| [e.g., Repository pattern] | [specific problem] | [why direct DB access insufficient] |
