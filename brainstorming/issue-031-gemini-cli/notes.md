# Add Gemini CLI Integration

**Issue**: [#31](https://github.com/FnSK4R17s/chakravarti-cli/issues/31)
**Created**: 2026-02-23
**Status**: In Progress

## Problem Statement

Chakravarti currently supports Claude Code, OpenAI Codex, and Kilo Code as first-class agent providers. Gemini access is indirect today (via Kilo/OpenRouter model routing), but there is no native Gemini CLI provider in the same way Claude and Codex are integrated.

This creates friction for users who want to use Gemini CLI credentials/config directly, route Gemini tasks explicitly in agent YAML, and treat Gemini as a first-class orchestration worker.

## Current State

- `ckrv-sandbox` currently has providers:
  - `ClaudeProvider`
  - `CodexProvider`
  - `KiloCodeProvider`
- Provider abstraction is centralized in `crates/ckrv-sandbox/src/agent/mod.rs` via `AgentProvider` trait and `AgentType` enum.
- CLI config layer (`crates/ckrv-cli/src/services/agent_lookup.rs`) supports:
  - `claude`
  - `claude_openrouter`
  - `claude_glm`
  - `codex`
  - `kilo_code`
- Docs (`crates/docs/agent-guide.md`) already describe multi-provider architecture and provider onboarding pattern.

Pain point: users can run Gemini models through Kilo/OpenRouter, but cannot configure a dedicated `gemini` agent type that maps cleanly to Gemini CLI auth/config semantics.

## Proposed Solution

Add native Gemini CLI integration as a new first-class provider, following the existing provider architecture.

### High-level design

1. Add `Gemini` variant to sandbox `AgentType` and provider factory.
2. Implement `GeminiProvider` in `ckrv-sandbox` that:
   - Builds Gemini CLI command for non-interactive execution.
   - Mounts Gemini CLI config/auth paths into container.
   - Parses stdout/stderr into `AgentOutput`.
3. Extend CLI config enum in `agent_lookup.rs` with `gemini` (snake_case serialization compatibility).
4. Update Docker image(s) used for agent execution to include Gemini CLI binary and runtime deps.
5. Add provider unit tests + command construction tests similar to Claude/Codex/Kilo test coverage.
6. Update agent docs and sample YAML snippets.

## User Stories

### US1: Configure Gemini as a dedicated executor
**As a** user with Gemini CLI configured,
**I want** to define an agent with `agent_type: gemini`,
**So that** I can route tasks to Gemini without a wrapper provider.

### US2: Keep orchestration model-agnostic but provider-explicit
**As a** power user with multiple AI subscriptions,
**I want** Gemini represented as its own provider,
**So that** planning/routing decisions are transparent and maintainable.

### US3: Run Gemini in the same sandbox contract as other providers
**As a** maintainer,
**I want** Gemini to conform to `AgentProvider`,
**So that** runner/orchestrator layers require minimal or no special-case logic.

## Technical Approach

### Options Considered

| Option | Pros | Cons |
|--------|------|------|
| Keep Gemini via Kilo/OpenRouter only | No new provider code | Not first-class, weaker UX and config clarity for Gemini users |
| Add native Gemini provider in `ckrv-sandbox` | Consistent with architecture, clear routing, explicit agent type | Requires CLI/install/mount/test work |
| Add Gemini only in UI layer | Faster visual support | Incomplete: backend execution path still missing |

### Decision

Add a native `GeminiProvider` in `ckrv-sandbox` and wire through CLI config + docs. This best aligns with project vision (multi-provider orchestration with explicit provider roles), and mirrors existing provider integration standards.

## Implementation Notes

### Candidate files to touch

- `crates/ckrv-sandbox/src/agent/mod.rs`
  - Add `mod gemini;`, export provider, add `AgentType::Gemini`, parsing aliases, display name, factory branch.
- `crates/ckrv-sandbox/src/agent/gemini.rs` (new)
  - Provider implementation.
- `crates/ckrv-sandbox/src/agent/tests.rs`
  - Add Gemini parse/build/mount tests.
- `crates/ckrv-cli/src/services/agent_lookup.rs`
  - Add `Gemini` to config-layer `AgentType` enum.
- `docker/Dockerfile.agent` (and/or dedicated Dockerfile)
  - Install Gemini CLI and ensure executable availability in runtime image.
- `crates/docs/agent-guide.md`
  - Add Gemini to supported tool/auth tables + “adding provider” examples if needed.

### Likely command/mount contract (to verify)

- Gemini CLI binary + non-interactive prompt flag behavior must be confirmed.
- Config/auth directory likely under user home (e.g., `.gemini*` or config dir); exact path must be validated before final implementation.

## Open Questions

- [ ] What exact non-interactive Gemini CLI command shape should `build_command()` generate?
- [ ] Does Gemini CLI support explicit working directory flags (`--cwd` or equivalent), or must execution rely on process cwd?
- [ ] Which auth/config files must be mounted read-only into container for successful execution?
- [ ] Should Gemini get a dedicated Dockerfile (`docker/Dockerfile.gemini`) or be added to the shared agent image?
- [ ] Are there provider-specific env vars to validate in `required_env_vars()` versus file-based auth only?

## Success Criteria

| Metric | Target |
|--------|--------|
| Provider wiring | `AgentType` + factory include Gemini in sandbox and CLI layers |
| Build health | `cargo build --workspace` passes |
| Test health | New Gemini provider tests pass in `ckrv-sandbox` |
| Docker readiness | Agent image builds with Gemini CLI available |
| Runtime smoke | Minimal task execution with Gemini provider succeeds end-to-end |

## Next Steps

- [ ] Confirm Gemini CLI command/auth/mount semantics from official docs.
- [ ] Implement provider and enum wiring.
- [ ] Add/update Docker image(s).
- [ ] Add tests and docs.
- [ ] Run full verification and refine based on failures.

## References

- Issue: https://github.com/FnSK4R17s/chakravarti-cli/issues/31
- Existing provider architecture: `crates/ckrv-sandbox/src/agent/mod.rs`
- Existing provider example: `crates/ckrv-sandbox/src/agent/kilo.rs`
- Agent config lookup: `crates/ckrv-cli/src/services/agent_lookup.rs`
- Agent docs: `crates/docs/agent-guide.md`
- Vision alignment: `guiding_docs/vision.md`
