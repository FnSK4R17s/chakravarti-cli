# Add Mistral Vibe - Tasks

**Issue**: [#29](https://github.com/FnSK4R17s/chakravarti-cli/issues/29)
**Brainstorm**: [notes.md](./notes.md)
**Created**: 2026-02-23

## Task Overview

| Phase | Tasks | Estimate |
|-------|-------|----------|
| Phase 1: Scope & Contract | 3 | 2h |
| Phase 2: Integration & UX | 4 | 4h |
| Phase 3: Validation & Docs | 3 | 2h |
| **Total** | **10** | **8h** |

---

## Phase 1: Scope & Contract

### Task 1.1: Confirm issue acceptance criteria
**Priority**: P0
**Estimate**: 30m

Confirm whether #29 expects a config/docs pathway or a new native provider implementation.

**Acceptance Criteria**:
- [ ] Explicit scope decision captured in issue or internal notes
- [ ] Decision references vision non-goals (orchestration layer first)

---

### Task 1.2: Define “Mistral Vibe” preset contract
**Priority**: P0
**Estimate**: 45m

Define model IDs, required auth/config keys, and expected runtime behavior for the preset path.

**Acceptance Criteria**:
- [ ] Preset fields documented (id, name, agent_type, model)
- [ ] Required env/auth prerequisites listed

---

### Task 1.3: Select implementation path (Option A/B)
**Priority**: P0
**Estimate**: 45m

Choose Kilo-based or OpenRouter-backed profile based on reliability and least complexity.

**Acceptance Criteria**:
- [ ] Selected path documented in notes/spec draft
- [ ] Fallback path identified for unsupported environments

---

## Phase 2: Integration & UX

### Task 2.1: Add config example(s) for Mistral profile
**Priority**: P0
**Estimate**: 1h

Add/update agent config examples with copy-paste-ready Mistral profile.

**Acceptance Criteria**:
- [ ] Example validates against current config schema
- [ ] Includes explanatory comments for model/provider values

---

### Task 2.2: Wire CLI discoverability touchpoints
**Priority**: P1
**Estimate**: 1h

Ensure CLI docs/help and relevant commands clearly indicate Mistral-capable setup route.

**Acceptance Criteria**:
- [ ] At least one CLI-facing doc path includes Mistral setup
- [ ] Terminology is consistent (“Mistral Vibe” vs model/provider naming)

---

### Task 2.3: Wire UI discoverability touchpoints
**Priority**: P1
**Estimate**: 1h

Add/adjust UI agent management guidance so users can configure or select Mistral-capable agents confidently.

**Acceptance Criteria**:
- [ ] UI copy points to supported setup route
- [ ] No ambiguous or conflicting provider instructions

---

### Task 2.4: Guardrails + error messaging
**Priority**: P1
**Estimate**: 1h

Improve validation/errors for common misconfigurations (missing auth, invalid model, unsupported route).

**Acceptance Criteria**:
- [ ] Missing prerequisite errors are actionable
- [ ] Invalid model/provider combo surfaces clear remediation text

---

## Phase 3: Validation & Docs

### Task 3.1: End-to-end smoke test for Mistral-backed run
**Priority**: P0
**Estimate**: 45m

Run a minimal task using the configured Mistral-capable agent path.

**Acceptance Criteria**:
- [ ] Task executes successfully
- [ ] Output and logs show expected agent/provider path

---

### Task 3.2: Update agent guide + getting started snippets
**Priority**: P1
**Estimate**: 45m

Document setup and verification steps in canonical docs.

**Acceptance Criteria**:
- [ ] Agent guide includes Mistral section/preset example
- [ ] Getting started has quick verification command/path

---

### Task 3.3: Promote brainstorm to spec-ready state
**Priority**: P0
**Estimate**: 30m

Finalize open questions that block specification and mark status accordingly.

**Acceptance Criteria**:
- [ ] Remaining unknowns reduced to explicit follow-ups
- [ ] Brainstorm status remains “Ready for Spec” with clear handoff

---

## Dependencies

```
Task 1.1 ──→ Task 1.2 ──→ Task 1.3 ──→ Task 2.1
                                   ├──→ Task 2.2
                                   ├──→ Task 2.3
                                   └──→ Task 2.4
Task 2.1/2.2/2.3/2.4 ──→ Task 3.1 ──→ Task 3.2 ──→ Task 3.3
```

## Blockers

- [x] Unable to execute local `claude` CLI in this runtime (`Permission denied`), so Claude-Code-specific planning run could not be performed directly.
- [ ] Need maintainer confirmation on whether #29 requires native provider implementation vs profile/preset integration.

## Notes

- This plan intentionally favors smallest viable integration consistent with project vision (orchestration over net-new agent stack).
- If issue scope expands to native provider work, create a dedicated follow-up spike/spec before implementation.
