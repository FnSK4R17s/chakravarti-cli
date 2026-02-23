# Add Cursor CLI integration - Tasks

**Issue**: [#32](https://github.com/FnSK4R17s/chakravarti-cli/issues/32)
**Brainstorm**: [notes.md](./notes.md)
**Created**: 2026-02-23

## Task Overview

| Phase | Tasks | Estimate |
|-------|-------|----------|
| Phase 1: Discovery & Contract | 3 | 3h |
| Phase 2: Provider Implementation | 4 | 5h |
| Phase 3: Runtime & Validation | 3 | 4h |
| Phase 4: QA & Docs | 3 | 3h |
| **Total** | **13** | **15h** |

---

## Phase 1: Discovery & Contract

### Task 1.1: Confirm Cursor CLI invocation contract
**Priority**: P0
**Estimate**: 1.5h

Document exact Cursor binary name, non-interactive execution flags, working directory argument, and model flag support for the version we intend to support.

**Acceptance Criteria**:
- [ ] Known-good command example captured in notes/docs
- [ ] Version compatibility baseline documented

### Task 1.2: Confirm authentication mechanism
**Priority**: P0
**Estimate**: 1h

Determine whether Cursor auth is env-based, file-based, or hybrid in containerized runs.

**Acceptance Criteria**:
- [ ] Auth dependency matrix documented (env/config paths)
- [ ] Required mounts/env list finalized

### Task 1.3: Decide runtime image strategy
**Priority**: P1
**Estimate**: 0.5h

Choose between extending an existing image vs dedicated Cursor image.

**Acceptance Criteria**:
- [ ] Chosen strategy recorded with rationale

---

## Phase 2: Provider Implementation

### Task 2.1: Add `AgentType::Cursor` + parser aliases
**Priority**: P0
**Estimate**: 0.5h

Update `AgentType` enum and `from_str` mapping for `cursor` / `cursor-cli` aliases.

**Acceptance Criteria**:
- [ ] Enum variant added
- [ ] Alias parsing tests pass

### Task 2.2: Implement `CursorProvider`
**Priority**: P0
**Estimate**: 2.5h

Create `crates/ckrv-sandbox/src/agent/cursor.rs` implementing `AgentProvider` methods.

**Acceptance Criteria**:
- [ ] `build_command` generates expected command vector
- [ ] `config_mounts` includes required Cursor config path(s)
- [ ] `parse_output` aligns with existing provider semantics

### Task 2.3: Register provider factory and module exports
**Priority**: P0
**Estimate**: 0.5h

Wire provider into module tree and factory creation path.

**Acceptance Criteria**:
- [ ] `create_agent(AgentType::Cursor)` returns `CursorProvider`

### Task 2.4: Add provider unit tests
**Priority**: P1
**Estimate**: 1.5h

Add focused tests for command building, mount setup, and parser behavior.

**Acceptance Criteria**:
- [ ] Provider tests pass locally
- [ ] Edge-case coverage for missing config

---

## Phase 3: Runtime & Validation

### Task 3.1: Ensure Cursor CLI availability in sandbox runtime
**Priority**: P0
**Estimate**: 2h

Update/install path so runner image contains Cursor CLI and executes as non-root.

**Acceptance Criteria**:
- [ ] Image build installs Cursor CLI successfully
- [ ] Runtime can execute `cursor --version` (or canonical binary)

### Task 3.2: Add startup/runtime validation errors
**Priority**: P0
**Estimate**: 1h

Surface actionable errors when Cursor binary or auth/config is missing.

**Acceptance Criteria**:
- [ ] Missing binary error is explicit
- [ ] Missing auth/config error includes remediation hints

### Task 3.3: Smoke test end-to-end task run
**Priority**: P1
**Estimate**: 1h

Run a minimal task with Cursor provider and verify normalized output + exit behavior.

**Acceptance Criteria**:
- [ ] One successful sample run captured
- [ ] Failure mode sampled for observability

---

## Phase 4: QA & Docs

### Task 4.1: Update docs for Cursor setup
**Priority**: P1
**Estimate**: 1h

Update `crates/docs/agent-guide.md` (or provider docs) with Cursor config/auth/runtime notes.

**Acceptance Criteria**:
- [ ] Cursor section added with YAML example
- [ ] Troubleshooting notes included

### Task 4.2: Add regression checklist
**Priority**: P2
**Estimate**: 1h

Capture checklist ensuring new providers maintain parity (command shape, auth, mounts, output).

**Acceptance Criteria**:
- [ ] Checklist added to relevant docs/workflow

### Task 4.3: Final verification run
**Priority**: P1
**Estimate**: 1h

Run targeted tests + lint for modified crates and document results.

**Acceptance Criteria**:
- [ ] Relevant tests pass
- [ ] No new clippy/rustfmt regressions in changed files

---

## Dependencies

```text
1.1 + 1.2 + 1.3
      ↓
2.1 → 2.2 → 2.3 → 2.4
      ↓
3.1 → 3.2 → 3.3
      ↓
4.1 + 4.2 + 4.3
```

## Blockers

- [x] Unable to run local `claude` binary in this environment (`/usr/local/bin/claude` symlink resolves to `/root/.claude/local/bin/claude`, execution denied for `node` user). Brainstorming was produced directly without CLI-assisted planning.
- [ ] Cursor CLI invocation/auth contract still needs confirmation against the exact supported Cursor CLI release.
