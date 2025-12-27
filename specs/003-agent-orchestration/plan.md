# Implementation Plan: Agent Orchestration

**Branch**: `003-agent-orchestration` | **Date**: 2025-12-25 | **Spec**: [specs/003-agent-orchestration/spec.md](spec.md)
**Input**: Feature specification from `/specs/003-agent-orchestration/spec.md`

**Note**: This template is filled in by the `/speckit.plan` command. See `.specify/templates/commands/plan.md` for the execution workflow.

## Summary

Build a meta-agent orchestration system similar to Rover, porting its architecture to Rust. Primary requirement is to enable multi-step workflows (Plan -> Implement) running within a secure Docker sandbox, using LLM agents like Claude Code.

## Technical Context

**Language/Version**: Rust 1.75+ (Stable)
**Primary Dependencies**: 
- `bollard` (Docker API)
- `tokio` (Async runtime)
- `serde`/`serde_yaml` (Config/Workflow parsing)
- `handlebars` (Template rendering for prompts)
- `clap` (CLI integration)
- `thiserror` / `anyhow` (Error handling)
**Storage**: Filesystem (.ckrv/tasks for state)
**Testing**: `cargo test` (Unit/Integration)
**Target Platform**: Linux (primary), potentially macOS/Windows via Docker Desktop
**Project Type**: CLI Application
**Performance Goals**: Minimal overhead (<500ms startup for orchestrator), agent speed limited by API
**Constraints**: Must run docker containers securely (volume mounts), handle robust error recovery
**Scale/Scope**: Orchestrate single-agent tasks initially, extensible to multi-agent

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

| Principle | Requirement | Status |
|-----------|-------------|--------|
| I. Code Quality Excellence | Full typing, zero lint errors, single responsibility | ✅ |
| II. Testing Standards | TDD approach planned, coverage targets defined | ✅ |
| III. Reliability First | Error handling strategy, idempotency considered | ✅ |
| IV. Security by Default | No hardcoded secrets, input validation planned | ✅ |
| V. Deterministic CLI Behavior | Machine-readable output, explicit exit codes | ✅ |

## Project Structure

### Documentation (this feature)

```text
specs/003-agent-orchestration/
├── plan.md              # This file (/speckit.plan command output)
├── research.md          # Phase 0 output (/speckit.plan command)
├── data-model.md        # Phase 1 output (/speckit.plan command)
├── quickstart.md        # Phase 1 output (/speckit.plan command)
├── contracts/           # Phase 1 output (/speckit.plan command)
└── tasks.md             # Phase 2 output (/speckit.tasks command - NOT created by /speckit.plan)
```

### Source Code (repository root)

```text
# Implementation in Rust
src/
├── orchestrator/          # New module for agent orchestration
│   ├── docker.rs          # Sandbox implementation (bollard)
│   ├── workflow.rs        # YAML parsing and definition
│   ├── runner.rs          # Step execution logic
│   ├── prompt.rs          # Template rendering
│   └── mod.rs
├── commands/
│   └── task.rs            # CLI entrypoint for `ckrv task`
└── lib/                   # Shared utilities (if any)

tests/
├── orchestrator/
│   ├── workflow_tests.rs
│   └── docker_tests.rs    # Integration tests requiring docker
```

**Structure Decision**: Integration into existing Rust CLI structure (`src/`) adding a dedicated `orchestrator` module to maintain encapsulation.

## Complexity Tracking

> **Fill ONLY if Constitution Check has violations that must be justified**

| Violation | Why Needed | Simpler Alternative Rejected Because |
|-----------|------------|-------------------------------------|
| N/A | | |
