# Tasks: Agent Orchestration

**Input**: Design documents from `/specs/003-agent-orchestration/`
**Prerequisites**: plan.md (required), spec.md (required for user stories), research.md, data-model.md, contracts/

**Tests**: Tests are included as per Constitution (Principle II: Testing Standards).

**Organization**: Tasks are grouped by user story to enable independent implementation and testing of each story.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel (different files, no dependencies)
- **[Story]**: Which user story this task belongs to (e.g., US1, US2, US3)
- Include exact file paths in descriptions

## Path Conventions

- **Project Type**: Rust CLI (single project)
- Paths: `src/`, `tests/` at repository root

---

## Phase 1: Setup (Shared Infrastructure)

**Purpose**: Project initialization and dependency configuration

- [x] T001 Add orchestration dependencies to `Cargo.toml` (`bollard`, `tokio`, `serde_yaml`, `handlebars`, `thiserror`, `anyhow`, `chrono`)
- [x] T002 [P] Create orchestrator module structure (implemented in `crates/ckrv-core/` modules)
- [x] T003 [P] Add default SWE workflow file to `.ckrv/workflows/swe.yml`

---

## Phase 2: Foundational (Blocking Prerequisites)

**Purpose**: Core infrastructure that MUST be complete before ANY user story can be implemented

**⚠️ CRITICAL**: No user story work can begin until this phase is complete

- [x] T004 Define `Workflow`, `WorkflowStep`, `StepOutput`, `OutputType` structs in `crates/ckrv-core/src/workflow.rs` per data-model.md
- [x] T005 [P] Define `AgentTask`, `AgentTaskStatus` structs in `crates/ckrv-core/src/agent_task.rs` per data-model.md
- [x] T006 [P] Define `StepExecutionResult` struct in `crates/ckrv-core/src/step_result.rs` per data-model.md
- [x] T007 Implement YAML parsing for `Workflow` using `serde_yaml` in `crates/ckrv-core/src/workflow.rs`
- [x] T008 [P] Implement Task persistence (load/save to `.ckrv/tasks/<id>/metadata.json`) in `crates/ckrv-core/src/agent_task.rs`
- [x] T009 [P] Implement git worktree creation helper in `crates/ckrv-cli/src/commands/task.rs` (using `ckrv-git`)
- [x] T010 Define error types for orchestrator in `crates/ckrv-core/src/runner.rs` (RunnerError)

**Checkpoint**: Foundation ready - user story implementation can now begin

---

## Phase 3: User Story 1 - Task Execution with SWE Workflow (Priority: P1) 🎯 MVP

**Goal**: Execute a multi-step workflow (Plan -> Implement) using Claude Code, storing artifacts.

**Independent Test**: Run `ckrv task "Create hello.txt"` and verify `hello.txt` is created and `plan.md` artifact exists in `.ckrv/tasks/<id>/`.

### Tests for User Story 1

> **NOTE: Write these tests FIRST, ensure they FAIL before implementation**

- [ ] T011 [P] [US1] Create unit test for `Workflow` YAML parsing in `tests/orchestrator/workflow_tests.rs`
- [ ] T012 [P] [US1] Create unit test for `PromptRenderer` template substitution in `tests/orchestrator/prompt_tests.rs`
- [ ] T013 [US1] Create integration test for full workflow execution (mocked agent) in `tests/orchestrator/runner_tests.rs`

### Implementation for User Story 1

- [x] T014 [P] [US1] Implement `PromptRenderer` using `handlebars` crate in `crates/ckrv-core/src/prompt.rs` (supports `{{steps.x.outputs.y}}` syntax)
- [x] T015 [US1] Implement `WorkflowRunner` struct (iterates steps, invokes agent, captures output) in `crates/ckrv-core/src/runner.rs`
- [x] T016 [US1] Implement agent invocation via CLI in `crates/ckrv-core/src/runner.rs` (calls Claude CLI, captures stdout/stderr)
- [x] T017 [US1] Implement step output parsing (JSON and file-based outputs) in `crates/ckrv-core/src/runner.rs`
- [x] T018 [US1] Wire `ckrv task` CLI command in `crates/ckrv-cli/src/commands/task.rs` (parse args, load workflow, create Task, run Runner)
- [x] T019 [US1] Implement `--json` flag for machine-readable output in `crates/ckrv-cli/src/commands/task.rs` (FR-010)
- [x] T020 [US1] Add explicit exit codes per CLI contract (0=success, 1=agent error, 2=config error) in `crates/ckrv-cli/src/commands/task.rs`

**Checkpoint**: At this point, User Story 1 should be fully functional and testable independently

---

## Phase 4: User Story 2 - Sandboxed Execution (Priority: P1)

**Goal**: Execute all agent commands inside a Docker container with proper volume mounts and credential handling.

**Independent Test**: Run a task and verify via `docker ps` that a container was created. Verify host files outside worktree are untouched.

### Tests for User Story 2

- [ ] T021 [P] [US2] Create integration test for Docker container lifecycle (create, start, stop) in `tests/orchestrator/docker_tests.rs` (requires Docker)
- [ ] T022 [P] [US2] Create unit test for credential mount generation in `tests/orchestrator/sandbox_tests.rs`

### Implementation for User Story 2

- [x] T023 [P] [US2] Implement `Sandbox` struct with `bollard` Docker client in `crates/ckrv-sandbox/src/docker.rs` (EXISTING)
- [x] T024 [US2] Implement container creation with workspace volume mount (`-v worktree:/workspace:rw`) in `crates/ckrv-sandbox/src/docker.rs` (EXISTING)
- [x] T025 [US2] Implement credential mounting (read-only mounts for `~/.claude.json`, etc.) in `crates/ckrv-sandbox/src/docker.rs`
- [x] T026 [US2] Implement container teardown (stop and remove) after task completion in `crates/ckrv-sandbox/src/docker.rs` (EXISTING)
- [x] T027 [US2] Integrate `Sandbox` into `Runner` so all agent invocations use Docker in `crates/ckrv-core/src/runner.rs`
- [x] T028 [US2] Add error handling for Docker unavailable/failed scenarios in `crates/ckrv-sandbox/src/error.rs` (EXISTING)

**Checkpoint**: At this point, User Stories 1 AND 2 should both work independently

---

## Phase 5: User Story 3 - Workflow Customization (Priority: P2)

**Goal**: Allow users to define custom workflows via YAML files.

**Independent Test**: Create `custom.yml` with a single "echo" step, run `ckrv task --workflow custom "test"`, verify custom step executes.

### Tests for User Story 3

- [ ] T029 [P] [US3] Create unit test for workflow loading from custom path in `tests/orchestrator/workflow_tests.rs`

### Implementation for User Story 3

- [x] T030 [US3] Implement workflow discovery (check `.ckrv/workflows/`, then embedded defaults) in `crates/ckrv-cli/src/commands/task.rs`
- [x] T031 [US3] Add `--workflow <name-or-path>` flag to `ckrv task` command in `crates/ckrv-cli/src/commands/task.rs`
- [x] T032 [US3] Validate custom workflow schema on load (reject invalid YAML) in `crates/ckrv-core/src/workflow.rs`

**Checkpoint**: All user stories should now be independently functional

---

## Phase 6: Polish & Cross-Cutting Concerns

**Purpose**: Improvements that affect multiple user stories

- [ ] T033 [P] Add module-level documentation (README) to `src/orchestrator/README.md`
- [ ] T034 [P] Ensure `clippy` passes with zero warnings on all new code
- [x] T035 Run `cargo fmt` on all new files
- [ ] T036 Validate against `quickstart.md` scenarios manually
- [x] T037 [P] Add `--dry-run` flag (display plan without execution) to `crates/ckrv-cli/src/commands/task.rs`

---

## Dependencies & Execution Order

### Phase Dependencies

- **Setup (Phase 1)**: No dependencies - can start immediately
- **Foundational (Phase 2)**: Depends on Setup completion - BLOCKS all user stories
- **User Stories (Phase 3+)**: All depend on Foundational phase completion
  - US1 and US2 are both P1 and closely coupled (Runner needs Sandbox)
  - Recommended: US1 first (core loop), then US2 (adds Docker), then US3
- **Polish (Final Phase)**: Depends on all desired user stories being complete

### User Story Dependencies

- **User Story 1 (P1)**: Can start after Foundational (Phase 2) - Core orchestration loop
- **User Story 2 (P1)**: Depends on US1 (Runner implementation exists) - Adds sandboxing
- **User Story 3 (P2)**: Can start after Foundational (Phase 2) - Custom workflow loading

### Within Each User Story

- Tests MUST be written and FAIL before implementation
- Data structs before logic
- Core implementation before CLI integration
- Story complete before moving to next priority

### Parallel Opportunities

- T002, T003 can run in parallel (different files)
- T004, T005, T006 can run in parallel (different files, no deps)
- T007, T008, T009, T010 can run in parallel after their respective structs exist
- All tests for a user story marked [P] can run in parallel
- T021, T022 can run in parallel
- T023 can run parallel to T014

---

## Parallel Example: User Story 1

```bash
# Launch all tests for User Story 1 together:
Task T011: "Unit test for Workflow YAML parsing"
Task T012: "Unit test for PromptRenderer template substitution"

# Then implement models in parallel:
Task T014: "Implement PromptRenderer using handlebars"
# (T015-T020 sequential as they build on each other)
```

---

## Implementation Strategy

### MVP First (User Story 1 Only)

1. Complete Phase 1: Setup
2. Complete Phase 2: Foundational (CRITICAL - blocks all stories)
3. Complete Phase 3: User Story 1
4. **STOP and VALIDATE**: Test with `ckrv task "create hello.txt"` (can use local agent, no Docker yet)
5. Proceed to US2 (sandboxed) for production-readiness

### Incremental Delivery

1. Setup + Foundational → Foundation ready
2. Add User Story 1 → Test independently → Demo (MVP without Docker)
3. Add User Story 2 → Test independently → Production-ready (with Docker isolation)
4. Add User Story 3 → Test independently → Customizable workflows
5. Polish → Documentation, linting, dry-run

---

## Notes

- [P] tasks = different files, no dependencies
- [Story] label maps task to specific user story for traceability
- Each user story should be independently completable and testable
- Verify tests fail before implementing (TDD per Constitution)
- Commit after each task or logical group
- Stop at any checkpoint to validate story independently
- US1 and US2 are tightly coupled in functionality but separately testable
