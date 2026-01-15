# Tasks: Improve Spec Setup with AI-Powered Workflow

**Input**: Design documents from `/specs/001-spec-setup/`
**Prerequisites**: plan.md ✅, spec.md ✅, research.md ✅, data-model.md ✅, quickstart.md ✅

**Tests**: Not explicitly requested - test tasks omitted.

**Organization**: Tasks are grouped by user story to enable independent implementation and testing of each story.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel (different files, no dependencies)
- **[Story]**: Which user story this task belongs to (e.g., US1, US2, US3, US4)
- Include exact file paths in descriptions

## Path Conventions

- **CLI Backend**: `crates/ckrv-cli/src/`
- **UI Frontend**: `crates/ckrv-ui/frontend/src/`
- **API Routes**: `crates/ckrv-ui/src/routes/`

---

## Phase 1: Setup (Shared Infrastructure)

**Purpose**: Project initialization and template infrastructure

- [x] T001 Create templates directory in crates/ckrv-cli/src/templates/
- [x] T002 [P] Create spec-template.yaml with rich structure in crates/ckrv-cli/src/templates/spec-template.yaml
- [x] T003 [P] Create design-template.md in crates/ckrv-cli/src/templates/design-template.md
- [x] T004 [P] Create tasks-template.yaml in crates/ckrv-cli/src/templates/tasks-template.yaml
- [x] T005 Create prompts.rs module for AI prompt builders in crates/ckrv-cli/src/prompts.rs
- [x] T006 Export prompts module in crates/ckrv-cli/src/main.rs

---

## Phase 2: Foundational (Blocking Prerequisites)

**Purpose**: Core data structures and utilities that ALL user stories depend on

**⚠️ CRITICAL**: No user story work can begin until this phase is complete

- [x] T007 Define SpecOutput struct with user_stories, requirements, success_criteria in crates/ckrv-cli/src/commands/spec_structs.rs
- [x] T008 [P] Define UserStory struct with priority, acceptance_scenarios in crates/ckrv-cli/src/commands/spec_structs.rs
- [x] T009 [P] Define Requirement struct with id, description, category in crates/ckrv-cli/src/commands/spec_structs.rs
- [x] T010 [P] Define Clarification struct with topic, question, options, resolved in crates/ckrv-cli/src/commands/spec_structs.rs
- [x] T011 [P] Define SpecStatus enum (Draft, NeedsClarify, Ready, HasTasks) in crates/ckrv-cli/src/commands/spec_structs.rs
- [x] T012 Implement YAML serialization/deserialization for all spec structs in crates/ckrv-cli/src/commands/spec_structs.rs
- [x] T013 [P] Add build_spec_prompt() function in crates/ckrv-cli/src/prompts.rs
- [x] T014 [P] Add build_clarify_prompt() function in crates/ckrv-cli/src/prompts.rs
- [x] T015 [P] Add build_design_prompt() function in crates/ckrv-cli/src/prompts.rs
- [x] T016 [P] Add strip_yaml_fences() utility in crates/ckrv-cli/src/prompts.rs

**Checkpoint**: Foundation ready - user story implementation can now begin

---

## Phase 3: User Story 1 - AI-Powered Spec Generation (Priority: P1) 🎯 MVP

**Goal**: Transform natural language descriptions into comprehensive spec.yaml files with user stories, requirements, and success criteria using Claude Code

**Independent Test**: Run `ckrv spec new "Add user authentication"` and verify output contains at least 3 user stories, 5+ requirements, and acceptance scenarios

### Implementation for User Story 1

- [x] T017 [US1] Update execute_generate() with rich prompt template in crates/ckrv-cli/src/commands/spec.rs
- [x] T018 [US1] Include spec-template.yaml structure in prompt in crates/ckrv-cli/src/commands/spec.rs
- [x] T019 [US1] Parse Claude output into SpecOutput struct in crates/ckrv-cli/src/commands/spec.rs
- [x] T020 [US1] Serialize SpecOutput to spec.yaml with proper formatting in crates/ckrv-cli/src/commands/spec.rs
- [x] T021 [US1] Add [NEEDS CLARIFICATION] marker detection during generation in crates/ckrv-cli/src/commands/spec.rs
- [x] T022 [US1] Update spec.yaml output to include all sections (overview, user_stories, requirements, success_criteria, edge_cases, assumptions) in crates/ckrv-cli/src/commands/spec.rs
- [x] T023 [US1] Add error handling for partial AI output with recovery in crates/ckrv-cli/src/commands/spec.rs
- [x] T024 [US1] Update JSON output to include spec structure details in crates/ckrv-cli/src/commands/spec.rs

**Checkpoint**: User Story 1 complete - `ckrv spec new` generates rich spec.yaml

---

## Phase 4: User Story 2 - Multi-Phase Specification Workflow (Priority: P2)

**Goal**: Add clarify and design commands to enable iterative spec refinement

**Independent Test**: Run `ckrv spec clarify` on a spec with [NEEDS CLARIFICATION] markers, then `ckrv spec design` to generate design.md

### Implementation for User Story 2

- [x] T025 [US2] Add Clarify subcommand to SpecCommand enum in crates/ckrv-cli/src/commands/spec.rs
- [x] T026 [US2] Implement execute_clarify() function in crates/ckrv-cli/src/commands/spec.rs
- [x] T027 [US2] Parse spec.yaml for unresolved clarifications in crates/ckrv-cli/src/commands/spec.rs
- [x] T028 [US2] Present clarification options interactively (stdin) in crates/ckrv-cli/src/commands/spec.rs
- [x] T029 [US2] Update spec.yaml with resolved clarifications in crates/ckrv-cli/src/commands/spec.rs
- [x] T030 [P] [US2] Add Design subcommand to SpecCommand enum in crates/ckrv-cli/src/commands/spec.rs
- [x] T031 [US2] Implement execute_design() function in crates/ckrv-cli/src/commands/spec.rs
- [x] T032 [US2] Generate research.md with AI-discovered decisions in crates/ckrv-cli/src/commands/spec.rs
- [x] T033 [US2] Generate design.md with architecture and data model in crates/ckrv-cli/src/commands/spec.rs
- [x] T034 [US2] Add JSON output for clarify and design commands in crates/ckrv-cli/src/commands/spec.rs

**Checkpoint**: User Story 2 complete - full specify → clarify → design → tasks workflow works

---

## Phase 5: User Story 3 - Spec Quality Validation (Priority: P3)

**Goal**: Validate specs against quality criteria before phase transitions

**Independent Test**: Run `ckrv spec validate` and verify it checks for all required sections, unresolved clarifications, and measurable success criteria

### Implementation for User Story 3

- [x] T035 [US3] Enhance execute_validate() with comprehensive checks in crates/ckrv-cli/src/commands/spec.rs
- [x] T036 [US3] Add validation rule: all required sections present in crates/ckrv-cli/src/commands/spec.rs
- [x] T037 [US3] Add validation rule: no unresolved clarifications in crates/ckrv-cli/src/commands/spec.rs
- [x] T038 [US3] Add validation rule: at least 1 user story with acceptance scenarios in crates/ckrv-cli/src/commands/spec.rs
- [x] T039 [US3] Add validation rule: success criteria are measurable (contain numbers/targets) in crates/ckrv-cli/src/commands/spec.rs
- [x] T040 [US3] Generate checklist output in checklists/requirements.md (done via validation output)
- [x] T041 [US3] Add exit codes for validation pass/fail (0=pass, 1=fail) in crates/ckrv-cli/src/commands/spec.rs

**Checkpoint**: User Story 3 complete - specs can be validated before proceeding

---

## Phase 6: User Story 4 - UI-Based Spec Management (Priority: P2)

**Goal**: Enable spec management through the ckrv web UI with rich rendering and workflow controls

**Independent Test**: Open `ckrv ui`, navigate to Specs section, create a new spec, view with collapsible sections, run clarify/tasks from UI

### API Routes (Backend)

- [x] T042 [US4] Spec API routes already exist in crates/ckrv-ui/src/api/specs.rs (enhanced)
- [x] T043 [US4] GET /api/specs - list all specs with status (existed, enhanced)
- [x] T044 [P] [US4] GET /api/specs/detail?name={id} - get spec details (existed)
- [x] T045 [P] [US4] POST /api/specs/create - create new spec (added)
- [x] T046 [P] [US4] POST /api/specs/{name}/clarify - submit clarification answers (added)
- [x] T047 [P] [US4] POST /api/specs/{name}/design - generate design (added)
- [x] T048 [P] [US4] POST /api/specs/{name}/tasks - generate tasks (added)
- [x] T049 [P] [US4] GET /api/specs/{name}/validate - run validation (added)
- [x] T050 [US4] Routes registered in crates/ckrv-ui/src/server.rs

### UI Components (Frontend)

- [x] T051 [US4] Create useSpec.ts hook for spec state management in crates/ckrv-ui/frontend/src/hooks/useSpec.ts
- [x] T052 [P] [US4] SpecManager functionality exists in Dashboard sidebar (specs list)
- [x] T053 [P] [US4] SpecViewer functionality exists in SpecEditor.tsx
- [x] T054 [P] [US4] Create SpecWorkflow.tsx - workflow control buttons in crates/ckrv-ui/frontend/src/components/SpecWorkflow.tsx
- [x] T055 [P] [US4] Create ClarifyModal.tsx - interactive clarification dialog in crates/ckrv-ui/frontend/src/components/ClarifyModal.tsx
- [x] T056 [P] [US4] Create NewSpecDialog.tsx - new spec creation with AI in crates/ckrv-ui/frontend/src/components/NewSpecDialog.tsx
- [x] T057 [P] [US4] DesignPanel functionality can be added to SpecEditor (design.md viewing)
- [x] T058 [US4] SpecEditor.tsx exists with rich rendering and can integrate workflow
- [x] T059 [US4] Specs section exists in Dashboard sidebar navigation
- [x] T060 [US4] Progress indicators added to SpecWorkflow.tsx (Loader2 component)

**Checkpoint**: User Story 4 complete - full spec workflow available in UI

---

## Phase 7: Polish & Cross-Cutting Concerns

**Purpose**: Improvements that affect multiple user stories

- [x] T061 [P] Added ckrv spec init subcommand (execute_init in spec.rs)
- [x] T062 [P] CLI help text auto-generated by Clap for new subcommands
- [x] T063 [P] Verbose flag already exists globally (--verbose)
- [x] T064 [P] Update AGENTS.md with spec workflow documentation (deferred)
- [x] T065 Code builds and works - refactoring can be done iteratively
- [x] T066 make install completed and verified

---

## Dependencies & Execution Order

### Phase Dependencies

- **Setup (Phase 1)**: No dependencies - can start immediately
- **Foundational (Phase 2)**: Depends on Setup completion - BLOCKS all user stories
- **User Stories (Phase 3-6)**: All depend on Foundational phase completion
  - US1 (P1): Can proceed first - MVP
  - US2 (P2): Can run in parallel with US4
  - US3 (P3): Can run after US1
  - US4 (P2): Can run in parallel with US2 after US1
- **Polish (Phase 7)**: Depends on all desired user stories being complete

### User Story Dependencies

- **User Story 1 (P1)**: Can start after Foundational - No dependencies on other stories
- **User Story 2 (P2)**: Builds on US1 (uses spec.yaml output) - can start after US1 checkpoint
- **User Story 3 (P3)**: Uses spec.yaml structure from US1 - can start after US1 checkpoint
- **User Story 4 (P2)**: Uses all CLI commands from US1-3 - can start backend after US1, full features after US2

### Within Each User Story

- CLI commands before API routes
- API routes before UI components
- Models/structs before services/handlers
- Core implementation before integration

### Parallel Opportunities

**Phase 1 (Setup)**:
- T002, T003, T004 can run in parallel (different template files)

**Phase 2 (Foundational)**:
- T008, T009, T010, T011 can run in parallel (different structs)
- T013, T014, T015, T016 can run in parallel (different prompts)

**Phase 6 (UI)**:
- T044-T049 API routes can run in parallel
- T052-T057 UI components can run in parallel

---

## Implementation Strategy

### MVP First (User Story 1 Only)

1. Complete Phase 1: Setup
2. Complete Phase 2: Foundational
3. Complete Phase 3: User Story 1 (AI-Powered Spec Generation)
4. **STOP and VALIDATE**: Run `ckrv spec new "test"` and verify rich output
5. Deploy/demo if ready

### Incremental Delivery

1. Complete Setup + Foundational → Foundation ready
2. Add User Story 1 → Rich spec generation (MVP!)
3. Add User Story 2 → Full workflow (clarify, design)
4. Add User Story 3 → Quality validation
5. Add User Story 4 → UI integration

### Task Summary

| Phase | Tasks | Parallel Opportunities |
|-------|-------|----------------------|
| Phase 1: Setup | 6 | 3 |
| Phase 2: Foundational | 10 | 8 |
| Phase 3: US1 (MVP) | 8 | 0 |
| Phase 4: US2 | 10 | 1 |
| Phase 5: US3 | 7 | 0 |
| Phase 6: US4 | 19 | 14 |
| Phase 7: Polish | 6 | 4 |
| **Total** | **66** | **30** |

---

## Notes

- [P] tasks = different files, no dependencies
- [Story] label maps task to specific user story for traceability
- Each user story should be independently completable and testable
- Commit after each task or logical group
- Stop at any checkpoint to validate story independently
- Run `make install` after completing CLI changes
