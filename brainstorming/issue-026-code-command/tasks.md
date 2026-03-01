# Organise all code related tasks under the `ckrv code` command - Tasks

**Issue**: [#26](https://github.com/FnSK4R17s/chakravarti-cli/issues/26)
**Brainstorm**: [notes.md](./notes.md)
**Created**: 2026-02-23

## Task Overview

| Phase | Tasks | Estimate |
|-------|-------|----------|
| Phase 1: Scope and command contract | 3 | 4h |
| Phase 2: CLI wiring and routing | 4 | 7h |
| Phase 3: Docs and UX parity updates | 3 | 4h |
| Phase 4: Tests and rollout safety | 3 | 4h |
| **Total** | **13** | **19h** |

---

## Phase 1: Scope and command contract

### Task 1.1: Freeze `ckrv code` v1 subcommand scope
**Priority**: P0
**Estimate**: 1h
**Files**: `brainstorming/issue-026-code-command/notes.md`

Lock V1 scope to `spec`, `tasks`, `plan`, `run` and explicitly decide whether `diff` is included in V1 or deferred.

**Acceptance Criteria**:
- [ ] Notes contain explicit V1 scope and explicit `diff` decision.
- [ ] Notes explicitly state `verify/fix/test/qa` are out of scope for this issue.
- [ ] Open question list is updated to reflect final scope decisions.

---

### Task 1.2: Define behavior contract for `ckrv code tasks`
**Priority**: P0
**Estimate**: 2h
**Files**: `brainstorming/issue-026-code-command/notes.md`, `crates/docs/cli-commands.md`

Specify whether `ckrv code tasks` is a thin alias for `ckrv spec tasks` or a dedicated tasks surface with extensibility for future operations.

**Acceptance Criteria**:
- [ ] Contract documents exact command behavior and flags for `ckrv code tasks`.
- [ ] Behavior parity with `ckrv spec tasks` is documented for V1.
- [ ] Any intentionally unsupported behavior is explicitly documented.

---

### Task 1.3: Define compatibility/deprecation policy
**Priority**: P1
**Estimate**: 1h
**Files**: `brainstorming/issue-026-code-command/notes.md`, `crates/docs/cli-commands.md`

Document how long top-level commands (`spec`, `plan`, `run`, optional `diff`) remain supported and how migration hints appear.

**Acceptance Criteria**:
- [ ] Policy states that legacy top-level commands continue to work in initial rollout.
- [ ] Policy defines where migration guidance appears (help/docs/changelog).
- [ ] Policy includes a concrete deprecation window trigger (release count or date).

---

## Phase 2: CLI wiring and routing

### Task 2.1: Add `code` command module and args schema
**Priority**: P0
**Estimate**: 2h
**Files**: `crates/ckrv-cli/src/commands/code.rs`, `crates/ckrv-cli/src/commands/mod.rs`

Create the new command module with Clap argument types and subcommand enum for `spec/tasks/plan/run` (and `diff` if approved in Task 1.1).

**Acceptance Criteria**:
- [ ] `commands/code.rs` compiles and exposes `CodeArgs`/subcommand types.
- [ ] `commands/mod.rs` exports the `code` module.
- [ ] Each new command has `long_about` and `after_help` where required by project conventions.

---

### Task 2.2: Register top-level `Code` command in CLI enum
**Priority**: P0
**Estimate**: 2h
**Files**: `crates/ckrv-cli/src/lib.rs`

Add `Commands::Code(...)` to the top-level Clap command tree with consistent display ordering and help text.

**Acceptance Criteria**:
- [ ] `ckrv --help` shows a new top-level `code` command.
- [ ] `ckrv code --help` lists expected V1 subcommands.
- [ ] Existing hidden command behavior (`task/status/report`) is unchanged.

---

### Task 2.3: Wire `code` command execution in main dispatcher
**Priority**: P0
**Estimate**: 1h
**Files**: `crates/ckrv-cli/src/main.rs`

Route `Commands::Code(args)` to the new execution path and keep existing top-level execution branches intact.

**Acceptance Criteria**:
- [ ] `main.rs` matches on `Commands::Code` and delegates to code executor.
- [ ] Existing command dispatch (`spec/plan/run/...`) remains functional.
- [ ] `cargo check -p ckrv-cli` passes.

---

### Task 2.4: Implement thin delegation to existing handlers
**Priority**: P1
**Estimate**: 2h
**Files**: `crates/ckrv-cli/src/commands/code.rs`, `crates/ckrv-cli/src/commands/spec.rs`, `crates/ckrv-cli/src/commands/plan.rs`, `crates/ckrv-cli/src/commands/run.rs`, `crates/ckrv-cli/src/commands/diff.rs`

Ensure `ckrv code *` executes through existing command logic to preserve behavior and output semantics.

API safety requirement: this task must not change external API contracts (HTTP routes, request/response shapes, Tauri invoke names).

**Acceptance Criteria**:
- [ ] `ckrv code spec ...` delegates to existing spec flow.
- [ ] `ckrv code tasks` matches `ckrv spec tasks` behavior.
- [ ] `ckrv code plan` and `ckrv code run` match existing behavior.
- [ ] `ckrv code diff` behavior is implemented only if approved in scope.
- [ ] Existing API-facing wrappers still function without payload/schema changes:
  - `/api/command/spec-new`, `/api/command/spec-tasks`, `/api/command/plan`, `/api/command/execute`, `/api/command/diff`
  - Tauri IPC: `run_spec_new`, `run_spec_tasks`, `run_plan`, `run_execute`, `run_diff`

---

## Phase 3: Docs and UX parity updates

### Task 3.1: Update canonical CLI documentation
**Priority**: P1
**Estimate**: 2h
**Files**: `crates/docs/cli-commands.md`

Document `ckrv code` as the preferred workflow entry and map it to Code page tabs.

**Acceptance Criteria**:
- [ ] `cli-commands.md` includes `ckrv code` section with V1 subcommands.
- [ ] Legacy top-level forms are marked as compatibility paths (not deprecated unless policy says so).
- [ ] Examples prefer `ckrv code ...` for Code workflow actions.

---

### Task 3.2: Update cross-doc command examples
**Priority**: P2
**Estimate**: 1h
**Files**: `crates/docs/architecture.md`, `crates/docs/agent-guide.md`, `README.md`

Refresh high-visibility examples that currently show top-level `spec/plan/run` paths where Code workflow naming is intended.

**Acceptance Criteria**:
- [ ] Cross-doc examples for Code workflow prefer `ckrv code ...`.
- [ ] No references mistakenly move Test/QA workflows under `ckrv code`.
- [ ] Example commands remain internally consistent with implemented CLI behavior.

---

### Task 3.3: Align UI command hints with new CLI taxonomy
**Priority**: P2
**Estimate**: 1h
**Files**: `crates/ckrv-ui/frontend/src/components/WorkflowPanel.tsx`, `crates/ckrv-ui/frontend/src/components/CommandPalette.tsx`

Update displayed command hints/snippets to reduce mismatch between UI workflow tabs and CLI naming. Keep this as a display-text pass only; existing API route wiring stays unchanged.

**Acceptance Criteria**:
- [ ] Code workflow hints use `ckrv code ...` where appropriate.
- [ ] No new `ckrv code ...` hints are introduced in Test/QA workflows.
- [ ] No regressions in page navigation or command trigger wiring.

---

## Phase 4: Tests and rollout safety

### Task 4.1: Add parser/metadata tests for `code` command
**Priority**: P1
**Estimate**: 2h
**Files**: `crates/ckrv-cli/src/lib.rs`, `crates/ckrv-transport/src/axum/commands.rs`, `crates/ckrv-ui/frontend/src/lib/api.ts`, `crates/ckrv-tauri/src/main.rs`

Extend existing command metadata tests to verify `code` appears and expected subcommands are exposed, and add API contract checks so command namespace changes do not alter API surface.

**Acceptance Criteria**:
- [ ] Test asserts `code` is present and visible in extracted metadata.
- [ ] Test asserts expected `code` subcommands are present.
- [ ] Existing metadata tests remain green.
- [ ] API contract checks confirm command endpoints remain stable in web + desktop flows:
  - `/api/command/*` routes in Axum remain unchanged
  - endpoint-to-command mapping in `frontend/src/lib/api.ts` remains valid
  - Tauri invoke registration in `crates/ckrv-tauri/src/main.rs` keeps existing `run_*` command names

---

### Task 4.2: Add compatibility tests for legacy command paths
**Priority**: P1
**Estimate**: 1h
**Files**: `crates/ckrv-cli/src/lib.rs`

Verify old top-level commands continue parsing while new `code` namespace exists.

**Acceptance Criteria**:
- [ ] Tests cover both `ckrv plan` and `ckrv code plan` parse paths.
- [ ] Tests cover both `ckrv run` and `ckrv code run` parse paths.
- [ ] If in scope, tests cover both `ckrv diff` and `ckrv code diff` parse paths.

---

### Task 4.3: End-to-end verification and release notes
**Priority**: P2
**Estimate**: 1h
**Files**: `crates/docs/cli-commands.md`, `README.md`

Run sanity checks and capture migration notes for users.

**Acceptance Criteria**:
- [ ] `cargo check -p ckrv-cli` succeeds.
- [ ] `cargo test -p ckrv-cli` succeeds.
- [ ] `cargo check -p ckrv-transport` succeeds.
- [ ] `cargo check -p ckrv-tauri` succeeds.
- [ ] Help output sanity checks pass for `ckrv --help` and `ckrv code --help`.
- [ ] API smoke checks are run for both transports (web Axum and Tauri IPC) with baseline/non-regression tracking:
  - `POST /api/command/spec-new`
  - `POST /api/command/spec-tasks`
  - `POST /api/command/plan`
  - `POST /api/command/execute`
  - `POST /api/command/diff`
  - Tauri invokes: `run_spec_new`, `run_spec_tasks`, `run_plan`, `run_execute`, `run_diff`
- [ ] Any pre-existing endpoint failures are documented before implementation and verified as not worsened after implementation.
- [ ] Documentation/release notes include migration guidance.

---

## Dependencies

```text
Task 1.1 ──┬──→ Task 2.1 ──→ Task 2.2 ──→ Task 2.3 ──→ Task 2.4
           ├──→ Task 1.2 ────────────────────────────────┘
           └──→ Task 1.3 ──→ Task 3.1 ──→ Task 3.2

Task 2.2 + Task 2.3 + Task 2.4 ──→ Task 3.3

Task 2.2 + Task 2.3 + Task 2.4 ──→ Task 4.1 ──→ Task 4.2 ──→ Task 4.3
Task 3.1 ────────────────────────────────────────────────────────────┘
```

## Blockers

- [ ] Decision pending on whether `ckrv code diff` is included in V1.
- [ ] Decision pending on whether `ckrv code tasks` is alias-only in V1 or expandable surface.

## Notes

- Keep `verify`, `fix`, `test`, and `qa` out of `ckrv code` scope for this issue.
- Preserve existing top-level command behavior during initial migration window.
- Treat API contract as frozen for this issue:
  - No route renames under `/api/command/*`
  - No request/response schema changes for existing command endpoints
  - No Tauri IPC command renames for existing `run_*` command wrappers
- Follow `crates/RUST_CONVENTIONS.md` for all Clap command docs (`long_about`, `after_help`).
