# Add Gemini CLI Integration - Tasks

**Issue**: [#31](https://github.com/FnSK4R17s/chakravarti-cli/issues/31)
**Brainstorm**: [notes.md](./notes.md)
**Created**: 2026-02-24

## Task Overview

| Phase | Tasks | Estimate |
|-------|-------|----------|
| Phase 1: Research | 2 | 2h |
| Phase 2: Implementation | 3 | 3h |
| Phase 3: Testing | 1 | 1h |
| **Total** | 6 | 6h |

---

## Phase 1: Research

### Task 1.1: Research Gemini CLI Interface
**Priority**: P0
**Estimate**: 1h
**Files**: N/A

Research the Gemini CLI command-line interface to understand:
- CLI command and arguments (e.g., `gemini`, `gemini-cli`)
- Available flags (--resume, --print, --model, etc.)
- Environment variables needed (GEMINI_API_KEY, etc.)
- Config file locations

**Acceptance Criteria**:
- [ ] Document exact CLI command format
- [ ] List required environment variables
- [ ] Identify config files to mount into container
- [ ] Note any differences from Claude/Codex interface

---

### Task 1.2: Review Existing Agent Implementations
**Priority**: P0
**Estimate**: 1h
**Files**: `crates/ckrv-sandbox/src/agent/claude.rs`, `crates/ckrv-sandbox/src/agent/codex.rs`

Study the existing agent implementations to understand patterns:
- How prompts are passed to agents
- How output is parsed
- How config mounts work
- Error handling patterns

**Acceptance Criteria**:
- [ ] Document key implementation patterns from claude.rs
- [ ] Document key implementation patterns from codex.rs
- [ ] Note any agent-specific nuances

---

## Phase 2: Implementation

### Task 2.1: Add Gemini Variant to AgentType Enum
**Priority**: P1
**Estimate**: 30m
**Files**: `crates/ckrv-sandbox/src/agent/mod.rs`

Add the Gemini agent type to the enum and implement:
- `AgentType::Gemini` variant
- Update `from_str()` to parse "gemini" 
- Update `display_name()` to return "Gemini CLI"

**Acceptance Criteria**:
- [ ] AgentType::Gemini variant added
- [ ] from_str("gemini") returns Some(AgentType::Gemini)
- [ ] display_name() returns "Gemini CLI"
- [ ] `cargo check -p ckrv-sandbox` passes

---

### Task 2.2: Create GeminiProvider Implementation
**Priority**: P1
**Estimate**: 2h
**Files**: `crates/ckrv-sandbox/src/agent/gemini.rs` (new)

Create the Gemini provider implementing AgentProvider trait:
- Implement `name()` - return "Gemini CLI"
- Implement `agent_type()` - return AgentType::Gemini
- Implement `build_command()` - construct CLI command with prompt
- Implement `required_env_vars()` - return ["GEMINI_API_KEY"]
- Implement `config_mounts()` - mount any config files
- Implement `parse_output()` - parse Gemini output into AgentOutput

**Acceptance Criteria**:
- [ ] GeminiProvider struct defined
- [ ] New method creates provider instance
- [ ] All trait methods implemented
- [ ] `cargo check -p ckrv-sandbox` passes
- [ ] Follows patterns from claude.rs and codex.rs

---

### Task 2.3: Register GeminiProvider in Factory
**Priority**: P1
**Estimate**: 15m
**Files**: `crates/ckrv-sandbox/src/agent/mod.rs`

Register the new provider in the create_agent function and add module:
- Add `mod gemini;` declaration
- Add `pub use gemini::GeminiProvider;` 
- Add case in `create_agent()` function

**Acceptance Criteria**:
- [ ] Module imported in mod.rs
- [ ] create_agent(AgentType::Gemini) returns GeminiProvider
- [ ] `cargo check -p ckrv-sandbox` passes

---

## Phase 3: Testing

### Task 3.1: Add Tests and Verify Build
**Priority**: P2
**Estimate**: 1h
**Files**: `crates/ckrv-sandbox/src/agent/tests.rs`

Add tests for the Gemini provider:
- Test AgentType::from_str("gemini")
- Test AgentType::display_name()
- Test create_agent(AgentType::Gemini)
- Test basic command building

**Acceptance Criteria**:
- [ ] Unit tests added for Gemini integration
- [ ] `cargo test -p ckrv-sandbox` passes
- [ ] `cargo clippy -p ckrv-sandbox` passes with no warnings

---

## Dependencies

```
Phase 1: Research ─────────────────────────────────────────────────►
  Task ─► Task 1.1 ─1.2
        │
        ▼
Phase 2: Implementation ────────────────────────────────────────────►
  Task 2.1 ──► Task 2.2 ──► Task 2.3
        │
        ▼
Phase 3: Testing ───────────────────────────────────────────────────►
  Task 3.1
```

## Notes

- Gemini CLI interface research is blocking - do this first
- Follow claude.rs patterns for implementation
- May need to add GEMINI_API_KEY to .env.example if applicable
