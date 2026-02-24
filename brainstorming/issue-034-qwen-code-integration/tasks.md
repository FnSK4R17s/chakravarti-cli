# Add Qwen Code Integration - Tasks

**Issue**: [#34](https://github.com/FnSK4R17s/chakravarti-cli/issues/34)
**Brainstorm**: [notes.md](./notes.md)
**Created**: 2026-02-24

## Task Overview

| Phase | Tasks | Estimate |
|-------|-------|----------|
| Phase 1: Core Implementation | 3 | 3h |
| Phase 2: Integration | 2 | 1.5h |
| Phase 3: Testing | 2 | 1.5h |
| **Total** | 7 | 6h |

---

## Phase 1: Core Implementation

### Task 1.1: Add Qwen variant to AgentType enum
**Priority**: P0
**Estimate**: 30m
**Files**: `crates/ckrv-sandbox/src/agent/mod.rs`

Add `Qwen` variant to the `AgentType` enum, update `from_str()` parser to accept "qwen" and "qwen-code", and update `display_name()` to return "Qwen Code".

**Acceptance Criteria**:
- [ ] `AgentType::from_str("qwen")` returns `Some(AgentType::Qwen)`
- [ ] `AgentType::from_str("qwen-code")` returns `Some(AgentType::Qwen)`
- [ ] `AgentType::Qwen.display_name()` returns "Qwen Code"

---

### Task 1.2: Create QwenProvider implementation
**Priority**: P0
**Estimate**: 2h
**Files**: `crates/ckrv-sandbox/src/agent/qwen.rs` (new file)

Create `qwen.rs` implementing the `AgentProvider` trait:
- `name()`: "qwen-code"
- `agent_type()`: `AgentType::Qwen`
- `build_command()`: Build qwen command with `--yes --approval-mode=auto` for headless mode, support both CLI and API modes via config
- `required_env_vars()`: Return `OPENAI_API_KEY`, `QWEN_AUTH_TOKEN`, `OPENAI_BASE_URL`
- `config_mounts()`: Mount `~/.qwen/` directory
- `parse_output()`: Parse standard output format

Support both:
- CLI mode: `qwen --yes --approval-mode=auto <prompt>`
- API mode: Use OpenAI-compatible endpoint with `qwen/qwen3-coder` model

**Acceptance Criteria**:
- [ ] `QwenProvider` implements `AgentProvider` trait
- [ ] CLI mode generates valid qwen command
- [ ] API mode generates valid OpenAI-compatible curl command
- [ ] `cargo check -p ckrv-sandbox` passes

---

### Task 1.3: Update create_agent function
**Priority**: P0
**Estimate**: 15m
**Files**: `crates/ckrv-sandbox/src/agent/mod.rs`

Add `AgentType::Qwen` case to `create_agent()` function.

**Acceptance Criteria**:
- [ ] `create_agent(AgentType::Qwen)` returns `Box::new(QwenProvider::new())`
- [ ] Code compiles without errors

---

## Phase 2: Integration

### Task 2.1: Add module declaration
**Priority**: P1
**Estimate**: 15m
**Files**: `crates/ckrv-sandbox/src/agent/mod.rs`

Add `mod qwen;` and `pub use qwen::QwenProvider;` to the module.

**Acceptance Criteria**:
- [ ] Module is publicly exported
- [ ] `cargo check -p ckrv-sandbox` passes

---

### Task 2.2: Update AgentConfig to support Qwen-specific options
**Priority**: P1
**Estimate**: 1h
**Files**: `crates/ckrv-sandbox/src/agent/mod.rs`

Add Qwen-specific configuration to `AgentConfig`:
- `use_api`: Boolean to switch between CLI and API mode
- `api_base_url`: Custom API endpoint (default: OpenAI-compatible)

**Acceptance Criteria**:
- [ ] `AgentConfig` has Qwen-specific options
- [ ] Options are passed through to `QwenProvider`

---

## Phase 3: Testing

### Task 3.1: Add unit tests for AgentType
**Priority**: P2
**Estimate**: 30m
**Files**: `crates/ckrv-sandbox/src/agent/tests.rs` or inline tests

Add tests for:
- `AgentType::from_str("qwen")`
- `AgentType::from_str("qwen-code")`
- `AgentType::Qwen.display_name()`

**Acceptance Criteria**:
- [ ] All new test cases pass
- [ ] `cargo test -p ckrv-sandbox` passes

---

### Task 3.2: Integration test for QwenProvider
**Priority**: P2
**Estimate**: 1h
**Files**: `crates/ckrv-sandbox/src/agent/tests.rs`

Add integration test that verifies:
- `build_command()` produces valid command
- `required_env_vars()` returns expected variables
- `config_mounts()` produces correct mount paths

**Acceptance Criteria**:
- [ ] Integration tests pass
- [ ] Document prerequisites: Node.js 20+ for CLI mode

---

## Dependencies

```
Phase 1 ─────────────────────────────────────────────────►
  Task 1.1 ──► Task 1.2 ──► Task 1.3
                   │
Phase 2 ───────────┼──────────────────────────────────────►
                   │
                   └───► Task 2.1 ──► Task 2.2
                                    │
Phase 3 ────────────────────────────┼─────────────────────►
                                    │
                                    └─► Task 3.1 ──► Task 3.2
```

## Notes

- Qwen Code requires Node.js 20+ for CLI mode (document as prerequisite)
- For API mode, uses OpenAI-compatible endpoints - supports Ollama, OpenRouter, ModelStudio
- CLI mode requires `--yes --approval-mode=auto` flags for headless/non-interactive execution
