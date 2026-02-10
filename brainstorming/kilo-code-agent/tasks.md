# Add Kilo Code Agent Integration - Tasks

**Issue**: None yet (create when ready to implement)
**Brainstorm**: [notes.md](./notes.md)
**Created**: 2026-02-10
**Status**: ✅ All tasks complete

## Task Overview

| Phase | Tasks | Estimate | Status |
|-------|-------|----------|--------|
| Phase 1: Core Provider | 3 | 2h | ✅ Done |
| Phase 2: Docker & Mounts | 3 | 2h | ✅ Done |
| Phase 3: CLI Integration & Docs | 2 | 2h | ✅ Done |
| **Total** | **8** | **~6h** | **✅** |

---

## Dependencies

```
Phase 1 ──────────────────────────────────────────────►
  T001 ──► T002 ──► T003
                       │
Phase 2 ───────────────┼─────────────────────────────►
                       │
                       ├─► T004 (P)
                       ├─► T005 (P)
                       └─► T006
                              │
Phase 3 ──────────────────────┼──────────────────────►
                              │
                              ├─► T007
                              └─► T008
```

---

## Phase 1: Core Provider

### Task 1.1: Create KiloCodeProvider implementation
- [x] T001 Create `crates/ckrv-sandbox/src/agent/kilo.rs`

**Priority**: P0
**Estimate**: 1h
**Files**: `crates/ckrv-sandbox/src/agent/kilo.rs`

**Acceptance Criteria**:
- [x] `KiloCodeProvider` struct exists with `#[derive(Debug, Default)]`
- [x] `AgentProvider` trait fully implemented
- [x] `build_command()` generates correct `kilo run` invocation
- [x] `config_mounts()` mounts `~/.config/kilo/` directory
- [x] `required_env_vars()` returns empty vec

---

### Task 1.2: Register KiloCode in agent module
- [x] T002 Add `KiloCode` variant to `AgentType` enum and update factory in `crates/ckrv-sandbox/src/agent/mod.rs`

**Priority**: P0
**Estimate**: 30m
**Files**: `crates/ckrv-sandbox/src/agent/mod.rs`

**Acceptance Criteria**:
- [x] `AgentType::KiloCode` variant exists
- [x] `AgentType::from_str("kilo")` returns `Some(AgentType::KiloCode)`
- [x] `AgentType::from_str("kilo-code")` returns `Some(AgentType::KiloCode)`
- [x] `AgentType::from_str("kilocode")` returns `Some(AgentType::KiloCode)`
- [x] `create_agent(AgentType::KiloCode)` returns `KiloCodeProvider`
- [x] `cargo check -p ckrv-sandbox` passes

---

### Task 1.3: Add tests for Kilo provider
- [x] T003 Add Kilo Code test cases in `crates/ckrv-sandbox/src/agent/tests.rs`

**Priority**: P1
**Estimate**: 30m
**Files**: `crates/ckrv-sandbox/src/agent/tests.rs`

**Acceptance Criteria**:
- [x] All new test functions added (7 Kilo-specific tests)
- [x] `cargo test -p ckrv-sandbox` passes with all 20 tests green
- [x] Tests cover: string parsing, command building, model override, streaming flag, output parsing

---

## Phase 2: Docker & Mounts

### Task 2.1: Create Dockerfile.kilo
- [x] T004 [P] Create dedicated Kilo Code Dockerfile at `docker/Dockerfile.kilo`

**Priority**: P1
**Estimate**: 30m
**Files**: `docker/Dockerfile.kilo`

**Acceptance Criteria**:
- [x] `Dockerfile.kilo` exists in `docker/` directory
- [x] Follows same structure as `Dockerfile.codex`
- [x] Installs `@kilocode/cli` globally
- [x] Creates `/home/kilo/.config/kilo/` with open permissions

---

### Task 2.2: Add Kilo CLI to combined Dockerfile.agent
- [x] T005 [P] Add Kilo Code CLI installation to `docker/Dockerfile.agent`

**Priority**: P1
**Estimate**: 30m
**Files**: `docker/Dockerfile.agent`

**Acceptance Criteria**:
- [x] Kilo CLI installed alongside Claude and Codex in combined image
- [x] Kilo config directory created with permissions
- [x] Version verification added

---

### Task 2.3: Verify docker mount integration
- [x] T006 Verify docker mount integration in `crates/ckrv-sandbox/src/docker.rs`

**Priority**: P1
**Estimate**: 1h
**Files**: `crates/ckrv-sandbox/src/docker.rs`

**Result**: No changes needed. `config_mounts()` is a trait method — the `KiloCodeProvider` implementation handles this via the trait interface.

---

## Phase 3: CLI Integration & Docs

### Task 3.1: Add KiloCode to CLI agent_lookup
- [x] T007 Add `KiloCode` variant to CLI `AgentType` enum in `crates/ckrv-cli/src/services/agent_lookup.rs`

**Priority**: P1
**Estimate**: 30m
**Files**: `crates/ckrv-cli/src/services/agent_lookup.rs`, `crates/ckrv-cli/src/commands/term.rs`

**Acceptance Criteria**:
- [x] `AgentType::KiloCode` variant in CLI enum
- [x] YAML config with `agent_type: kilo_code` deserializes correctly
- [x] All match statements in `term.rs` updated (4 locations)
- [x] `cargo check --workspace` passes

---

### Task 3.2: Update agent-guide documentation
- [x] T008 Document Kilo Code integration in `crates/docs/agent-guide.md`

**Priority**: P2
**Estimate**: 30m
**Files**: `crates/docs/agent-guide.md`

**Acceptance Criteria**:
- [x] Kilo Code listed in supported agents table
- [x] Authentication method documented
- [x] Architecture diagram updated
- [x] Configuration examples provided
- [x] Streaming output documented
- [x] Links to Kilo Code GitHub and docs

---

## Validation Checklist

- [x] `cargo check --workspace` succeeds
- [x] `cargo test -p ckrv-sandbox` — all 20 agent tests pass
- [x] Dockerfiles created/updated for Kilo Code
- [x] Documentation updated in agent-guide.md
