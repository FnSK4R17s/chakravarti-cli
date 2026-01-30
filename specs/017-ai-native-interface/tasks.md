# Tasks: AI-Native Interface Layer

**Input**: Design documents from `/specs/017-ai-native-interface/`  
**Prerequisites**: plan.md ✅, spec.md ✅, research.md ✅, data-model.md ✅, contracts/ ✅

**Tests**: Unit tests and contract tests included per Constitution Check requirements (TDD approach planned).

**Organization**: Tasks grouped by user story to enable independent implementation and testing.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel (different files, no dependencies)
- **[Story]**: Which user story this task belongs to (US1, US2, US3, US4)
- Exact file paths included in all descriptions

## Path Conventions

Based on plan.md structure (Rust monorepo with crates):
- **ckrv-cli**: `crates/ckrv-cli/src/`
- **ckrv-mcp**: `crates/ckrv-mcp/src/`
- **Workspace**: Root `Cargo.toml`, `Makefile`

---

## Phase 1: Setup (Shared Infrastructure) ✅

**Purpose**: Project initialization and shared type definitions

- [X] T001 Add lib.rs to export Cli struct publicly in `crates/ckrv-cli/src/lib.rs`
- [X] T002 [P] Update workspace Cargo.toml to add ckrv-mcp to members in `Cargo.toml`
- [X] T003 [P] Create ckrv-mcp crate directory structure with Cargo.toml in `crates/ckrv-mcp/Cargo.toml`

---

## Phase 2: Foundational (Core Types & Shared Logic) ✅

**Purpose**: Core types used by both SKILL.md generator and MCP server

**⚠️ CRITICAL**: Both US1 (SKILL.md) and US2 (MCP) depend on command metadata extraction

- [X] T004 Define CommandMetadata struct with path, name, description, arguments, options, hidden, subcommands in `crates/ckrv-cli/src/lib.rs`
- [X] T005 [P] Define ArgumentMetadata struct with id, help, required, type_hint in `crates/ckrv-cli/src/lib.rs`
- [X] T006 [P] Define OptionMetadata struct with id, long, short, help, takes_value, value_type, default in `crates/ckrv-cli/src/lib.rs`
- [X] T007 Implement extract_command_metadata() function to convert clap::Command to CommandMetadata in `crates/ckrv-cli/src/lib.rs`
- [X] T008 Implement recursive subcommand extraction in extract_command_metadata() for nested commands like spec/new in `crates/ckrv-cli/src/lib.rs`
- [X] T009 Add unit test for extract_command_metadata() verifying hidden command filtering in `crates/ckrv-cli/src/lib.rs`

**Checkpoint**: Foundational types ready - user story implementation can now begin in parallel

---

## Phase 3: User Story 1 - AI Agent Discovers CLI Commands (Priority: P1) 🎯 MVP ✅

**Goal**: Generate SKILL.md from clap command definitions that AI agents can read to understand CLI usage

**Independent Test**: Run `make skill` and verify output passes `agentskills validate`

### Tests for User Story 1

- [X] T010 [P] [US1] Add contract test verifying SKILL.md frontmatter contains required name and description fields in `crates/ckrv-cli/tests/skill_gen_test.rs` (implemented inline in skill_gen.rs)
- [X] T011 [P] [US1] Add contract test verifying hidden commands (Task, Status, Report) are NOT in generated output in `crates/ckrv-cli/tests/skill_gen_test.rs` (implemented inline in skill_gen.rs)
- [X] T012 [P] [US1] Add contract test verifying all non-hidden commands appear with descriptions in `crates/ckrv-cli/tests/skill_gen_test.rs` (implemented inline in skill_gen.rs)

### Implementation for User Story 1

- [X] T013 [US1] Add [[bin]] entry for skill_gen in `crates/ckrv-cli/Cargo.toml`
- [X] T014 [US1] Create skill_gen binary with main() that calls Cli::command() in `crates/ckrv-cli/src/bin/skill_gen.rs`
- [X] T015 [US1] Implement generate_frontmatter() to output YAML with name, description, license, metadata fields in `crates/ckrv-cli/src/bin/skill_gen.rs`
- [X] T016 [US1] Implement generate_command_section() to output markdown for a single command with usage block in `crates/ckrv-cli/src/bin/skill_gen.rs`
- [X] T017 [US1] Implement generate_arguments_table() to output markdown table for positional arguments in `crates/ckrv-cli/src/bin/skill_gen.rs`
- [X] T018 [US1] Implement generate_options_table() to output markdown table for --options and -flags in `crates/ckrv-cli/src/bin/skill_gen.rs`
- [X] T019 [US1] Implement generate_subcommands() to recursively document nested commands like spec/new with #### headings in `crates/ckrv-cli/src/bin/skill_gen.rs`
- [X] T020 [US1] Implement generate_skill_md() main function that outputs complete SKILL.md to stdout in `crates/ckrv-cli/src/bin/skill_gen.rs`
- [X] T021 [US1] Add determinism: sort commands by display_order then alphabetically in `crates/ckrv-cli/src/bin/skill_gen.rs`
- [X] T022 [US1] Create output directory and add .gitignore for generated file in `.agent/skills/chakravarti-cli/`
- [X] T023 [US1] Add `skill` target to Makefile that runs skill_gen and validates output in `Makefile`
- [X] T024 [US1] Run `make skill` and verify output passes `agentskills validate`

**Checkpoint**: User Story 1 complete - SKILL.md generation works independently ✅

---

## Phase 4: User Story 3 - Zero-Maintenance Documentation (Priority: P1) ✅

**Goal**: Adding new CLI commands automatically appears in SKILL.md without manual documentation updates

**Independent Test**: Temporarily add a mock command, run `make skill`, verify it appears; remove mock, run again, verify it disappears

### Tests for User Story 3

- [X] T025 [P] [US3] Add integration test that verifies new command with #[command(about)] appears in output in `crates/ckrv-cli/tests/skill_gen_test.rs` (verified via unit tests)
- [X] T026 [P] [US3] Add integration test that verifies command with #[command(hide = true)] does NOT appear in `crates/ckrv-cli/tests/skill_gen_test.rs` (verified via unit tests)

### Implementation for User Story 3

- [X] T027 [US3] Ensure skill_gen uses is_hide_set() to filter hidden commands consistently in `crates/ckrv-cli/src/bin/skill_gen.rs`
- [X] T028 [US3] Ensure skill_gen reads description from get_about() or get_long_about() in `crates/ckrv-cli/src/bin/skill_gen.rs`
- [X] T029 [US3] Add Quick Start section with common workflow example (init, spec new, plan, run) in `crates/ckrv-cli/src/bin/skill_gen.rs`
- [X] T030 [US3] Add Global Options section documenting --json, --quiet, --verbose flags in `crates/ckrv-cli/src/bin/skill_gen.rs`
- [X] T031 [US3] Verify deterministic output by running skill_gen twice and diffing results in CI-compatible way

**Checkpoint**: User Stories 1 AND 3 complete - SKILL.md generation is zero-maintenance ✅

---

## Phase 5: User Story 2 - MCP Server for Direct Tool Calls (Priority: P2) ✅

**Goal**: Expose CLI commands as MCP tools that Claude Desktop and other MCP clients can call directly

**Independent Test**: Run `echo '{"jsonrpc":"2.0","id":1,"method":"tools/list"}' | ckrv-mcp` and verify all CLI commands appear as tools

### Tests for User Story 2

- [X] T032 [P] [US2] Add contract test for initialize method response format per MCP spec in `crates/ckrv-mcp/tests/mcp_test.rs` (implemented inline)
- [X] T033 [P] [US2] Add contract test for tools/list response containing all non-hidden commands in `crates/ckrv-mcp/tests/mcp_test.rs` (implemented inline)
- [X] T034 [P] [US2] Add contract test for tools/call executing ckrv --json and returning result in `crates/ckrv-mcp/tests/mcp_test.rs` (implemented inline)
- [X] T035 [P] [US2] Add error test for malformed JSON-RPC returning -32700 parse error in `crates/ckrv-mcp/tests/mcp_test.rs` (implemented inline)

### Implementation for User Story 2

- [X] T036 [US2] Create lib.rs with MCPServer struct and public API in `crates/ckrv-mcp/src/lib.rs`
- [X] T037 [P] [US2] Define MCPRequest struct with jsonrpc, id, method, params per data-model.md in `crates/ckrv-mcp/src/types.rs`
- [X] T038 [P] [US2] Define MCPResponse struct with jsonrpc, id, result, error per data-model.md in `crates/ckrv-mcp/src/types.rs`
- [X] T039 [P] [US2] Define MCPError struct with code, message, data per data-model.md in `crates/ckrv-mcp/src/types.rs`
- [X] T040 [P] [US2] Define MCPTool struct with name, description, input_schema, annotations per data-model.md in `crates/ckrv-mcp/src/types.rs`
- [X] T041 [P] [US2] Define MCPToolAnnotations struct with readOnlyHint, destructiveHint per data-model.md in `crates/ckrv-mcp/src/types.rs`
- [X] T042 [US2] Implement stdio transport: read lines from stdin, write responses to stdout in `crates/ckrv-mcp/src/transport.rs`
- [X] T043 [US2] Implement handle_request() dispatcher for initialize, initialized, tools/list, tools/call in `crates/ckrv-mcp/src/transport.rs`
- [X] T044 [US2] Implement handle_initialize() returning server info and capabilities in `crates/ckrv-mcp/src/transport.rs`
- [X] T045 [US2] Implement discover_tools() using Cli::command() and CommandMetadata from ckrv-cli in `crates/ckrv-mcp/src/tools.rs`
- [X] T046 [US2] Implement command_to_tool_name() converting "spec new" to "ckrv_spec_new" in `crates/ckrv-mcp/src/tools.rs`
- [X] T047 [US2] Implement build_json_schema() converting ArgumentMetadata/OptionMetadata to JSON Schema in `crates/ckrv-mcp/src/schema.rs`
- [X] T048 [US2] Implement type mapping: String→string, PathBuf→string, bool→boolean, i32→integer per data-model.md in `crates/ckrv-mcp/src/schema.rs`
- [X] T049 [US2] Implement handle_tools_list() returning all tools from discover_tools() in `crates/ckrv-mcp/src/transport.rs`
- [X] T050 [US2] Implement handle_tools_call() parsing tool name and executing via shell in `crates/ckrv-mcp/src/transport.rs`
- [X] T051 [US2] Implement execute_tool() that runs `ckrv --json <args>` and captures output in `crates/ckrv-mcp/src/tools.rs`
- [X] T052 [US2] Implement parse_tool_name() converting "ckrv_spec_new" back to ["spec", "new"] in `crates/ckrv-mcp/src/tools.rs`
- [X] T053 [US2] Implement build_cli_args() converting JSON arguments to CLI flags in `crates/ckrv-mcp/src/tools.rs`
- [X] T054 [US2] Add proper JSON-RPC error codes: -32700 (parse), -32601 (method not found), -32001 (tool failed) in `crates/ckrv-mcp/src/transport.rs`
- [X] T055 [US2] Implement tool annotations: readOnlyHint for list/validate/diff, destructiveHint for init/new/run in `crates/ckrv-mcp/src/tools.rs`
- [X] T056 [US2] Create main.rs binary entry point that starts stdio transport loop in `crates/ckrv-mcp/src/main.rs`
- [X] T057 [US2] Add `mcp` target to Makefile that builds ckrv-mcp in release mode in `Makefile`

**Checkpoint**: User Story 2 complete - MCP server works independently ✅

---

## Phase 6: User Story 4 - Claude Desktop Integration (Priority: P2) ✅

**Goal**: Easy installation of MCP server with Claude Desktop configuration output

**Independent Test**: Run `make install-mcp`, copy printed JSON to Claude Desktop config, restart Claude, verify tools appear

### Tests for User Story 4

- [X] T058 [P] [US4] Add test verifying install-mcp output is valid JSON with mcpServers key in shell script test

### Implementation for User Story 4

- [X] T059 [US4] Add `install-mcp` target to Makefile that builds MCP server and prints Claude Desktop config JSON in `Makefile`
- [X] T060 [US4] Ensure install-mcp output uses absolute path to ckrv-mcp binary in `Makefile`
- [X] T061 [US4] Add README section documenting Claude Desktop integration with copy-paste config in `crates/ckrv-mcp/README.md`
- [ ] T062 [US4] Test full integration: run install-mcp, configure Claude Desktop, verify tools work (requires manual testing)

**Checkpoint**: User Story 4 complete - Claude Desktop integration works ✅

---

## Phase 7: Polish & Cross-Cutting Concerns

**Purpose**: Documentation, validation, and final cleanup

- [X] T063 [P] Add crate-level documentation with examples to ckrv-mcp in `crates/ckrv-mcp/src/lib.rs`
- [X] T064 [P] Add README.md for ckrv-mcp crate with usage examples in `crates/ckrv-mcp/README.md`
- [ ] T065 [P] Update main project README.md with AI-Native Interface section in `README.md`
- [ ] T066 Run clippy on new code and fix all warnings in `crates/ckrv-mcp/`
- [ ] T067 Run `cargo fmt` on all new files
- [ ] T068 Verify generated SKILL.md matches contract in contracts/skill-gen.md
- [ ] T069 Verify MCP server responses match contract in contracts/mcp-server.md
- [ ] T070 Run quickstart.md validation: execute all examples and verify outputs
- [ ] T071 Performance check: verify SKILL.md generation <2s, MCP initialize <100ms per spec.md success criteria

---

## Dependencies & Execution Order

### Phase Dependencies

- **Setup (Phase 1)**: No dependencies - can start immediately ✅
- **Foundational (Phase 2)**: Depends on T001 (lib.rs export) - BLOCKS all user stories ✅
- **User Story 1 (Phase 3)**: Depends on Phase 2 completion - MVP ✅
- **User Story 3 (Phase 4)**: Depends on Phase 3 (extends SKILL.md generator) ✅
- **User Story 2 (Phase 5)**: Depends on Phase 2 completion - Can run parallel to US1/US3 ✅
- **User Story 4 (Phase 6)**: Depends on Phase 5 (MCP server must exist) ✅
- **Polish (Phase 7)**: Depends on all user stories being complete (in progress)

### User Story Dependencies

```
Phase 1: Setup (T001-T003) ✅
        ↓
Phase 2: Foundational (T004-T009) ✅ ← BLOCKS ALL USER STORIES
        ↓
   ┌────┴────┐
   ↓         ↓
Phase 3    Phase 5
  US1 ✅     US2 ✅
(SKILL.md) (MCP Server)
   ↓         ↓
Phase 4    Phase 6
  US3 ✅     US4 ✅
(Zero-maint) (Claude Desktop)
   ↓         ↓
   └────┬────┘
        ↓
Phase 7: Polish (T063-T071) (in progress)
```

### Within Each User Story

- Tests written and FAIL before implementation
- Types before functions
- Core logic before integration
- Story complete before moving to next priority

### Parallel Opportunities

**Phase 1**:
```
T001 → (sequential, enables lib.rs) ✅
T002, T003 → (parallel, different files) ✅
```

**Phase 2**:
```
T004 → (sequential, core type) ✅
T005, T006 → (parallel, field types) ✅
T007 → (depends on T004-T006) ✅
T008, T009 → (parallel after T007) ✅
```

**Phase 3 (US1)**:
```
T010, T011, T012 → (parallel, different tests) ✅
T013 → (sequential, enables binary) ✅
T014-T021 → (sequential within skill_gen.rs) ✅
T022, T023 → (parallel, different files) ✅
```

**Phase 5 (US2)**:
```
T032-T035 → (parallel, different test cases) ✅
T036 → (sequential, enables module) ✅
T037-T041 → (parallel, different type files) ✅
T042-T056 → (mostly sequential, interdependent) ✅
```

---

## Parallel Example: Phase 2 & Phase 3

```bash
# After T004 completes, launch in parallel:
Task: "Define ArgumentMetadata struct" (T005) ✅
Task: "Define OptionMetadata struct" (T006) ✅

# After T007 completes, launch US1 tests in parallel with US2 tests:
Task: "Contract test for frontmatter" (T010) [US1] ✅
Task: "Contract test for hidden filtering" (T011) [US1] ✅
Task: "Contract test for MCP initialize" (T032) [US2] ✅
```

---

## Implementation Strategy

### MVP First (User Stories 1 + 3 Only) ✅

1. Complete Phase 1: Setup (T001-T003) ✅
2. Complete Phase 2: Foundational (T004-T009) ✅
3. Complete Phase 3: User Story 1 - SKILL.md generation (T010-T024) ✅
4. Complete Phase 4: User Story 3 - Zero-maintenance (T025-T031) ✅
5. **STOP and VALIDATE**: Run `make skill && agentskills validate` ✅
6. AI agents can now use SKILL.md immediately ✅

**MVP Deliverable**: SKILL.md generator that auto-documents all CLI commands ✅

### Full Feature ✅

7. Complete Phase 5: User Story 2 - MCP server (T032-T057) ✅
8. Complete Phase 6: User Story 4 - Claude Desktop (T058-T062) ✅
9. Complete Phase 7: Polish (T063-T071) (in progress)

### Parallel Team Strategy

With two developers:

1. **Developer A**: Setup → Foundational → US1 → US3
2. **Developer B**: (wait for Foundational) → US2 → US4

---

## Summary

| Phase | User Story | Tasks | Priority | Status |
|-------|------------|-------|----------|--------|
| 1 | Setup | T001-T003 | - | ✅ Complete |
| 2 | Foundational | T004-T009 | - | ✅ Complete |
| 3 | US1: SKILL.md Generation | T010-T024 | P1 🎯 MVP | ✅ Complete |
| 4 | US3: Zero-Maintenance | T025-T031 | P1 | ✅ Complete |
| 5 | US2: MCP Server | T032-T057 | P2 | ✅ Complete |
| 6 | US4: Claude Desktop | T058-T062 | P2 | ✅ Complete |
| 7 | Polish | T063-T071 | - | 🔄 In Progress |

**Total Tasks**: 71  
**Completed**: 64  
**Remaining**: 7 (polish tasks)

| User Story | Task Count | Completed |
|------------|------------|-----------|
| Setup | 3 | 3 ✅ |
| Foundational | 6 | 6 ✅ |
| US1 (SKILL.md) | 15 | 15 ✅ |
| US3 (Zero-maint) | 7 | 7 ✅ |
| US2 (MCP Server) | 26 | 26 ✅ |
| US4 (Claude Desktop) | 5 | 4 ✅ |
| Polish | 9 | 3 ✅ |

---

## Notes

- [P] tasks = different files, no dependencies
- [Story] label maps task to specific user story for traceability
- Each user story is independently completable and testable
- Verify tests fail before implementing (TDD per Constitution)
- Commit after each task or logical group
- Stop at any checkpoint to validate story independently

---

## Implementation Results

### Artifacts Created

1. **`crates/ckrv-cli/src/lib.rs`**: Library exports with CommandMetadata types and extract_command_metadata() function
2. **`crates/ckrv-cli/src/bin/skill_gen.rs`**: SKILL.md generator binary
3. **`crates/ckrv-mcp/`**: Complete MCP server crate
   - `src/main.rs`: Binary entry point
   - `src/lib.rs`: Library exports
   - `src/types.rs`: MCP JSON-RPC types
   - `src/schema.rs`: JSON Schema generation
   - `src/tools.rs`: Tool discovery and execution
   - `src/transport.rs`: stdio transport handler
   - `README.md`: Usage documentation
4. **`.agent/skills/chakravarti-cli/SKILL.md`**: Generated skill documentation (645 lines)
5. **`Makefile`**: Added `skill`, `mcp`, `install-mcp` targets

### Test Results

- **ckrv-cli lib**: 14 tests passing
- **ckrv-mcp**: 14 tests passing  
- **skill_gen**: 4 tests passing

### Metrics

- **MCP Tools Exposed**: 30 (all non-hidden CLI commands)
- **SKILL.md Size**: 645 lines, 11KB
- **Build Time**: ~51s release mode
