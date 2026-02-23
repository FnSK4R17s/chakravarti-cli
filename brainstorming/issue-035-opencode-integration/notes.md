# Add Opencode integration

**Issue**: [#35](https://github.com/FnSK4R17s/chakravarti-cli/issues/35)
**Created**: 2026-02-23
**Status**: In Progress

## Problem Statement

chakravarti-cli currently supports multiple agent CLIs, but Opencode is not yet a first-class integration. Users with existing Opencode setup cannot route execution through `ckrv` without custom hacks.

This blocks a key part of the product vision: orchestrating *all* major coding CLIs through one spec-first interface.

## Current State

- Existing integrations follow a common provider pattern (`AgentProvider` in `ckrv-sandbox`).
- New providers usually require updates across:
  - provider implementation in `crates/ckrv-sandbox/src/agent/`
  - enum/factory wiring in `agent/mod.rs`
  - CLI-facing lookup in `crates/ckrv-cli/src/services/agent_lookup.rs`
  - Docker/mount behavior in `crates/ckrv-sandbox/src/docker.rs`
  - docs (agent guide + user-facing command docs)
- There is a symlinked `.opencode` config in repo roots, indicating ecosystem intent but no full provider implementation yet.

## Proposed Solution

Add a native `OpencodeProvider` integration using the same contract as existing providers.

### High-level behavior

1. User configures an agent with `agent_type: opencode`.
2. `ckrv` routes task execution to Opencode CLI inside sandbox/worktree.
3. Provider supports model override pass-through and deterministic non-interactive run mode.
4. Config and auth data mount into sandbox safely (read-only where possible).

## User Stories

### US1: Run tasks with Opencode
**As a** user who already uses Opencode,
**I want** to select Opencode as an executor in `ckrv`,
**So that** I can use my existing Opencode account/models in orchestrated workflows.

### US2: Switch providers without spec rewrite
**As a** user with multiple AI CLI subscriptions,
**I want** to swap task agent from Claude/Codex to Opencode via config,
**So that** I can optimize quality/cost without changing my specs.

### US3: Keep sandboxed execution behavior
**As a** reliability-focused user,
**I want** Opencode tasks to run with the same isolation guarantees as other providers,
**So that** introducing Opencode does not reduce execution safety.

## Technical Approach

### Options Considered

| Option | Pros | Cons |
|--------|------|------|
| Add minimal Opencode wrapper in CLI crate only | Fast initial patch | Breaks architecture consistency; bypasses sandbox provider model |
| Full provider integration in `ckrv-sandbox` (recommended) | Matches existing design, reusable by orchestrator, testable | Slightly more up-front wiring |
| External plugin architecture first, then Opencode | Most extensible long-term | Overkill for issue #35 scope |

### Decision

**Use full provider integration in `ckrv-sandbox`** to preserve architecture consistency and avoid special-case execution paths.

### Implementation Sketch

- Add `opencode.rs` provider with:
  - `name()` => `"Opencode"`
  - `agent_type()` => `AgentType::Opencode`
  - command builder for non-interactive execution
  - optional model pass-through
  - required env var and mount declaration matching Opencode auth mechanism
- Wire provider into module exports + factory.
- Add CLI lookup/type parsing support.
- Add tests for command construction and provider registration.
- Add docs for setup and troubleshooting.

## Integration Surface (Expected Files)

- `crates/ckrv-sandbox/src/agent/mod.rs`
- `crates/ckrv-sandbox/src/agent/opencode.rs` (new)
- `crates/ckrv-sandbox/src/agent/tests.rs`
- `crates/ckrv-cli/src/services/agent_lookup.rs`
- `crates/ckrv-sandbox/src/docker.rs` (if mounts required)
- `crates/docs/agent-guide.md`
- `crates/docs/cli-commands.md` (if flags/examples need update)

## Open Questions

- [ ] What is the canonical non-interactive Opencode invocation for deterministic runs?
- [ ] Does Opencode require token env vars, config files, or both?
- [ ] Where are Opencode credentials stored by default (path and format)?
- [ ] Do we need a dedicated Dockerfile (`Dockerfile.opencode`) or can we extend existing agent image?
- [ ] How should streaming/event output be normalized to existing execution logs?

## Success Criteria

| Metric | Target |
|--------|--------|
| Build health | `cargo build --workspace` passes |
| Provider registration | `opencode` selectable in agent config + lookup |
| Sandbox execution | test task can run using Opencode provider |
| Docs completeness | setup + troubleshooting documented |
| Parity | behavior consistent with existing provider UX |

## Next Steps

- [ ] Confirm Opencode CLI command contract and auth model
- [ ] Implement `OpencodeProvider`
- [ ] Wire enum/factory/lookup
- [ ] Add tests
- [ ] Document usage and limits
- [ ] Run smoke validation in local sandbox

## References

- Issue: https://github.com/FnSK4R17s/chakravarti-cli/issues/35
- Vision: `guiding_docs/vision.md`
- Similar brainstorm patterns: `brainstorming/kilo-code-agent/notes.md`
