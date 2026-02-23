# Add Qwen Code Agent Integration - Tasks

**Issue**: [#34](https://github.com/FnSK4R17s/chakravarti-cli/issues/34)
**Brainstorm**: [notes.md](./notes.md)
**Created**: 2026-02-23
**Status**: Pending

## Pre-Flight: Verify CLI API

Before writing any `build_command` code, verify the Qwen Code CLI interface:

```bash
npm install -g @qwen-code/cli
qwen --help
qwen --version
```

Confirm:
- [ ] Binary name (`qwen` or `qwen-code`)
- [ ] Non-interactive flag (`--no-interactive`, `--print`, or other)
- [ ] Workdir flag (`--directory`, `--cwd`, or env-based)
- [ ] Auth: `DASHSCOPE_API_KEY` env var or config file only
- [ ] Config directory path (`~/.qwen/` or `~/.config/qwen/`)

> Update `notes.md` open questions and this tasks file before implementing Phase 1.

---

## Task Overview

| Phase | Tasks | Estimate | Status |
|-------|-------|----------|--------|
| Phase 1: Core Provider | 3 | ~2h | ⏳ Pending |
| Phase 2: Docker & Mounts | 3 | ~2h | ⏳ Pending |
| Phase 3: CLI Integration & Docs | 2 | ~1.5h | ⏳ Pending |
| **Total** | **8** | **~5.5h** | **⏳** |

---

## Dependencies

```
Pre-Flight ──────────────────────────────────────────────►
  (CLI flag verification)
       │
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

(P) = can run in parallel after T003 completes.

---

## Phase 1: Core Provider

### Task 1.1 — Create QwenCodeProvider implementation

**ID**: T001  
**Priority**: P0  
**Estimate**: 1h  
**Files**: `crates/ckrv-sandbox/src/agent/qwen.rs` (new file)

**Acceptance Criteria**:
- [ ] `QwenCodeProvider` struct exists with `#[derive(Debug, Default)]`
- [ ] `AgentProvider` trait fully implemented (all 5 methods)
- [ ] `build_command()` generates the correct non-interactive Qwen Code invocation with workdir and optional model override
- [ ] `required_env_vars()` returns `["DASHSCOPE_API_KEY"]`
- [ ] `config_mounts()` mounts `~/.qwen/` if it exists (read-write, matching Kilo pattern)
- [ ] `parse_output()` returns normalized `AgentOutput` based on exit code
- [ ] Module-level doc comment (`//!`) describes the provider
- [ ] `cargo check -p ckrv-sandbox` passes

---

### Task 1.2 — Register QwenCode in agent module

**ID**: T002  
**Priority**: P0  
**Estimate**: 30m  
**Files**: `crates/ckrv-sandbox/src/agent/mod.rs`

**Acceptance Criteria**:
- [ ] `mod qwen;` added to module declarations
- [ ] `pub use qwen::QwenCodeProvider;` exported
- [ ] `AgentType::QwenCode` variant added to enum with doc comment
- [ ] `AgentType::from_str` handles: `"qwen"`, `"qwen-code"`, `"qwencode"`, `"qwen_code"` → `Some(Self::QwenCode)`
- [ ] `AgentType::display_name()` returns `"Qwen Code"` for the new variant
- [ ] `create_agent(AgentType::QwenCode)` returns `Box<QwenCodeProvider>`
- [ ] All existing `match` arms on `AgentType` remain exhaustive (no `_` wildcards hiding missing arms)
- [ ] `cargo check -p ckrv-sandbox` passes

---

### Task 1.3 — Add tests for Qwen provider

**ID**: T003  
**Priority**: P1  
**Estimate**: 30m  
**Files**: `crates/ckrv-sandbox/src/agent/tests.rs`

**Acceptance Criteria**:
- [ ] `test_qwen_code_from_str()` — verifies all string aliases parse correctly
- [ ] `test_qwen_code_build_command_basic()` — verifies non-interactive flag and prompt appear in output
- [ ] `test_qwen_code_build_command_with_model()` — verifies `--model qwen3-coder-480b-a22b` appears when model is set
- [ ] `test_qwen_code_build_command_workdir()` — verifies workdir flag present and correct
- [ ] `test_qwen_code_required_env_vars()` — asserts `["DASHSCOPE_API_KEY"]`
- [ ] `test_qwen_code_parse_output_success()` — exit 0 → `success: true`
- [ ] `test_qwen_code_parse_output_failure()` — exit 1 → `success: false`
- [ ] `cargo test -p ckrv-sandbox` passes with all tests green

---

## Phase 2: Docker & Mounts

### Task 2.1 — Create Dockerfile.qwen

**ID**: T004 (parallel)  
**Priority**: P1  
**Estimate**: 30m  
**Files**: `docker/Dockerfile.qwen` (new file)

**Acceptance Criteria**:
- [ ] `docker/Dockerfile.qwen` exists
- [ ] Base image: `node:22-slim` (matching `Dockerfile.codex` and `Dockerfile.kilo`)
- [ ] Installs system deps: `git curl ca-certificates`
- [ ] Installs Qwen Code CLI globally via npm
- [ ] Creates non-root user `qwen` with home `/home/qwen`
- [ ] Creates `/home/qwen/.qwen/` with correct ownership
- [ ] Creates `/workspace` with `qwen:qwen` ownership
- [ ] Sets `WORKDIR /workspace` and `ENV HOME=/home/qwen`
- [ ] Runs `qwen --version || true` before `USER` switch to verify install
- [ ] Ends with `USER qwen` directive (non-root required)
- [ ] `docker build -f docker/Dockerfile.qwen .` succeeds locally

---

### Task 2.2 — Add Qwen CLI to Dockerfile.agent

**ID**: T005 (parallel)  
**Priority**: P1  
**Estimate**: 30m  
**Files**: `docker/Dockerfile.agent`

**Acceptance Criteria**:
- [ ] Qwen Code CLI installed alongside Claude Code, Codex, and Kilo Code in the combined image
- [ ] `~/.qwen/` config directory created with correct permissions in the combined image
- [ ] Version verification step added (`qwen --version || true`)
- [ ] `docker build -f docker/Dockerfile.agent .` succeeds

---

### Task 2.3 — Verify docker mount integration

**ID**: T006  
**Priority**: P1  
**Estimate**: 30m  
**Files**: `crates/ckrv-sandbox/src/docker.rs` (read-only verify, modify only if needed)

**Acceptance Criteria**:
- [ ] Confirm `config_mounts()` from `QwenCodeProvider` is called correctly through the `AgentProvider` trait dispatch in `docker.rs`
- [ ] If `docker.rs` has any agent-type-specific mount logic that bypasses the trait, add Qwen Code handling there
- [ ] `cargo check --workspace` passes

> **Expected result**: No changes needed. The trait interface handles this automatically (same result as Kilo Code T006).

---

## Phase 3: CLI Integration & Docs

### Task 3.1 — Add QwenCode to CLI agent_lookup

**ID**: T007  
**Priority**: P1  
**Estimate**: 30m  
**Files**: `crates/ckrv-cli/src/services/agent_lookup.rs`, `crates/ckrv-cli/src/commands/term.rs`

**Acceptance Criteria**:
- [ ] `AgentType::QwenCode` variant added to the CLI `AgentType` enum in `agent_lookup.rs`
- [ ] `#[serde(rename_all = "snake_case")]` ensures `agent_type: qwen_code` in YAML deserializes to `QwenCode`
- [ ] All `match` expressions on the CLI `AgentType` in `term.rs` are updated (audit all match sites)
- [ ] `cargo check --workspace` passes with zero warnings
- [ ] Manual test: a `agents.yaml` with `agent_type: qwen_code` loads without error

---

### Task 3.2 — Update agent-guide documentation

**ID**: T008  
**Priority**: P2  
**Estimate**: 30m  
**Files**: `crates/docs/agent-guide.md`

**Acceptance Criteria**:
- [ ] Qwen Code added to the **Agentic Coding Tools** table (Tool, Provider, Description)
- [ ] Qwen Code added to the **Authentication Methods** table (`DASHSCOPE_API_KEY` env var)
- [ ] Architecture Mermaid diagram updated with `QwenCode[QwenCodeProvider]` node
- [ ] **Qwen Code Integration** section added below Kilo Code section, covering:
  - Prerequisites (npm install, API key)
  - `agents.yaml` configuration example
  - CLI usage example (`ckrv task run --agent qwen-agent`)
  - Known model IDs (qwen3-coder-480b-a22b, qwen-coder-turbo, qwen-coder-plus)
  - Key differences from Claude/Codex/Kilo table
- [ ] Links to Qwen Code GitHub and DashScope API docs
- [ ] `last_updated` frontmatter updated to `2026-02-23`

---

## Validation Checklist

- [ ] `cargo check --workspace` succeeds with zero warnings
- [ ] `cargo test -p ckrv-sandbox` — all agent tests pass (existing + new Qwen tests)
- [ ] `docker build -f docker/Dockerfile.qwen .` succeeds
- [ ] `docker build -f docker/Dockerfile.agent .` succeeds
- [ ] `agents.yaml` with `agent_type: qwen_code` loads and resolves correctly
- [ ] Documentation updated in `agent-guide.md`

## Out of Scope

The following are explicitly deferred to follow-up issues:
- UI Agent Manager card for Qwen Code (requires `ckrv-transport` handler + frontend component)
- `/agents/qwen-models` API route for dynamic model discovery from DashScope
- Streaming output parsing (JSON event format) — add in follow-up once CLI streaming flags are confirmed
