# Add GitHub Copilot integration - Tasks

**Issue**: [#37](https://github.com/FnSK4R17s/chakravarti-cli/issues/37)
**Brainstorm**: [notes.md](./notes.md)
**Created**: 2026-02-23

## Task Overview

| Phase | Tasks | Estimate |
|-------|-------|----------|
| Phase 1: Discovery + contract | 3 | 3h |
| Phase 2: Provider implementation | 5 | 5h |
| Phase 3: CLI/UI integration | 4 | 4h |
| Phase 4: Validation + docs | 4 | 4h |
| **Total** | **16** | **16h** |

---

## Phase 1: Discovery + Contract

### Task 1.1: Finalize Copilot execution interface
**Priority**: P0  
**Estimate**: 1h

Determine the canonical execution strategy for v1 (CLI path + auth assumptions + non-interactive usage constraints).

**Acceptance Criteria**:
- [ ] Chosen interface documented in notes.md (or follow-up research.md)
- [ ] Required binaries/dependencies explicitly listed
- [ ] Auth requirement documented with expected failure states

---

### Task 1.2: Define credential mount/env strategy
**Priority**: P0  
**Estimate**: 1h

Identify where Copilot credentials live and how they should be mounted/passed into Docker safely.

**Acceptance Criteria**:
- [ ] Host path assumptions documented
- [ ] Container mount mapping documented
- [ ] Security caveats and masking behavior documented

---

### Task 1.3: Add provider contract checklist
**Priority**: P1  
**Estimate**: 1h

Create checklist for required `AgentProvider` methods and expected behavior parity with Claude/Codex/Kilo.

**Acceptance Criteria**:
- [ ] Checklist references `name`, `agent_type`, `command`, `args`, `env_vars`, `config_mounts`
- [ ] Includes negative-path behavior (missing binary/auth)

---

## Phase 2: Provider Implementation

### Task 2.1: Add `AgentType::Copilot`
**Priority**: P0  
**Estimate**: 1h

Extend enum parsing/display/default support for Copilot aliases.

**Acceptance Criteria**:
- [ ] `from_str` recognizes `copilot` variants
- [ ] display name is stable and user-friendly
- [ ] tests added for parse/display behavior

---

### Task 2.2: Implement `CopilotProvider`
**Priority**: P0  
**Estimate**: 1.5h

Create provider module implementing `AgentProvider` contract.

**Acceptance Criteria**:
- [ ] Module added under `ckrv-sandbox/src/agent/`
- [ ] command/args/env vars built deterministically
- [ ] config mounts implemented per Task 1.2 outcomes

---

### Task 2.3: Wire provider factory + defaults
**Priority**: P0  
**Estimate**: 45m

Integrate Copilot into `create_agent(...)` and any provider selection/default code paths.

**Acceptance Criteria**:
- [ ] provider is constructible through existing factory path
- [ ] no regressions in existing provider selection tests

---

### Task 2.4: Add prerequisite detection and errors
**Priority**: P1  
**Estimate**: 45m

Add clear diagnostics for “binary missing” and “auth missing/invalid.”

**Acceptance Criteria**:
- [ ] user-facing error messages include remediation hints
- [ ] errors are surfaced consistently in CLI and UI pathways

---

### Task 2.5: Add provider-focused tests
**Priority**: P0  
**Estimate**: 1h

Expand test coverage for new provider behavior.

**Acceptance Criteria**:
- [ ] unit tests for command/args/env/mounts
- [ ] parse and factory tests include Copilot
- [ ] all agent tests pass

---

## Phase 3: CLI/UI Integration

### Task 3.1: Extend CLI agent selection/config flows
**Priority**: P0  
**Estimate**: 1h

Ensure command surfaces that reference agent types include Copilot.

**Acceptance Criteria**:
- [ ] `--agent` style args accept Copilot
- [ ] help text and validation mention Copilot

---

### Task 3.2: Extend UI agent management surfaces
**Priority**: P1  
**Estimate**: 1.5h

Update UI API/domain types and dropdowns/toggles for Copilot provider.

**Acceptance Criteria**:
- [ ] Copilot appears in agent lists and settings forms
- [ ] save/load config works with Copilot selected

---

### Task 3.3: Add migration/default compatibility handling
**Priority**: P1  
**Estimate**: 45m

Ensure older config files continue to work while new type is added.

**Acceptance Criteria**:
- [ ] backward-compatible config parsing
- [ ] no breaking changes for existing users

---

### Task 3.4: Smoke test `term` and task execution entrypoints
**Priority**: P1  
**Estimate**: 45m

Run end-to-end smoke checks for Copilot selected agent path.

**Acceptance Criteria**:
- [ ] command path resolves correctly when Copilot is chosen
- [ ] expected errors shown when Copilot prerequisites absent

---

## Phase 4: Validation + Docs

### Task 4.1: Regression test existing providers
**Priority**: P0  
**Estimate**: 1h

Verify Claude/Codex/Kilo unaffected by Copilot additions.

**Acceptance Criteria**:
- [ ] existing tests pass
- [ ] no provider command changes for non-Copilot flows

---

### Task 4.2: Update agent guide docs
**Priority**: P1  
**Estimate**: 1h

Document Copilot setup requirements and troubleshooting.

**Acceptance Criteria**:
- [ ] `crates/docs/agent-guide.md` includes Copilot section
- [ ] setup and common failures documented

---

### Task 4.3: Add release note fragment
**Priority**: P2  
**Estimate**: 30m

Document feature availability and any beta/experimental caveat.

**Acceptance Criteria**:
- [ ] release notes mention Copilot support and scope

---

### Task 4.4: Final readiness review
**Priority**: P0  
**Estimate**: 1.5h

Review issue acceptance against success criteria and mark brainstorm status.

**Acceptance Criteria**:
- [ ] all P0 tasks complete
- [ ] unresolved items moved to explicit follow-up list
- [ ] notes status moved toward “Ready for Spec”

---

## Dependencies

```text
Task 1.1 ─┬─> Task 2.2 ─> Task 2.3 ─> Task 3.1
Task 1.2 ─┘         │              └─> Task 3.2
Task 1.3 ───────────┘
Task 2.4 ───────────────> Task 3.4
Task 2.5 ───────────────> Task 4.1
Task 3.1 + 3.2 + 4.1 ──> Task 4.4
Task 4.2 + 4.3 ────────> Task 4.4
```

## Blockers

- [ ] Canonical Copilot non-interactive interface must be validated before coding starts.
- [ ] Credential mount path may vary by OS; needs tested assumptions.

## Notes

- This task plan intentionally keeps issue #37 scoped to a first-class provider integration, not a full provider plugin system redesign.
- If runtime constraints make direct Copilot execution unreliable, fallback plan is to ship behind experimental flag and document limitations explicitly.
