# Add Opencode integration - Tasks

**Issue**: [#35](https://github.com/FnSK4R17s/chakravarti-cli/issues/35)
**Brainstorm**: [notes.md](./notes.md)
**Created**: 2026-02-23

## Task Overview

| Phase | Tasks | Estimate |
|-------|-------|----------|
| Phase 1: Discovery & parity mapping | 3 | 4h |
| Phase 2: CLI/runtime integration | 4 | 8h |
| Phase 3: Docs, validation, and polish | 3 | 4h |
| **Total** | **10** | **16h** |

---

## Phase 1: Discovery & parity mapping

### Task 1.1: Enumerate all agent-selection surfaces
**Priority**: P0
**Estimate**: 1h

Audit CLI commands, config handling, skills references, and runtime code paths where supported agents are listed or validated.

**Acceptance Criteria**:
- [ ] List of all selection points is documented in implementation notes/PR description
- [ ] Any hardcoded allowlists are identified

---

### Task 1.2: Define Opencode parity contract
**Priority**: P0
**Estimate**: 2h

Define expected behavior when `opencode` is selected (invocation path, sandbox/image requirements, logs, and failure semantics).

**Acceptance Criteria**:
- [ ] Parity contract is written and reviewable
- [ ] Edge-case behavior (missing runtime/images/config) is explicitly defined

---

### Task 1.3: Add/adjust integration test plan
**Priority**: P1
**Estimate**: 1h

Identify existing tests to extend and new tests needed to validate Opencode selection and execution.

**Acceptance Criteria**:
- [ ] Test cases cover positive and negative selection paths
- [ ] Test plan includes at least one end-to-end workflow assertion

---

## Phase 2: CLI/runtime integration

### Task 2.1: Update agent option parsing and validation
**Priority**: P0
**Estimate**: 2h

Add `opencode` to supported agent enums/validators and ensure command help reflects it.

**Acceptance Criteria**:
- [ ] CLI accepts `opencode` where agent selection is supported
- [ ] Invalid-agent errors remain clear and unchanged for unsupported values

---

### Task 2.2: Wire Opencode execution path
**Priority**: P0
**Estimate**: 3h

Ensure runtime dispatch selects the Opencode backend/container/tooling consistently.

**Acceptance Criteria**:
- [ ] Selecting `opencode` routes to the correct execution path
- [ ] Runtime failures produce actionable errors

---

### Task 2.3: Add/refresh container and environment assumptions
**Priority**: P1
**Estimate**: 2h

Verify Docker image/build hooks and required environment checks for Opencode execution.

**Acceptance Criteria**:
- [ ] Build/dependency docs and scripts align with Opencode runtime needs
- [ ] Missing prerequisite messages are explicit

---

### Task 2.4: Implement integration tests
**Priority**: P1
**Estimate**: 1h

Implement tests from Phase 1 test plan to protect new behavior.

**Acceptance Criteria**:
- [ ] Tests fail before and pass after integration changes
- [ ] CI-local test command for affected scope is documented

---

## Phase 3: Docs, validation, and polish

### Task 3.1: Update skills/docs references
**Priority**: P1
**Estimate**: 1.5h

Update `.agents` skills and user-facing docs to include Opencode setup/usage examples.

**Acceptance Criteria**:
- [ ] All relevant docs mention Opencode consistently
- [ ] At least one concrete command example is provided

---

### Task 3.2: Validate issue-to-workflow user journey
**Priority**: P1
**Estimate**: 1.5h

Run through issue planning/execution flow using Opencode selection and verify artifacts/logs are sensible.

**Acceptance Criteria**:
- [ ] End-to-end dry run or execution is documented
- [ ] Any deviations are captured as follow-up tasks/issues

---

### Task 3.3: Prepare PR notes and rollout guidance
**Priority**: P2
**Estimate**: 1h

Document what changed, migration impact, and contributor instructions.

**Acceptance Criteria**:
- [ ] PR summary includes behavior changes and test evidence
- [ ] Rollout notes include fallback/mitigation guidance

---

## Dependencies

```text
Task 1.1 ──→ Task 1.2 ──→ Task 2.1 ──→ Task 2.2 ──→ Task 2.4
      └────→ Task 1.3 ────────────────────────────────┘
Task 2.3 ───────────────────────────────────────────→ Task 3.2
Task 3.1 ──→ Task 3.3
```

## Blockers

- [ ] Confirm exact Opencode runtime/backend constraints in target environments
- [ ] Ensure CI runners have required dependencies/images for Opencode validation

## Notes

- Keep naming canonical as `opencode` across code, docs, and examples.
- Avoid introducing agent-specific divergence unless technically necessary.
- Prefer extending current abstractions to preserve maintainability.
