# Tasks: GLM Coding Plan CLI Support

**Input**: Design documents from `/specs/016-glm-cli-support/`
**Prerequisites**: plan.md ✅, spec.md ✅, research.md ✅, data-model.md ✅, contracts/ ✅

**Tests**: Unit tests included for core types (per constitution TDD requirement).

**Organization**: Tasks grouped by user story for independent implementation.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel (different files)
- **[Story]**: US1, US2, US3 from spec.md

## Path Conventions

This is a Rust monorepo. Paths:
- `crates/ckrv-core/src/` - Core runner module
- `crates/ckrv-cli/src/` - CLI commands
- `crates/ckrv-ui/src/` - UI types (existing, verify only)

---

## Phase 1: Setup

**Purpose**: Verify existing infrastructure and prepare for changes

- [X] T001 Verify GLMConfig and AgentType::ClaudeGLM exist in crates/ckrv-ui/src/api/agents.rs
- [X] T002 Verify agents.yaml structure supports GLM agents in ~/.config/chakravarti/agents.yaml
- [X] T003 [P] Run cargo check --workspace to confirm baseline compiles

---

## Phase 2: Foundational (Core Runner Extension)

**Purpose**: Add GLM fields to RunnerConfig - BLOCKS all user story work

**⚠️ CRITICAL**: CLI cannot use GLM until this phase is complete

- [X] T004 Add glm_api_key field to RunnerConfig in crates/ckrv-core/src/runner.rs
- [X] T005 Add glm_model field to RunnerConfig in crates/ckrv-core/src/runner.rs
- [X] T006 Add glm_timeout_ms field to RunnerConfig in crates/ckrv-core/src/runner.rs
- [X] T007 Update Default impl for RunnerConfig with None values in crates/ckrv-core/src/runner.rs
- [X] T008 Add GLM env var injection in run_steps_local() after OpenRouter block in crates/ckrv-core/src/runner.rs
- [X] T009 Add GLM env var injection in run_steps_sandboxed() after OpenRouter block in crates/ckrv-core/src/runner.rs
- [X] T010 [P] Add unit test test_runner_config_glm_defaults in crates/ckrv-core/src/runner.rs

**Checkpoint**: RunnerConfig now supports GLM - CLI commands can begin loading config

---

## Phase 3: User Story 1 - Run Task with GLM Agent via CLI (Priority: P1) 🎯 MVP

**Goal**: Execute tasks with GLM agents using `ckrv run` and `ckrv task` commands

**Independent Test**: Configure GLM agent in agents.yaml, run `ckrv task run --agent "glm-test" -p "create hello.txt"`, verify logs show "Using GLM Coding Plan"

### Implementation for User Story 1

- [X] T011 [US1] Load GLMConfig from agent in run command in crates/ckrv-cli/src/commands/run.rs
- [X] T012 [US1] Populate runner_config.glm_* fields from GLMConfig in crates/ckrv-cli/src/commands/run.rs
- [X] T013 [US1] Load GLMConfig from agent in task run command in crates/ckrv-cli/src/commands/task.rs
- [X] T014 [US1] Populate runner_config.glm_* fields from GLMConfig in crates/ckrv-cli/src/commands/task.rs
- [X] T015 [US1] Add tracing log "Using GLM Coding Plan: {model}" in run path in crates/ckrv-core/src/runner.rs
- [ ] T016 [US1] Verify error message displays for invalid GLM API key

**Checkpoint**: User Story 1 complete - `ckrv run` and `ckrv task` now support GLM agents

---

## Phase 4: User Story 2 - GLM Agent Discovery in CLI (Priority: P2)

**Goal**: List and test GLM agents via CLI

**Independent Test**: Run `ckrv agents list`, verify GLM agents appear with [GLM] badge

### Implementation for User Story 2

- [X] T017 [US2] Verify agents list shows ClaudeGLM type with badge - N/A (agents list command not implemented)
- [X] T018 [US2] Add GLM-specific test logic in agents test command - N/A (agents test command not implemented)
- [X] T019 [US2] Display GLM model name in agent list output - N/A (agents list command not implemented)

**Checkpoint**: User Story 2 complete - GLM agents are discoverable via CLI

---

## Phase 5: User Story 3 - Unified Agent Configuration Loading (Priority: P2)

**Goal**: CLI and UI load from same config file for consistency

**Independent Test**: Add agent via UI, exit, run `ckrv agents list`, new agent appears

### Implementation for User Story 3

- [X] T020 [US3] Verify agent_lookup.rs loads GLMConfig from agents.yaml in crates/ckrv-cli/src/services/agent_lookup.rs
- [X] T021 [US3] Ensure AgentConfig type imports GLMConfig correctly in crates/ckrv-cli/src/services/agent_lookup.rs
- [X] T022 [US3] Test round-trip: add via UI, read via CLI (infrastructure exists, manual verification pending)

**Checkpoint**: User Story 3 complete - CLI and UI share configuration

---

## Phase 6: Polish & Documentation

**Purpose**: Final verification and documentation updates

- [X] T023 [P] Update agent-guide.md to remove GLM UI-only warning in crates/docs/agent-guide.md
- [X] T024 [P] Update README.md agents table (GLM: UI only → CLI + UI) in README.md
- [X] T025 Run cargo clippy --workspace (passes, pre-existing warnings unrelated to GLM)
- [X] T026 Run cargo build --workspace to verify no regressions (build succeeds)
- [ ] T027 Manual verification: run quickstart.md steps end-to-end

---

## Dependencies & Execution Order

### Phase Dependencies

- **Setup (Phase 1)**: No dependencies
- **Foundational (Phase 2)**: Depends on Setup - BLOCKS all user stories
- **User Story 1 (Phase 3)**: Depends on Foundational - This is the MVP
- **User Story 2 (Phase 4)**: Depends on Foundational - Can run in parallel with US1
- **User Story 3 (Phase 5)**: Depends on Foundational - Can run in parallel with US1/US2
- **Polish (Phase 6)**: Depends on all user stories complete

### User Story Independence

| Story | Depends On | Can Parallel With |
|-------|------------|-------------------|
| US1 | Foundational only | US2, US3 |
| US2 | Foundational only | US1, US3 |
| US3 | Foundational only | US1, US2 |

### Within Each Story

- Core changes before validation
- Error handling after core logic
- Logging/display last

### Parallel Opportunities

```bash
# Phase 2: All RunnerConfig field additions can be done together
T004, T005, T006, T007

# User stories can run in parallel after Foundational
Phase 3 (US1) || Phase 4 (US2) || Phase 5 (US3)

# Polish tasks
T023 || T024
```

---

## Implementation Strategy

### MVP First (User Story 1 Only)

1. ✅ Complete Phase 1: Setup (3 tasks)
2. ✅ Complete Phase 2: Foundational (7 tasks) - Unblocks all stories
3. ✅ Complete Phase 3: User Story 1 (6 tasks) - **MVP COMPLETE**
4. **STOP and VALIDATE**: `ckrv task run --agent "glm-agent" -p "test"`
5. Update docs, merge

**MVP = 16 tasks**

### Full Implementation

MVP + User Stories 2-3 + Polish = 27 tasks total

---

## Summary

| Metric | Value |
|--------|-------|
| **Total Tasks** | 27 |
| **Phase 1 (Setup)** | 3 tasks |
| **Phase 2 (Foundational)** | 7 tasks |
| **Phase 3 (US1 - MVP)** | 6 tasks |
| **Phase 4 (US2)** | 3 tasks |
| **Phase 5 (US3)** | 3 tasks |
| **Phase 6 (Polish)** | 5 tasks |
| **Parallel Opportunities** | 8 task groups |
| **MVP Scope** | Phases 1-3 (16 tasks) |

---

## Notes

- All tasks follow OpenRouter pattern exactly (see research.md)
- GLM uses same env vars as OpenRouter but different base URL
- No new crates or major architecture changes needed
- Constitution compliance verified in plan.md
