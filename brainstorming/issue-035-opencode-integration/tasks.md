# Add Opencode integration - Tasks

**Issue**: [#35](https://github.com/FnSK4R17s/chakravarti-cli/issues/35)
**Brainstorm**: [notes.md](./notes.md)
**Created**: 2026-02-23

## Task Overview

| Phase | Tasks | Estimate |
|-------|-------|----------|
| Phase 1: Discovery & contract lock | 3 | 2-3h |
| Phase 2: Provider implementation | 4 | 4-6h |
| Phase 3: Validation & docs | 4 | 2-4h |
| **Total** | **11** | **8-13h** |

---

## Phase 1: Discovery & Contract Lock

### Task 1.1: Confirm Opencode CLI non-interactive command
**Priority**: P0
**Estimate**: 45m

Validate exact invocation pattern, required flags, and expected output format for automation.

**Acceptance Criteria**:
- [ ] Canonical command documented in notes and/or code comments
- [ ] Error code semantics understood (success/failure)

---

### Task 1.2: Confirm authentication and config locations
**Priority**: P0
**Estimate**: 45m

Identify whether Opencode uses env vars, config files, or both, and which paths require sandbox mounts.

**Acceptance Criteria**:
- [ ] Auth/config path list documented
- [ ] Mount policy decision (RO/RW) made

---

### Task 1.3: Decide container strategy
**Priority**: P1
**Estimate**: 1h

Choose between extending shared agent image vs dedicated `Dockerfile.opencode`.

**Acceptance Criteria**:
- [ ] Decision recorded with rationale
- [ ] Build impact/tradeoff documented

---

## Phase 2: Provider Implementation

### Task 2.1: Create `OpencodeProvider`
**Priority**: P0
**Estimate**: 2h

Add provider implementation in `crates/ckrv-sandbox/src/agent/opencode.rs`.

**Acceptance Criteria**:
- [ ] Implements `AgentProvider` trait methods
- [ ] Command builder supports prompt + optional model
- [ ] Required env/mounts declared

---

### Task 2.2: Wire provider into registry/factory
**Priority**: P0
**Estimate**: 1h

Update provider enum and factory wiring in `agent/mod.rs`.

**Acceptance Criteria**:
- [ ] `AgentType::Opencode` available
- [ ] Factory creates `OpencodeProvider` correctly

---

### Task 2.3: Update CLI lookup/config parsing
**Priority**: P0
**Estimate**: 45m

Ensure CLI/service layer recognizes Opencode as valid agent type.

**Acceptance Criteria**:
- [ ] `agent_lookup` supports `opencode`
- [ ] Invalid type errors remain clear and unchanged for other types

---

### Task 2.4: Add docker/config mounts
**Priority**: P1
**Estimate**: 1h

Add Opencode-specific mount behavior if needed.

**Acceptance Criteria**:
- [ ] Mount paths added only if required
- [ ] Follows existing security/mount conventions

---

## Phase 3: Validation & Documentation

### Task 3.1: Add provider unit tests
**Priority**: P0
**Estimate**: 1h

Cover command generation and registration.

**Acceptance Criteria**:
- [ ] Tests for default command path
- [ ] Tests for model override path
- [ ] Tests for provider factory selection

---

### Task 3.2: Add smoke test path (if available)
**Priority**: P1
**Estimate**: 1h

Add/extend integration test for provider execution stub or dry-run.

**Acceptance Criteria**:
- [ ] Smoke test executed in CI-compatible way (or clearly marked ignored)

---

### Task 3.3: Update docs
**Priority**: P1
**Estimate**: 45m

Document setup, required auth, configuration example, and troubleshooting.

**Acceptance Criteria**:
- [ ] `agent-guide.md` includes Opencode section
- [ ] CLI docs updated if needed

---

### Task 3.4: Final verification checklist
**Priority**: P1
**Estimate**: 30m

Run final build/tests and capture results.

**Acceptance Criteria**:
- [ ] `cargo build --workspace` passes
- [ ] Relevant tests pass
- [ ] Example config snippet validated

---

## Dependencies

```text
1.1 + 1.2 + 1.3 ──→ 2.1 ──→ 2.2 + 2.3 + 2.4 ──→ 3.1 + 3.2 + 3.3 ──→ 3.4
```

## Blockers

- [x] Planning CLI blocker: `claude` command execution is unavailable in this environment (`Permission denied`), so Claude Code could not be invoked as the planner tool.
- [ ] Need confirmed Opencode CLI contract/auth details before implementation starts.

## Notes

- This task file is implementation-ready and intentionally scoped to issue #35.
- Brainstorm focuses on architecture parity with existing providers (no special-case executor path).
