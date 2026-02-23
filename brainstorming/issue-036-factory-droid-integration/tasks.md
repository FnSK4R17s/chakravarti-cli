# Add Factory Droid integration - Tasks

**Issue**: [#36](https://github.com/FnSK4R17s/chakravarti-cli/issues/36)
**Brainstorm**: [notes.md](./notes.md)
**Created**: 2026-02-23

## Task Overview

| Phase | Tasks | Estimate |
|-------|-------|----------|
| Phase 1: Discovery & contract | 3 | 2-4h |
| Phase 2: Provider implementation | 5 | 4-8h |
| Phase 3: UX/docs hardening | 4 | 2-4h |
| **Total** | **12** | **8-16h** |

---

## Phase 1: Discovery & Contract

### Task 1.1: Validate Factory CLI runtime contract
**Priority**: P0  
**Estimate**: 1-2h

Identify exact binary name, required flags, non-interactive mode behavior, and exit-code semantics.

**Acceptance Criteria**:
- [ ] Command examples for prompt/task execution are documented
- [ ] Exit code + stdout/stderr behavior is captured
- [ ] Required runtime dependencies are listed

### Task 1.2: Define auth and environment handling
**Priority**: P0  
**Estimate**: 0.5-1h

Document how auth is passed in local and containerized execution environments.

**Acceptance Criteria**:
- [ ] Required env vars or credential locations are known
- [ ] Missing-auth failure mode is defined
- [ ] Redaction policy for logs is documented

### Task 1.3: Confirm provider capability matrix
**Priority**: P1  
**Estimate**: 0.5-1h

Map Factory features to `ckrv` provider capabilities (streaming, tool calls, plan/task granularity).

**Acceptance Criteria**:
- [ ] Capability matrix section added to implementation notes/spec
- [ ] Unsupported features are explicitly gated with clear errors

---

## Phase 2: Provider Implementation

### Task 2.1: Add Factory provider enum/config wiring
**Priority**: P0  
**Estimate**: 1h

Add provider type and config parsing/validation where agent providers are defined.

**Acceptance Criteria**:
- [ ] Config accepts Factory provider entries
- [ ] Validation errors are descriptive for malformed Factory config

### Task 2.2: Implement Factory executor adapter
**Priority**: P0  
**Estimate**: 2-3h

Implement command builder + execution adapter that transforms ckrv task payloads into Factory invocation.

**Acceptance Criteria**:
- [ ] Adapter executes with expected working directory/context
- [ ] Prompt/task payload reaches Factory intact
- [ ] Stdout/stderr capture integrates with existing run logs

### Task 2.3: Add robust error mapping
**Priority**: P0  
**Estimate**: 1h

Translate common Factory failure modes into actionable `ckrv` errors.

**Acceptance Criteria**:
- [ ] Missing binary, missing auth, invalid flags are distinguishable
- [ ] Error messages include remediation hints

### Task 2.4: Integrate provider selection in orchestration flow
**Priority**: P1  
**Estimate**: 0.5-1h

Ensure planner/executor can schedule tasks to Factory where configured.

**Acceptance Criteria**:
- [ ] Factory can be selected as default or per-task provider
- [ ] Mixed-provider execution remains functional

### Task 2.5: Add tests for provider config and command construction
**Priority**: P1  
**Estimate**: 1-2h

Add unit/integration tests for config parsing and command-generation invariants.

**Acceptance Criteria**:
- [ ] Tests cover happy path and common failure paths
- [ ] Existing tests continue to pass

---

## Phase 3: UX/Docs Hardening

### Task 3.1: Update agent/provider docs
**Priority**: P1  
**Estimate**: 0.5-1h

Add Factory setup, auth requirements, and quick-start examples.

**Acceptance Criteria**:
- [ ] Docs include install/config/run flow
- [ ] Troubleshooting section includes top 3 setup failures

### Task 3.2: Expose Factory in UI/API provider listings (if applicable)
**Priority**: P1  
**Estimate**: 0.5-1h

Ensure any provider catalog shown in UI/API includes Factory consistently.

**Acceptance Criteria**:
- [ ] Factory appears where users select agents/providers
- [ ] Labels/descriptions are clear and consistent

### Task 3.3: Smoke-test in isolated runtime
**Priority**: P1  
**Estimate**: 0.5-1h

Run a representative sample spec/task through Factory in a sandbox-like environment.

**Acceptance Criteria**:
- [ ] End-to-end run succeeds in at least one realistic environment
- [ ] Logs confirm expected provider path

### Task 3.4: Final readiness review
**Priority**: P2  
**Estimate**: 0.5h

Review issue scope, known limitations, and whether to transition to a formal spec.

**Acceptance Criteria**:
- [ ] Remaining open questions are tracked
- [ ] Decision documented: "Ready for Spec" or "Implement directly"

---

## Dependencies

```text
1.1 + 1.2 ──→ 2.1 ──→ 2.2 ──→ 2.3 ──→ 3.3
      └──────→ 1.3 ──→ 2.4 ──→ 3.2
2.1 + 2.2 ──→ 2.5
2.x + 3.x ──→ 3.4
```

## Blockers

- [x] External docs/API discovery is limited from the issue body alone
- [x] Brave web search unavailable in this runtime (`missing_brave_api_key`), reducing external discovery speed
- [ ] Need concrete Factory CLI/auth documentation to finalize implementation contract

## Notes

If CLI contract details are confirmed quickly, this can proceed directly to implementation tasks. If not, create a formal spec with explicit assumptions and a risk-managed fallback path.
