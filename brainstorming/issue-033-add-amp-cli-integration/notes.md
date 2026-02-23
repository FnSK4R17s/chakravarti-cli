# Add AMP CLI integration

**Issue**: [#33](https://github.com/FnSK4R17s/chakravarti-cli/issues/33)
**Created**: 2026-02-23
**Status**: In Progress

## Problem Statement

Chakravarti currently supports Claude Code, Codex, and Kilo Code, but not AMP CLI as a first-class execution provider. Users who prefer AMP (or already pay for it) cannot select it directly in `agents.yaml`, run it in sandboxed execution, or use it in terminal sessions.

This leaves a capability gap against the product vision of cross-provider orchestration for users with multiple AI coding subscriptions.

## Current State

- Agent integration points are mature in `ckrv-sandbox` via `AgentProvider`.
- Existing providers (Claude, Codex, Kilo) define patterns for:
  - command construction
  - required env vars
  - credential mount strategy
  - output parsing / streaming normalization
- Docker images are per-agent (`docker/Dockerfile.<agent>`) and a combined image exists for multi-agent workflows.
- AMP is not represented in:
  - `AgentType` enums
  - provider factory registration
  - Docker image build/publish matrix
  - CLI/UI agent type selectors and docs

## Proposed Solution

Add AMP as a first-class agent provider by following the existing provider contract and integration flow:

1. Add `Amp` variant to sandbox `AgentType` and CLI-facing agent mappings.
2. Implement `AmpProvider` in `ckrv-sandbox/src/agent/amp.rs`.
3. Add `docker/Dockerfile.amp` and include AMP CLI installation + non-root execution.
4. Extend CLI/UI agent catalog and config loading (`agent_type: amp`).
5. Update docs, examples, and tests.

## User Stories

### US1: Configure AMP Agent
**As a** developer using AMP,
**I want** to define an AMP agent in `~/.config/chakravarti/agents.yaml`,
**So that** I can run specs with my preferred CLI provider.

### US2: Run AMP in Sandbox
**As a** user running isolated execution,
**I want** AMP tasks to run in Docker worktrees like other providers,
**So that** I retain the same safety and reproducibility guarantees.

### US3: Use AMP in Interactive Terminal
**As a** user debugging or iterating on a task,
**I want** `ckrv term --agent <amp-agent>` support,
**So that** AMP works in the same interactive workflow as Claude/Codex/Kilo.

## Technical Approach

### Option A: Native AMP Provider (Recommended)
Implement AMP with its own `AgentProvider` and Docker image.

**Pros**
- First-class UX in config, logs, and telemetry.
- Independent auth/mount behavior without overloading existing providers.
- Clear upgrade path for AMP-specific flags and output parsing.

**Cons**
- Requires ongoing maintenance for AMP CLI contract changes.
- Adds one more image to CI build matrix.

### Option B: Route AMP Through Kilo/OpenRouter
Use existing multi-provider bridge instead of dedicated AMP provider.

**Pros**
- Lower initial implementation effort.
- No new image/provider surface area.

**Cons**
- Not truly “AMP integration” (user intent mismatch for issue #33).
- Loses AMP-native auth/runtime behavior and branding.
- Harder debugging when failures occur across proxy layers.

### Decision

**Option A** (native provider) is preferred because issue #33 explicitly asks for AMP CLI integration and aligns with ckrv’s agent-as-worker architecture.

## Implementation Notes

### Integration Points (expected)

- `crates/ckrv-sandbox/src/agent/mod.rs`
  - add enum variant
  - parse aliases (e.g., `amp`, `ampcode`)
  - register in `create_agent()`
- `crates/ckrv-sandbox/src/agent/amp.rs`
  - `build_command()` with non-interactive/automation-safe flags
  - `required_env_vars()` (if API-key based mode exists)
  - `config_mounts()` for AMP auth/config files
  - `parse_output()` for success/failure normalization
- `docker/Dockerfile.amp`
  - install AMP CLI
  - create non-root `amp` user
  - configure HOME/workspace permissions
- `docker/` combined image and build scripts
  - include AMP binary in multi-agent image if required by current flows
- CLI/UI wiring
  - new agent type in config schema and UI selectors
  - help text/docs updates

### Authentication + Mounting Strategy (to validate)

Likely patterns:
- token via env var (if supported)
- local auth/config directory mount from host home into container home

Needs confirmation from AMP CLI docs for exact path + vars.

### Output/Streaming Considerations

- Prefer a machine-readable output mode (JSON/NDJSON) if AMP supports it.
- Fallback to plain stdout parsing should preserve deterministic success/error handling.
- Ensure parity with existing run logs and telemetry fields.

## Open Questions

- [ ] What is AMP CLI’s exact non-interactive invocation contract for one-shot prompts?
- [ ] Does AMP support structured output mode suitable for streaming parser integration?
- [ ] Which auth files/env vars are required and where are they stored on host machines?
- [ ] Does AMP require additional Linux packages in container images?
- [ ] Should AMP be included in `docker/Dockerfile.agent` immediately or staged later?

## Success Criteria

| Metric | Target |
|--------|--------|
| Config | `agent_type: amp` validates and loads |
| Execution | `ckrv task run --agent amp-*` executes successfully in sandbox |
| Terminal | `ckrv term --agent amp-*` launches AMP session |
| Reliability | Output parsing returns normalized success/failure without regressions |
| Docs | Agent guide + config examples include AMP |

## Next Steps

- [ ] Validate AMP CLI install/auth/command flags from official docs.
- [ ] Draft provider + Docker implementation plan in `tasks.md`.
- [ ] Move status to **Ready for Spec** once open questions are resolved.

## References

- https://github.com/FnSK4R17s/chakravarti-cli/issues/33
- `crates/docs/agent-guide.md`
- `crates/ckrv-sandbox/src/agent/mod.rs`
- `crates/ckrv-sandbox/src/agent/{claude,codex,kilo}.rs`
- https://ampcode.com/
