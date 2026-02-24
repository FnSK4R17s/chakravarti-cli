# Add AMP CLI integration - Tasks

**Issue**: [#33](https://github.com/FnSK4R17s/chakravarti-cli/issues/33)
**Brainstorm**: [notes.md](./notes.md)
**Created**: 2026-02-24

## Task Overview

| Phase | Tasks | Estimate |
|-------|-------|----------|
| Phase 1: Research | 1 | 1h |
| Phase 2: Add Amp to AgentType enum | 3 | 1h |
| Phase 3: Create AmpProvider implementation | 5 | 4h |
| Phase 4: Integration & Testing | 2 | 2h |
| **Total** | 11 | 8h |

---

## Phase 1: Research

### Task 1.1: Research Amp CLI capabilities
**Priority**: P1
**Estimate**: 1h
**Files**: None

Research Amp CLI flags, configuration location, environment variables, and output format for programmatic parsing.

**Acceptance Criteria**:
- [ ] Document Amp CLI installation command
- [ ] Document `amp --help` output (flags for non-interactive execution)
- [ ] Identify config file location (~/.config/amp/ or similar)
- [ ] Identify required environment variables (API key?)
- [ ] Identify output format for parsing

---

## Phase 2: Add Amp to AgentType enum

### Task 2.1: Add Amp variant to AgentType enum
**Priority**: P0
**Estimate**: 15m
**Files**: `crates/ckrv-sandbox/src/agent/mod.rs`

Add `Amp` variant to the `AgentType` enum.

**Acceptance Criteria**:
- [ ] `Amp` variant added to enum
- [ ] `cargo check -p ckrv-sandbox` passes

---

### Task 2.2: Add Amp string parsing
**Priority**: P0
**Estimate**: 15m
**Files**: `crates/ckrv-sandbox/src/agent/mod.rs`

Add string parsing for "amp" in `AgentType::from_str()`.

**Acceptance Criteria**:
- [ ] `AgentType::from_str("amp")` returns `Some(AgentType::Amp)`
- [ ] Tests pass

---

### Task 2.3: Add Amp display name
**Priority**: P0
**Estimate**: 15m
**Files**: `crates/ckrv-sandbox/src/agent/mod.rs`

Add display name entry for Amp in `display_name()`.

**Acceptance Criteria**:
- [ ] `AgentType::Amp.display_name()` returns "Amp"
- [ ] Tests pass

---

## Phase 3: Create AmpProvider implementation

### Task 3.1: Create amp.rs module
**Priority**: P0
**Estimate**: 30m
**Files**: `crates/ckrv-sandbox/src/agent/amp.rs`

Create the new `amp.rs` module file with basic struct definition.

**Acceptance Criteria**:
- [ ] File created at `crates/ckrv-sandbox/src/agent/amp.rs`
- [ ] Module declared in `mod.rs`
- [ ] `AmpProvider` struct defined
- [ ] `cargo check -p ckrv-sandbox` passes

---

### Task 3.2: Implement AgentProvider trait - name and agent_type
**Priority**: P0
**Estimate**: 30m
**Files**: `crates/ckrv-sandbox/src/agent/amp.rs`

Implement `name()` and `agent_type()` methods.

**Acceptance Criteria**:
- [ ] `name()` returns "amp"
- [ ] `agent_type()` returns `AgentType::Amp`

---

### Task 3.3: Implement build_command
**Priority**: P0
**Estimate**: 1h
**Files**: `crates/ckrv-sandbox/src/agent/amp.rs`

Implement `build_command()` to construct the Amp CLI command with prompt and workdir.

**Acceptance Criteria**:
- [ ] Returns correct command vector for Amp execution
- [ ] Handles workdir correctly
- [ ] Respects model override from config

---

### Task 3.4: Implement required_env_vars and config_mounts
**Priority**: P1
**Estimate**: 1h
**Files**: `crates/ckrv-sandbox/src/agent/amp.rs`

Implement environment variables and Docker mounts for Amp config.

**Acceptance Criteria**:
- [ ] `required_env_vars()` returns needed env vars
- [ ] `config_mounts()` returns correct mount paths for Amp config

---

### Task 3.5: Implement parse_output
**Priority**: P1
**Estimate**: 1h
**Files**: `crates/ckrv-sandbox/src/agent/amp.rs`

Implement output parsing to normalize Amp CLI output.

**Acceptance Criteria**:
- [ ] Parses stdout/stderr correctly
- [ ] Returns normalized `AgentOutput`
- [ ] Handles exit codes

---

## Phase 4: Integration & Testing

### Task 4.1: Add AmpProvider to create_agent
**Priority**: P0
**Estimate**: 15m
**Files**: `crates/ckrv-sandbox/src/agent/mod.rs`

Add `AmpProvider` to the match statement in `create_agent()`.

**Acceptance Criteria**:
- [ ] `create_agent(AgentType::Amp)` returns `Box<new(AmpProvider::new())>`
- [ ] `cargo check -p ckrv-sandbox` passes

---

### Task 4.2: Add tests and verify
**Priority**: P1
**Estimate**: 2h
**Files**: `crates/ckrv-sandbox/src/agent/tests.rs`

Add tests for AmpProvider and verify AgentType parsing.

**Acceptance Criteria**:
- [ ] Tests for `AgentType::from_str("amp")`
- [ ] Tests for `AmpProvider` methods
- [ ] `cargo test -p ckrv-sandbox` passes

---

## Dependencies

```
Phase 1 ──────────────────────────────►
  Task 1.1 ──► Task 2.1 ──► Task 2.2 ──► Task 2.3
                          │
                          ▼
                   Phase 3 ──────────────────────────────►
                     Task 3.1 ──► Task 3.2 ──► Task 3.3
                                   │         │
                                   │         ▼
                                   │   Task 3.4 ──► Task 3.5
                                   │
Phase 4 ────────────────────────────┼─────────────────────►
                                     │
                                     └───► Task 4.1 ──► Task 4.2
```

---

## Notes

- Amp CLI installation: `curl -fsSL https://ampcode.com/install.sh | bash`
- Amp uses `amp` command (not `ampcode`)
- Reference existing implementations: `crates/ckrv-sandbox/src/agent/{claude,codex,kilo}.rs`
