# Add AMP CLI Integration - Tasks

**Issue**: [#33](https://github.com/FnSK4R17s/chakravarti-cli/issues/33)
**Brainstorm**: [notes.md](./notes.md)
**Created**: 2026-02-23

## Task Overview

| Phase | Tasks | Estimate |
|-------|-------|----------|
| Phase 1: Backend (Sandbox) | 3 | 3h |
| Phase 2: CLI Integration | 3 | 2h |
| Phase 3: UI Updates | 2 | 1h |
| **Total** | 8 | 6h |

---

## Phase 1: Backend (Sandbox)

### Task 1.1: Add Amp to AgentType enum
**Priority**: P0
**Estimate**: 30m
**Files**: `crates/ckrv-sandbox/src/agent/mod.rs`

Add `Amp` variant to the `AgentType` enum and implement `from_str()` and `display_name()` methods.

**Acceptance Criteria**:
- [ ] `AgentType::Amp` added to enum
- [ ] `from_str("amp")` returns `Some(AgentType::Amp)`
- [ ] `display_name()` returns "AMP"
- [ ] `cargo check -p ckrv-sandbox` passes

---

### Task 1.2: Create AmpProvider implementation
**Priority**: P0
**Estimate**: 2h
**Files**: `crates/ckrv-sandbox/src/agent/amp.rs` (new)

Create `AmpProvider` struct implementing the `AgentProvider` trait:
- `build_command()` - Construct `amp` CLI command
- `required_env_vars()` - Return any required env vars (e.g., `AMPCODE_API_KEY`)
- `config_mounts()` - Return any config file mounts
- `parse_output()` - Parse AMP CLI output into `AgentOutput`

**Acceptance Criteria**:
- [ ] `AmpProvider` implements `AgentProvider` trait
- [ ] Module exports `AmpProvider` from `mod.rs`
- [ ] `cargo check -p ckrv-sandbox` passes

---

### Task 1.3: Add Amp to provider factory
**Priority**: P0
**Estimate**: 15m
**Files**: `crates/ckrv-sandbox/src/agent/mod.rs`

Update `create_agent()` function to return `Box::new(AmpProvider::new())` for `AgentType::Amp`.

**Acceptance Criteria**:
- [ ] `create_agent(AgentType::Amp)` returns `AmpProvider`
- [ ] `cargo test -p ckrv-sandbox` passes

---

## Phase 2: CLI Integration

### Task 2.1: Add Amp to CLI AgentType enum
**Priority**: P1
**Estimate**: 15m
**Files**: `crates/ckrv-cli/src/services/agent_lookup.rs`

Add `Amp` variant to the CLI `AgentType` enum in `agent_lookup.rs`.

**Acceptance Criteria**:
- [ ] `AgentType::Amp` added to CLI enum
- [ ] `cargo check -p ckrv-cli` passes

---

### Task 2.2: Handle Amp in CLI agent mapping
**Priority**: P1
**Estimate**: 1h
**Files**: `crates/ckrv-cli/src/commands/term.rs`

Update `to_sandbox_agent_type()` to map `AgentType::Amp` to `ckrv_sandbox::AgentType::Amp`. Add Amp handling in:
- Type badge matching (line ~511)
- Sandbox image selection (line ~1078)
- Container home path (line ~1091)

**Acceptance Criteria**:
- [ ] `to_sandbox_agent_type(&AgentType::Amp)` returns `ckrv_sandbox::AgentType::Amp`
- [ ] Type badge shows "amp"
- [ ] `cargo check -p ckrv-cli` passes

---

### Task 2.3: Add Amp option to agent list
**Priority**: P1
**Estimate**: 30m
**Files**: `crates/ckrv-cli/src/commands/term.rs`

Add `AgentType::Amp` to appropriate agent lists (e.g., `CLAUDE_AGENTS` or new `AMP_AGENTS` constant).

**Acceptance Criteria**:
- [ ] Amp appears in agent selection prompts
- [ ] `cargo check -p ckrv-cli` passes

---

## Phase 3: UI Updates

### Task 3.1: Add Amp to TypeScript types
**Priority**: P2
**Estimate**: 30m
**Files**: `crates/ckrv-ui/frontend/src/types/api.generated.ts`

Add "amp" to the `AgentType` union in TypeScript types.

**Acceptance Criteria**:
- [ ] `AgentType` includes "amp"
- [ ] `npm run typecheck` passes (if available)

---

### Task 3.2: Add Amp to UI dropdown
**Priority**: P2
**Estimate**: 30m
**Files**: `crates/ckrv-ui/frontend/src/components/AgentManager.tsx`

Add "amp" option to agent dropdown in the UI.

**Acceptance Criteria**:
- [ ] Amp appears in agent selection dropdown
- [ ] UI builds successfully

---

## Dependencies

```
Phase 1 ─────────────────────────────────────────────────►
  Task 1.1 ──► Task 1.2 ──► Task 1.3
                   │
Phase 2 ───────────┼──────────────────────────────────────►
                   │
                   └───► Task 2.1 ──► Task 2.2 ──► Task 2.3
                                           
Phase 3 ─────────────────────────────────────────────────►
  Task 3.1 ──► Task 3.2
```

---

## Notes

- AMP CLI is installed via `curl -fsSL https://ampcode.com/install.sh | bash`
- Research AMP CLI command-line interface to determine exact execution model
- Consider if Docker image needed or direct CLI invocation suffices
