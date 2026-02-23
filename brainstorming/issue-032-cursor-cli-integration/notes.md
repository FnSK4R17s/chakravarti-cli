# Add Cursor CLI integration

**Issue**: [#32](https://github.com/FnSK4R17s/chakravarti-cli/issues/32)
**Created**: 2026-02-23
**Status**: In Progress

## Problem Statement

`ckrv` currently supports Claude Code, Codex, and Kilo Code, but does not support Cursor CLI as an execution provider. Teams already paying for Cursor cannot route tasks through Cursor in orchestrated runs, reducing the core value proposition of `ckrv` (multi-subscription orchestration across providers).

## Current State

- Agent abstraction exists via `AgentProvider` in `ckrv-sandbox`.
- Implemented providers: Claude, Codex, Kilo.
- Agent lifecycle and selection are already wired through CLI/UI config and runtime.
- Per `crates/docs/agent-guide.md`, adding a new provider follows a repeatable flow:
  1. Add `AgentType` variant
  2. Implement provider module
  3. Register factory mapping
  4. Ensure container/runtime dependencies
  5. Add tests and docs

## Proposed Solution

Add a first-class `CursorProvider` under the same provider model used by existing integrations.

### High-level behavior

- New `agent_type`: `cursor` (plus alias parsing e.g., `cursor-cli`)
- Provider command construction for non-interactive prompt execution
- Env/config mounts required for Cursor auth/config
- Output normalization into `AgentOutput`
- Agent appears in YAML config, CLI selection, and UI management

## User Stories

### US1: Cursor subscriber can execute tasks via ckrv
**As a** developer with a Cursor subscription,
**I want** to configure a Cursor agent in `agents.yaml`,
**So that** `ckrv run` can assign and execute tasks through Cursor CLI.

### US2: Multi-agent routing includes Cursor
**As a** developer with multiple AI subscriptions,
**I want** to mix Cursor with Claude/Codex/Kilo agents,
**So that** I can optimize cost/speed/quality per task.

### US3: Operational clarity
**As a** maintainer,
**I want** predictable auth validation and actionable errors for Cursor,
**So that** failures are diagnosable and support burden is low.

## Technical Approach

### Options Considered

| Option | Pros | Cons |
|--------|------|------|
| A) Native provider in `ckrv-sandbox` (recommended) | Consistent architecture, clean UX, reusable in CLI/UI, testable | Requires provider-specific command/env/mount logic |
| B) Generic shell provider + custom command string | Faster initial patch | Poor typing, fragile UX, weak validation, harder docs/support |

### Decision

**Option A**. Cursor should be integrated as a proper provider, not a shell escape hatch, to preserve ckrv’s provider abstraction quality and maintainability.

## Implementation Notes

### 1) Provider wiring

- Add `Cursor` variant to `AgentType` in `crates/ckrv-sandbox/src/agent/mod.rs`
- Extend parser aliases:
  - `cursor`
  - `cursor-cli`
  - `cursor-agent`
- Register in `create_agent(...)`

### 2) New provider module

Create `crates/ckrv-sandbox/src/agent/cursor.rs` implementing `AgentProvider`:

- `name()` → `"Cursor CLI"`
- `agent_type()` → `AgentType::Cursor`
- `build_command(...)`:
  ```rust
  let mut cmd = vec!["cursor-agent".to_string()];
  cmd.push("--print".to_string());  // Non-interactive mode
  cmd.push("-p".to_string());       // Short flag
  cmd.push(prompt.to_string());
  
  // Optional model override
  if let Some(ref model) = config.model {
      cmd.push("--model".to_string());
      cmd.push(model.to_string());
  }
  
  cmd.push("--force".to_string());           // Auto-approve
  cmd.push("--output-format=text".to_string());
  cmd.push("--cwd".to_string());
  cmd.push(workdir.to_string_lossy().to_string());
  ```
- `required_env_vars()`:
  - `CURSOR_API_KEY` (primary auth method)
  - Alternative: `--api-key` CLI flag
- `config_mounts(...)`:
  - Mount `~/.cursor` directory for config/MCP (read-only)
- `parse_output(...)`:
  - Keep normalization consistent with existing providers

### 3) Docker/runtime

- Ensure execution image used for agent runs has Cursor CLI installed.
- Follow same non-root execution rule used for other agent images.
- Add a health/version check in image build pipeline to fail early if Cursor binary is missing.

### 4) Config and UX

- Add sample config entry in docs:

```yaml
agents:
  - id: cursor-main
    name: Cursor CLI
    agent_type: cursor
    enabled: true
    description: Cursor-based execution provider
    # Optional: specify model
    # model: auto  # Options: auto, composer 1, opus 4.6, sonnet 4.6, gpt-5.2, gpt-5.3, codex, gemini 3 pro, grok
```

- Ensure `ckrv` validation surfaces clear error when Cursor auth/config is missing.

### 5) Testing

Add/extend tests for:
- `AgentType::from_str` mapping for cursor aliases
- `CursorProvider::build_command` expected command shape
- `create_agent` returns Cursor provider
- env/config validation behavior

## Open Questions

- [x] Exact Cursor CLI command and non-interactive flags (CONFIRMED)
  - Command: `cursor-agent` (not `cursor`)
  - Non-interactive: `--print` or `-p` flag
  - Full example: `cursor-agent -p "prompt" --model auto --force --output-format=text --print`
  - Version check: `cursor-agent --version`
- [x] Canonical auth strategy in sandbox: env vars vs mounted config files (CONFIRMED)
  - Primary: `CURSOR_API_KEY` environment variable (recommended)
  - Alternative: `--api-key` flag for CLI
  - Browser login also supported: `cursor-agent login` (not useful for sandbox)
- [x] Config directory location (CONFIRMED)
  - Path: `~/.cursor` (similar structure to Claude's `~/.claude`)
  - Can mount for MCP server configs if needed
- [x] Model selection (CONFIRMED)
  - Supported models: `auto`, `composer 1`, `opus 4.6`, `sonnet 4.6`, `gpt-5.2`, `gpt-5.3`, `codex`, `gemini 3 pro`, `grok`
  - Use `--model` flag to specify
- [ ] Do we need a dedicated Docker image or can we extend an existing base image?

## Risks & Mitigations

| Risk | Impact | Mitigation |
|------|--------|------------|
| Cursor CLI invocation flags change across versions | Medium | Pin/test known compatible version in Docker image; capture in docs and CI checks |
| CURSOR_API_KEY missing in container | High | Validate env var presence at startup with actionable error; require explicit config |
| Behavior drift vs other providers | Medium | Keep output normalization and error schema aligned with current providers |
| CLI hangs in non-interactive mode | Medium | Add timeout wrapper; use `--force` flag; monitor for known hang issues |

## Success Criteria

| Metric | Target |
|--------|--------|
| Cursor agent can be selected via config | ✅ |
| Cursor task executes in sandbox and returns normalized output | ✅ |
| Unit tests for parser/factory/provider command building | Added and passing |
| Docs updated with setup + troubleshooting | ✅ |

## Next Steps

- [x] Confirm Cursor CLI command/auth contract (version + flags) - DONE via docs research
- [ ] Implement `CursorProvider` and `AgentType` wiring
- [ ] Add/adjust Docker image with Cursor CLI installation
- [ ] Add tests and update docs
- [ ] Validate end-to-end run with Cursor agent in a sample repo

## References

- [Issue #32](https://github.com/FnSK4R17s/chakravarti-cli/issues/32)
- [Agent Extensibility Guide](../../crates/docs/agent-guide.md)
- Existing provider implementations:
  - `crates/ckrv-sandbox/src/agent/claude.rs`
  - `crates/ckrv-sandbox/src/agent/codex.rs`
  - `crates/ckrv-sandbox/src/agent/kilo.rs`
