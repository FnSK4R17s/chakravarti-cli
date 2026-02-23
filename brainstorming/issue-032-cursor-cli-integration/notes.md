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
- Register in `create_agent(...)`

### 2) New provider module

Create `crates/ckrv-sandbox/src/agent/cursor.rs` implementing `AgentProvider`:

- `name()` → `"Cursor CLI"`
- `agent_type()` → `AgentType::Cursor`
- `build_command(...)`:
  - Construct Cursor non-interactive execution call with prompt + working dir
  - Respect optional model/profile config if available in `AgentConfig`
- `required_env_vars()`:
  - Include required auth env(s) if Cursor supports env-based auth in this setup
  - If auth is file-based, return empty and rely on mount checks
- `config_mounts(...)`:
  - Mount Cursor config/auth directory read-only (e.g. `~/.cursor` or equivalent actual path used by Cursor CLI)
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
```

- Ensure `ckrv` validation surfaces clear error when Cursor auth/config is missing.

### 5) Testing

Add/extend tests for:
- `AgentType::from_str` mapping for cursor aliases
- `CursorProvider::build_command` expected command shape
- `create_agent` returns Cursor provider
- env/config validation behavior

## Open Questions

- [ ] Exact Cursor CLI binary and non-interactive flags to standardize on (needs confirmation from current Cursor CLI docs/version in use).
- [ ] Canonical auth strategy in sandbox: env vars vs mounted config files.
- [ ] Whether model selection is exposed in Cursor CLI and how to map from `AgentConfig`.
- [ ] Do we need a dedicated Docker image (like other providers) or can we extend an existing base image safely?

## Risks & Mitigations

| Risk | Impact | Mitigation |
|------|--------|------------|
| Cursor CLI invocation flags change across versions | Medium | Pin/test known compatible version; capture in docs and CI checks |
| Auth path mismatch in container | High | Add explicit startup validation + actionable error text |
| Behavior drift vs other providers | Medium | Keep output normalization and error schema aligned with current providers |

## Success Criteria

| Metric | Target |
|--------|--------|
| Cursor agent can be selected via config | ✅ |
| Cursor task executes in sandbox and returns normalized output | ✅ |
| Unit tests for parser/factory/provider command building | Added and passing |
| Docs updated with setup + troubleshooting | ✅ |

## Next Steps

- [ ] Confirm Cursor CLI command/auth contract (version + flags)
- [ ] Implement `CursorProvider` and `AgentType` wiring
- [ ] Add/adjust Docker image and validation checks
- [ ] Add tests and update docs
- [ ] Validate end-to-end run with Cursor agent in a sample repo

## References

- [Issue #32](https://github.com/FnSK4R17s/chakravarti-cli/issues/32)
- [Agent Extensibility Guide](../../crates/docs/agent-guide.md)
- Existing provider implementations:
  - `crates/ckrv-sandbox/src/agent/claude.rs`
  - `crates/ckrv-sandbox/src/agent/codex.rs`
  - `crates/ckrv-sandbox/src/agent/kilo.rs`
