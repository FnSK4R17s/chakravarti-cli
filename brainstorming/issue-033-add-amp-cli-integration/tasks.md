# Add AMP CLI integration - Tasks

**Issue**: [#33](https://github.com/FnSK4R17s/chakravarti-cli/issues/33)
**Brainstorm**: [notes.md](./notes.md)
**Created**: 2026-02-23

## Task Overview

| Phase | Tasks | Estimate |
|-------|-------|----------|
| Phase 1: Discovery & Contract Lock | 3 | 2.5h |
| Phase 2: Sandbox Provider Integration | 4 | 6h |
| Phase 3: Docker & Runtime Integration | 3 | 4h |
| Phase 4: CLI/UI + Docs + Validation | 4 | 4.5h |
| **Total** | 14 | 17h |

---

## Phase 1: Discovery & Contract Lock

### Task 1.1: Confirm AMP CLI invocation contract
**Priority**: P0
**Estimate**: 1h
**Files**: `brainstorming/issue-033-add-amp-cli-integration/notes.md`

Validate AMP CLI’s supported automation flags, one-shot prompt mode, exit-code semantics, and structured output options.

**Acceptance Criteria**:
- [ ] One canonical command shape documented (interactive + non-interactive).
- [ ] Exit-code behavior documented for success/failure.
- [ ] JSON/structured-output capability confirmed (or explicitly unavailable).

---

### Task 1.2: Confirm AMP auth + config mount requirements
**Priority**: P0
**Estimate**: 1h
**Files**: `brainstorming/issue-033-add-amp-cli-integration/notes.md`, `crates/docs/agent-guide.md`

Identify AMP’s config directory and env var requirements for containerized execution.

**Acceptance Criteria**:
- [ ] Host auth/config paths identified.
- [ ] Required env vars listed.
- [ ] Container mount plan defined with read-only strategy.

---

### Task 1.3: Freeze provider design contract
**Priority**: P0
**Estimate**: 30m
**Files**: `brainstorming/issue-033-add-amp-cli-integration/notes.md`, `brainstorming/issue-033-add-amp-cli-integration/tasks.md`

Finalize contract assumptions for `AmpProvider` before implementation.

**Acceptance Criteria**:
- [ ] `build_command`, `required_env_vars`, `config_mounts`, `parse_output` expectations documented.
- [ ] Remaining unknowns explicitly tracked as blockers.

---

## Phase 2: Sandbox Provider Integration

### Task 2.1: Add AMP to sandbox `AgentType` and provider factory
**Priority**: P0
**Estimate**: 1h
**Files**: `crates/ckrv-sandbox/src/agent/mod.rs`

Add enum variant, string parsing aliases, and `create_agent()` registration.

**Acceptance Criteria**:
- [ ] `AgentType::Amp` variant exists.
- [ ] String aliases parse correctly (`amp`, `ampcode` if desired).
- [ ] Factory returns `AmpProvider`.

---

### Task 2.2: Implement `AmpProvider`
**Priority**: P0
**Estimate**: 2.5h
**Files**: `crates/ckrv-sandbox/src/agent/amp.rs`

Implement trait methods with AMP command builder, env requirements, auth mounts, and output parsing.

**Acceptance Criteria**:
- [ ] Command generation works for sandbox task execution.
- [ ] Required env vars are enforced or validated.
- [ ] Mount strategy mirrors least-privilege pattern.
- [ ] Output parsing returns normalized success/failure.

---

### Task 2.3: Add/extend agent unit tests
**Priority**: P1
**Estimate**: 1.5h
**Files**: `crates/ckrv-sandbox/src/agent/tests.rs` (or module-specific tests)

Add tests for command generation, parser behavior, and mount config.

**Acceptance Criteria**:
- [ ] `AmpProvider` command test covers key flags/args.
- [ ] Parse tests cover success and failure examples.
- [ ] Mount tests assert expected host/container path wiring.

---

### Task 2.4: Wire AMP through runner/config mapping
**Priority**: P0
**Estimate**: 1h
**Files**: `crates/ckrv-core/src/runner.rs`, `crates/ckrv-cli/*` (agent mapping locations)

Ensure AMP agent type can be selected from configured agents and routed into sandbox execution.

**Acceptance Criteria**:
- [ ] AMP appears in resolved runner agent selection.
- [ ] Unsupported mapping paths fail with actionable error.
- [ ] `cargo check --workspace` passes.

---

## Phase 3: Docker & Runtime Integration

### Task 3.1: Add `docker/Dockerfile.amp`
**Priority**: P0
**Estimate**: 1.5h
**Files**: `docker/Dockerfile.amp`

Create dedicated AMP image with CLI installed, non-root runtime user, and correct HOME/workspace ownership.

**Acceptance Criteria**:
- [ ] Image builds locally.
- [ ] `amp --version` (or equivalent) succeeds during build verification.
- [ ] Container runs as non-root user.

---

### Task 3.2: Update build scripts/matrix for AMP image
**Priority**: P1
**Estimate**: 1h
**Files**: `justfile`, CI workflows under `.github/workflows/` (if present)

Include AMP image in local/CI build paths.

**Acceptance Criteria**:
- [ ] AMP image participates in build task.
- [ ] CI references AMP image where agent images are enumerated.

---

### Task 3.3: Optional inclusion in combined agent image
**Priority**: P2
**Estimate**: 1.5h
**Files**: `docker/Dockerfile.agent`

Decide whether AMP should be bundled in the combined image now or deferred.

**Acceptance Criteria**:
- [ ] Decision documented (include now vs defer).
- [ ] If included, build verification passes.
- [ ] If deferred, explicit follow-up task tracked.

---

## Phase 4: CLI/UI + Docs + Validation

### Task 4.1: Expose AMP in CLI/UI agent catalogs
**Priority**: P1
**Estimate**: 1.5h
**Files**: `crates/ckrv-cli/*`, `crates/ckrv-ui/*` (agent list/config schemas)

Add AMP to user-facing agent type options and validation.

**Acceptance Criteria**:
- [ ] AMP can be selected in config/UI.
- [ ] Invalid AMP config shows actionable validation errors.

---

### Task 4.2: Update documentation
**Priority**: P1
**Estimate**: 1h
**Files**: `crates/docs/agent-guide.md`, `README.md` (if agent matrix present)

Document prerequisites, auth setup, and usage examples for AMP.

**Acceptance Criteria**:
- [ ] Agent guide contains AMP section with setup + sample YAML.
- [ ] Any provider matrix/docs references are updated.

---

### Task 4.3: End-to-end smoke tests
**Priority**: P1
**Estimate**: 1.5h
**Files**: test artifacts / docs notes

Run representative flows for task execution and terminal session.

**Acceptance Criteria**:
- [ ] `ckrv task run --agent <amp-agent> -p "..."` smoke test passes.
- [ ] `ckrv term --agent <amp-agent>` launches successfully.
- [ ] No regressions for Claude/Codex/Kilo smoke checks.

---

### Task 4.4: Finalize brainstorm status and handoff to spec
**Priority**: P1
**Estimate**: 30m
**Files**: `brainstorming/issue-033-add-amp-cli-integration/notes.md`

Close open questions, set status to Ready for Spec, and link follow-up spec/task workflows.

**Acceptance Criteria**:
- [ ] Notes status updated to **Ready for Spec**.
- [ ] Open questions resolved or clearly scoped as known follow-ups.

---

## Dependencies

```text
Task 1.1 ──→ Task 1.2 ──→ Task 1.3
   │                         │
   └───────────────→ Task 2.1 ──→ Task 2.2 ──→ Task 2.3
                                 │
                                 └──→ Task 2.4 ──→ Task 3.1 ──→ Task 3.2 ──→ Task 4.1
                                                      │                      │
                                                      └──→ Task 3.3          ├──→ Task 4.2
                                                                             ├──→ Task 4.3
                                                                             └──→ Task 4.4
```

## Blockers

- [ ] AMP CLI official docs needed for exact non-interactive flags and auth mount contract.
- [ ] Confirm whether AMP has structured output mode suitable for parser/telemetry parity.

## Notes

- Keep implementation aligned with existing provider patterns in `claude.rs`, `codex.rs`, and `kilo.rs`.
- Prefer explicit compatibility and graceful failure over heuristic command parsing.
