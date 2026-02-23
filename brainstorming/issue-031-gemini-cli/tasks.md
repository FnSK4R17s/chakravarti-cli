# Add Gemini CLI Integration - Tasks

**Issue**: [#31](https://github.com/FnSK4R17s/chakravarti-cli/issues/31)
**Brainstorm**: [notes.md](./notes.md)
**Created**: 2026-02-23

## Task Overview

| Phase | Tasks | Estimate |
|-------|-------|----------|
| Phase 1: Discovery & Contract Validation | 3 | 1.5h |
| Phase 2: Core Provider Integration | 4 | 2.5h |
| Phase 3: Docker + Config Surface | 3 | 1.5h |
| Phase 4: Verification + Docs | 3 | 1.5h |
| **Total** | **13** | **7.0h** |

---

## Phase 1: Discovery & Contract Validation

### Task 1.1: Validate Gemini CLI non-interactive command contract
**Priority**: P0
**Estimate**: 45m
**Files**: `brainstorming/issue-031-gemini-cli/notes.md` (update findings section)

Confirm exact CLI syntax for prompt execution, model override, and exit-code behavior.

**Acceptance Criteria**:
- [ ] Command shape for prompt execution is documented in notes.
- [ ] Expected success/failure exit behavior is documented.
- [ ] Any model argument support is explicitly confirmed or ruled out.

---

### Task 1.2: Validate auth/config path and mount requirements
**Priority**: P0
**Estimate**: 30m
**Files**: `brainstorming/issue-031-gemini-cli/notes.md` (update implementation notes)

Identify required on-host config/auth files and intended in-container mount targets.

**Acceptance Criteria**:
- [ ] Host config path(s) documented.
- [ ] Container target path(s) documented.
- [ ] Read-only/read-write mount requirement decided.

---

### Task 1.3: Decide image strategy (shared Dockerfile vs dedicated)
**Priority**: P1
**Estimate**: 15m
**Files**: `brainstorming/issue-031-gemini-cli/notes.md`

Choose whether Gemini CLI is installed into existing shared image or separate image.

**Acceptance Criteria**:
- [ ] Strategy decision documented with rationale.
- [ ] Follow-up implementation tasks updated accordingly.

---

## Phase 2: Core Provider Integration

### Task 2.1: Add Gemini variant to sandbox agent enum and factory
**Priority**: P0
**Estimate**: 30m
**Files**: `crates/ckrv-sandbox/src/agent/mod.rs`

Add `Gemini` enum variant, parsing aliases, display name, and provider factory branch.

**Acceptance Criteria**:
- [ ] `AgentType::Gemini` exists.
- [ ] `from_str()` resolves common Gemini aliases.
- [ ] `display_name()` returns a clear Gemini label.
- [ ] `create_agent()` routes Gemini to provider implementation.

---

### Task 2.2: Implement `GeminiProvider`
**Priority**: P0
**Estimate**: 1.0h
**Files**: `crates/ckrv-sandbox/src/agent/gemini.rs`

Implement full `AgentProvider` contract: command construction, env requirements, mounts, output parsing.

**Acceptance Criteria**:
- [ ] `GeminiProvider` compiles and satisfies trait.
- [ ] `build_command()` uses validated CLI contract.
- [ ] `config_mounts()` reflects validated auth/config paths.
- [ ] `parse_output()` normalizes success/failure behavior.

---

### Task 2.3: Wire module exports and visibility
**Priority**: P0
**Estimate**: 15m
**Files**: `crates/ckrv-sandbox/src/agent/mod.rs`

Register `mod gemini;` and `pub use gemini::GeminiProvider;`.

**Acceptance Criteria**:
- [ ] Module compiles with no unresolved imports.
- [ ] Provider is publicly available through agent module.

---

### Task 2.4: Extend CLI-level agent type enum
**Priority**: P0
**Estimate**: 45m
**Files**: `crates/ckrv-cli/src/services/agent_lookup.rs`

Add `Gemini` to serialized config enum used by agents YAML and role selection.

**Acceptance Criteria**:
- [ ] `AgentType` includes `Gemini` with snake_case serialization compatibility.
- [ ] Existing configs deserialize without regression.
- [ ] New `agent_type: gemini` config parses successfully.

---

## Phase 3: Docker + Config Surface

### Task 3.1: Install Gemini CLI in execution image(s)
**Priority**: P0
**Estimate**: 45m
**Files**: `docker/Dockerfile.agent` (and/or `docker/Dockerfile.gemini`)

Add Gemini CLI installation and any prerequisite runtime packages.

**Acceptance Criteria**:
- [ ] Docker build succeeds.
- [ ] Gemini binary is available in container PATH.
- [ ] Non-root execution constraints remain satisfied.

---

### Task 3.2: Ensure runtime mount wiring supports Gemini config
**Priority**: P1
**Estimate**: 30m
**Files**: `crates/ckrv-sandbox/src/agent/gemini.rs`, possibly sandbox mount plumbing

Ensure provider-specific mount declarations are correctly passed to runtime.

**Acceptance Criteria**:
- [ ] Container sees mounted config paths.
- [ ] Mount mode (ro/rw) matches auth requirements.

---

### Task 3.3: Provide example YAML snippet in docs or comments
**Priority**: P2
**Estimate**: 15m
**Files**: `crates/docs/agent-guide.md` (or related docs)

Document minimal user config for `agent_type: gemini`.

**Acceptance Criteria**:
- [ ] Example agent config snippet included.
- [ ] User-facing naming is consistent (Gemini / gemini).

---

## Phase 4: Verification + Docs

### Task 4.1: Add/extend unit tests for Gemini provider behavior
**Priority**: P0
**Estimate**: 45m
**Files**: `crates/ckrv-sandbox/src/agent/tests.rs`

Add tests for command construction, parse behavior, and mount path generation.

**Acceptance Criteria**:
- [ ] Gemini provider tests cover success and failure parse paths.
- [ ] Command shape test validates expected flags/arguments.
- [ ] Mount test validates expected source/target path mapping.

---

### Task 4.2: Update Agent Extensibility docs
**Priority**: P1
**Estimate**: 30m
**Files**: `crates/docs/agent-guide.md`

Add Gemini to supported tools/auth table and architecture references.

**Acceptance Criteria**:
- [ ] Gemini appears in provider list and auth docs.
- [ ] Any architecture diagrams/tables remain accurate.

---

### Task 4.3: Run integration verification and smoke test
**Priority**: P0
**Estimate**: 15m
**Files**: N/A (verification commands + notes update)

Run build/tests and a simple Gemini-backed execution smoke test.

**Acceptance Criteria**:
- [ ] `cargo build --workspace` passes.
- [ ] Relevant test subset passes.
- [ ] Basic Gemini execution path works end-to-end in sandbox.
- [ ] Any residual gaps documented in brainstorm blockers.

---

## Dependencies

```
Task 1.1 ─┬─→ Task 2.1 ─→ Task 2.2 ─→ Task 2.3 ─→ Task 4.1
          ├─→ Task 2.4 ────────────────────────────────┘
Task 1.2 ─┴─→ Task 3.2 ────────────────────────────────┐
Task 1.3 ─────→ Task 3.1 ─→ Task 4.3                   │
Task 3.3 ───────────────→ Task 4.2 ────────────────────┘
```

## Blockers

- [ ] Gemini CLI command contract not yet validated in-repo.
- [ ] Gemini auth/config mount path not yet validated.
- [ ] Potential Docker image size/runtime trade-off decision pending.

## Notes

- This task plan intentionally keeps discovery first to avoid implementing incorrect CLI assumptions.
- The integration should follow existing provider conventions used by Claude, Codex, and Kilo to minimize orchestration-layer churn.
