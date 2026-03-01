# Add Github Copilot integration - Tasks

**Issue**: [#37](https://github.com/FnSK4R17s/chakravarti-cli/issues/37)
**Brainstorm**: [notes.md](./notes.md)
**Created**: 2026-02-23

## Task Overview

| Phase | Tasks | Estimate |
|-------|-------|----------|
| Phase 1 | 2 | 12h |
| Phase 2 | 1 | 3h |
| **Total** | 3 | 15h |

---

## Phase 1: Provider foundation

### Task 1.1: Add Copilot provider enum and config schema
**Priority**: P0
**Estimate**: 4h

Wire GitHub Copilot into agent selection, configuration parsing, and validation with clear errors for missing prerequisites.

**Acceptance Criteria**:
- [ ] Provider name is selectable from CLI/UI config paths
- [ ] Config validation rejects malformed Copilot settings with actionable messages

---

### Task 1.2: Add Copilot provider enum and config schema
**Priority**: P0
**Estimate**: 4h

Implement provider runner that launches Copilot non-interactively, captures output, and maps lifecycle events to Chakravarti job status.

**Acceptance Criteria**:
- [ ] Adapter completes successful prompt-response cycle in integration test

---

## Phase 2: Provider foundation

### Task 2.1: Add Copilot provider enum and config schema
**Priority**: P0
**Estimate**: 4h

Implement provider runner that launches Copilot non-interactively, captures output, and maps lifecycle events to Chakravarti job status.

**Acceptance Criteria**:
- [ ] Smoke tests cover happy path plus missing-auth failure mode

---

## Dependencies

```
Task 1.1 ──→ Task 1.2 ──→ Task 2.1
```

## Blockers

<!-- Any external dependencies or blockers -->

- [ ] Confirm stable Copilot CLI command contract in CI containers
- [ ] Validate licensing/auth expectations for non-interactive runs

## Notes

<!-- Additional implementation notes -->
