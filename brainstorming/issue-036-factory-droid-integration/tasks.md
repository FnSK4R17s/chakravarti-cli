# Add Factory Droid integration - Tasks

**Issue**: [#36](https://github.com/FnSK4R17s/chakravarti-cli/issues/36)
**Brainstorm**: [notes.md](./notes.md)
**Created**: 2026-02-24

## Task Overview

| Phase | Tasks | Estimate |
|-------|-------|----------|
| Phase 1: Research | 1 | 1h |
| Phase 2: Implementation | 4 | 4h |
| Phase 3: Integration | 2 | 2h |
| **Total** | 7 | 7h |

---

## Phase 1: Research

### Task 1.1: Research Factory CLI
**Priority**: P0
**Estimate**: 1h
**Files**: N/A (research only)

Research Factory Droid CLI to determine:
- Exact CLI command name (`droid` vs `factory`)
- Authentication method (API key, config file)
- Available CLI flags for non-interactive execution
- Output format for parsing

**Acceptance Criteria**:
- [ ] Document CLI command name
- [ ] Document authentication requirements
- [ ] Document available flags
- [ ] Document output format

---

## Phase 2: Implementation

### Task 2.1: Create Factory Droid provider module
**Priority**: P1
**Estimate**: 1h
**Files**: `crates/ckrv-sandbox/src/agent/factory.rs`

Create `FactoryDroidProvider` struct implementing the `AgentProvider` trait.

**Acceptance Criteria**:
- [ ] Implements `AgentProvider` trait
- [ ] `name()` returns "Factory Droid"
- [ ] `agent_type()` returns `AgentType::FactoryDroid`
- [ ] `build_command()` constructs CLI command
- [ ] `required_env_vars()` returns Factory API key var
- [ ] `config_mounts()` mounts `~/.factory` config directory
- [ ] `parse_output()` parses Factory output format

---

### Task 2.2: Add FactoryDroid to AgentType enum
**Priority**: P1
**Estimate**: 30m
**Files**: `crates/ckrv-sandbox/src/agent/mod.rs`

Add `FactoryDroid` variant to the `AgentType` enum and update all implementations.

**Acceptance Criteria**:
- [ ] `AgentType::FactoryDroid` added
- [ ] `AgentType::from_str()` handles "factory" and "factory-droid"
- [ ] `AgentType::display_name()` returns "Factory Droid"
- [ ] `create_agent()` handles `AgentType::FactoryDroid`

---

### Task 2.3: Add factory module to mod.rs
**Priority**: P1
**Estimate**: 15m
**Files**: `crates/ckrv-sandbox/src/agent/mod.rs`

Register the new factory module.

**Acceptance Criteria**:
- [ ] `mod factory;` added
- [ ] `pub use factory::FactoryDroidProvider;` added
- [ ] `cargo check -p ckrv-sandbox` passes

---

### Task 2.4: Update create_agent function
**Priority**: P1
**Estimate**: 15m
**Files**: `crates/ckrv-sandbox/src/agent/mod.rs`

Add Factory Droid case to `create_agent()` function.

**Acceptance Criteria**:
- [ ] `create_agent(AgentType::FactoryDroid)` returns `Box<FactoryDroidProvider>`

---

## Phase 3: Integration

### Task 3.1: Test basic execution
**Priority**: P2
**Estimate**: 1h
**Files**: `crates/ckrv-sandbox/src/agent/factory.rs`

Test Factory Droid execution in Docker sandbox.

**Acceptance Criteria**:
- [ ] Builds successfully
- [ ] Executes in Docker sandbox (may require API key)

---

### Task 3.2: Update agent-guide.md documentation
**Priority**: P2
**Estimate**: 1h
**Files**: `crates/docs/agent-guide.md`

Document Factory Droid integration.

**Acceptance Criteria**:
- [ ] Agent listed in supported agents section
- [ ] Configuration instructions added
- [ ] Any special requirements documented

---

## Dependencies

```
Phase 1 ─────────────────────────────────────────────────►
  Task 1.1 ──►
               │
Phase 2 ───────┼──────────────────────────────────────────►
               │ Task 2.1 ──► Task 2.2 ──► Task 2.3 ──► Task 2.4
               │
Phase 3 ───────┼──────────────────────────────────────────►
               │
               └───► Task 3.1 ──► Task 3.2
```

## Validation Checklist

- [x] All phases from brainstorm are represented
- [x] Each task has clear acceptance criteria
- [x] Estimates are realistic
- [x] Dependencies form a valid DAG (no cycles)
- [x] P0 tasks are truly blocking
- [x] File paths are accurate
