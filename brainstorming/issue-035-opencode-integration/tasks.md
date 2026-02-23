# Add OpenCode Integration - Tasks

**Issue**: [#35](https://github.com/FnSK4R17s/chakravarti-cli/issues/35)
**Brainstorm**: [notes.md](./notes.md)
**Created**: 2026-02-23

## Task Overview

| Phase | Tasks | Estimate |
|-------|-------|----------|
| Phase 1: Research | 2 | 1h |
| Phase 2: Implementation | 4 | 4h |
| Phase 3: Testing & Docs | 2 | 2h |
| **Total** | 8 | 7h |

---

## Phase 1: Research

### Task 1.1: Verify OpenCode CLI headless behavior
**Priority**: P1
**Estimate**: 30m
**Files**: None (research only)

Verify that `opencode run` command works without TUI interaction:
- Check if daemon/server is required
- Test basic execution with `--help`
- Document exit code behavior

**Acceptance Criteria**:
- [ ] Document whether OpenCode requires a running daemon
- [ ] Document CLI flags for non-interactive mode
- [ ] Note: Can be done in parallel with Task 1.2

---

### Task 1.2: Verify JSON output format
**Priority**: P1
**Estimate**: 30m
**Files**: None (research only)

Investigate JSON output options for parsing:
- Check for `--json` or similar flags
- Verify structured output format
- Document parsing requirements

**Acceptance Criteria**:
- [ ] Document available output formats
- [ ] Determine if JSON parsing is feasible for AgentOutput

---

## Phase 2: Implementation

### Task 2.1: Add OpenCode variant to AgentType enum
**Priority**: P0
**Estimate**: 15m
**Files**: `crates/ckrv-sandbox/src/agent/mod.rs`

Add `OpenCode` variant to the `AgentType` enum and implement:
- `from_str()` parsing for "opencode" 
- `display_name()` returning "OpenCode"

**Acceptance Criteria**:
- [ ] `AgentType::OpenCode` variant exists
- [ ] `AgentType::from_str("opencode")` returns `Some(AgentType::OpenCode)`
- [ ] Display name shows "OpenCode"
- [ ] `cargo check -p ckrv-sandbox` passes

---

### Task 2.2: Create OpenCodeProvider implementation
**Priority**: P0
**Estimate**: 2h
**Files**: `crates/ckrv-sandbox/src/agent/opencode.rs` (new)

Create new file implementing `AgentProvider` trait:
- Structure follows existing pattern in `claude.rs`
- Command: `opencode run "prompt" --project /path`
- Config mounts for `~/.opencode/` if exists
- Parse output (success = exit_code == 0)

**Acceptance Criteria**:
- [ ] `OpenCodeProvider` struct created
- [ ] Implements `AgentProvider` trait fully
- [ ] `build_command()` constructs valid CLI args
- [ ] `required_env_vars()` returns OPENCODE_API_KEY or similar
- [ ] `config_mounts()` mounts ~/.opencode/ if present

---

### Task 2.3: Add OpenCodeProvider to module exports
**Priority**: P0
**Estimate**: 15m
**Files**: `crates/ckrv-sandbox/src/agent/mod.rs`

Update the agent module:
- Add `mod opencode;`
- Add `pub use opencode::OpenCodeProvider;`
- Add `AgentType::OpenCode` case to `create_agent()` factory

**Acceptance Criteria**:
- [ ] Module compiles without errors
- [ ] `create_agent(AgentType::OpenCode)` returns `OpenCodeProvider`
- [ ] `cargo check -p ckrv-sandbox` passes

---

### Task 2.4: Support model override in command
**Priority**: P2
**Estimate**: 30m
**Files**: `crates/ckrv-sandbox/src/agent/opencode.rs`

Add `--model` flag support to `build_command()`:
- Use `--model provider/model` syntax
- Handle config.model Option

**Acceptance Criteria**:
- [ ] `--model` flag added when config.model is set
- [ ] Test: verify command includes model flag

---

## Phase 3: Testing & Docs

### Task 3.1: Add unit tests for OpenCodeProvider
**Priority**: P1
**Estimate**: 1h
**Files**: `crates/ckrv-sandbox/src/agent/tests/` (or inline)

Add tests following existing pattern:
- Test `build_command()` output
- Test `from_str()` parsing
- Test `display_name()`

**Acceptance Criteria**:
- [ ] Unit tests pass: `cargo test -p ckrv-sandbox`
- [ ] At least 3 test cases added

---

### Task 3.2: Update agent documentation
**Priority**: P2
**Estimate**: 1h
**Files**: `crates/docs/agent-guide.md`

Document the new agent:
- Add OpenCode to supported agents list
- Show example `agents.yaml` configuration
- Document CLI requirements

**Acceptance Criteria**:
- [ ] Documentation updated with OpenCode section
- [ ] Example config provided

---

## Dependencies

```
Phase 1 ─────────────────────────────────────────────────►
  Task 1.1 ──►
  Task 1.2 ──►
              │
Phase 2 ──────┼────────────────────────────────────────────►
              ├──► Task 2.1 ──► Task 2.2 ──► Task 2.3 ──► Task 2.4
              │
Phase 3 ──────┴────────────────────────────────────────────►
                                └──► Task 3.1 ──► Task 3.2
```

## Notes

- Task 2.2 (OpenCodeProvider) is the core implementation - estimate 2h for first pass
- Task 2.4 (model override) can be deferred if OpenCode doesn't support model flag
- Phase 1 research tasks can run in parallel with initial implementation work
