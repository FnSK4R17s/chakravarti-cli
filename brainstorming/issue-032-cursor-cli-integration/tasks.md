# Add Cursor CLI integration - Tasks

**Issue**: [#32](https://github.com/FnSK4R17s/chakravarti-cli/issues/32)
**Brainstorm**: [notes.md](./notes.md)
**Created**: 2026-02-23

## Task Overview

| Phase | Tasks | Estimate |
|-------|-------|----------|
| Phase 1: Agent Model + Config | 3 | 3h |
| Phase 2: Term Spawn Integration | 4 | 5h |
| Phase 3: UX, Docs, Validation | 3 | 3h |
| **Total** | **10** | **11h** |

---

## Phase 1: Agent Model + Config

### Task 1.1: Add Cursor agent type
**Priority**: P0
**Estimate**: 1h

Add `Cursor` variant to the agent type enum and ensure serialization/deserialization supports `cursor` in YAML config.

**Acceptance Criteria**:
- [ ] `AgentType` includes `Cursor` variant.
- [ ] YAML `agent_type: cursor` parses without errors.
- [ ] Existing agent types remain backward-compatible.

---

### Task 1.2: Extend agent config validation + lookup
**Priority**: P0
**Estimate**: 1h

Ensure agent loading and validation paths treat Cursor as a valid enabled agent type with no regressions.

**Acceptance Criteria**:
- [ ] Cursor agents appear in loaded config when enabled.
- [ ] Invalid/missing Cursor config emits clear errors.
- [ ] No changes required to existing configs unless opting into Cursor.

---

### Task 1.3: Add sample config snippet for Cursor
**Priority**: P1
**Estimate**: 1h

Provide a documented configuration example for Cursor agent entries (including optional `binary_path` and `extra_args`).

**Acceptance Criteria**:
- [ ] Example YAML snippet included in docs/help surface.
- [ ] Example validated against parser expectations.

---

## Phase 2: Term Spawn Integration

### Task 2.1: Implement command builder support for Cursor
**Priority**: P0
**Estimate**: 2h

Update term command construction to resolve Cursor binary and launch similarly to other agent types.

**Acceptance Criteria**:
- [ ] Cursor command path resolves from `binary_path` or `cursor` fallback.
- [ ] Spawn path executes without special-case crashes.
- [ ] Error message is actionable if Cursor binary is unavailable.

---

### Task 2.2: Wire passthrough args and extra args
**Priority**: P0
**Estimate**: 1h

Ensure `ckrv term -- ...` passthrough and configured `extra_args` are applied for Cursor launches.

**Acceptance Criteria**:
- [ ] Passthrough args reach Cursor process intact.
- [ ] Configured `extra_args` are appended in expected order.
- [ ] Behavior matches established patterns for other agents.

---

### Task 2.3: Include Cursor in list/selection UX
**Priority**: P1
**Estimate**: 1h

Make sure interactive/non-interactive listing surfaces Cursor as a selectable provider with clear label.

**Acceptance Criteria**:
- [ ] `ckrv term --list` includes Cursor entries.
- [ ] Interactive selection label is recognizable.
- [ ] No UI regressions in current menu flow.

---

### Task 2.4: Add smoke test path for spawn behavior
**Priority**: P1
**Estimate**: 1h

Add or update tests/smoke checks for command construction and launch path with Cursor agent type.

**Acceptance Criteria**:
- [ ] At least one test/smoke path covers Cursor command construction.
- [ ] CI/local test suite remains green for touched crates.

---

## Phase 3: UX, Docs, Validation

### Task 3.1: Update CLI help text + docs
**Priority**: P1
**Estimate**: 1h

Reflect Cursor support in user-facing docs and command help where agent options are enumerated.

**Acceptance Criteria**:
- [ ] Help text references Cursor where relevant.
- [ ] Documentation includes configuration and launch examples.

---

### Task 3.2: Validate end-to-end local workflow
**Priority**: P1
**Estimate**: 1h

Run a local flow using a sample Cursor agent config and verify launch/listing behavior.

**Acceptance Criteria**:
- [ ] `ckrv term --list` shows Cursor.
- [ ] `ckrv term --agent <cursor-id>` launches expected binary.
- [ ] Failure mode tested when binary is missing.

---

### Task 3.3: Capture release notes / migration note
**Priority**: P2
**Estimate**: 1h

Add a concise note about new Cursor integration and any setup prerequisites.

**Acceptance Criteria**:
- [ ] Release/migration note drafted.
- [ ] Setup prerequisites (install Cursor CLI, config key fields) are explicit.

---

## Dependencies

```text
Task 1.1 ──→ Task 1.2 ──→ Task 2.1 ──→ Task 2.2 ──→ Task 2.3
                          └──────────→ Task 2.4
Task 2.1/2.2/2.3/2.4 ──→ Task 3.1 ──→ Task 3.2 ──→ Task 3.3
```

## Blockers

- [ ] Confirm minimum supported Cursor CLI version.
- [ ] Confirm whether Cursor requires extra environment variables for first release.

## Notes

- Keep implementation scoped to term integration first unless issue discussion requires broader run/executor support.
- Reuse existing command-construction patterns to reduce regressions.
