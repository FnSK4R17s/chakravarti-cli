# Add Factory Droid integration

**Issue**: [#36](https://github.com/FnSK4R17s/chakravarti-cli/issues/36)
**Created**: 2026-02-23
**Status**: In Progress

## Problem Statement

`ckrv` already supports multiple agent providers (Claude Code, Codex, Gemini/OpenCode paths), but it does not support Factory Droids yet. Users with Factory subscriptions cannot route execution through Factory while keeping the same spec-first orchestration flow.

This misses a core value proposition from the vision doc: interoperability across agent ecosystems and avoiding vendor lock-in.

## Current State

- Issue #36 currently contains only the Factory homepage link (`https://factory.ai/`) and no technical acceptance criteria.
- Existing agent integrations in `ckrv` follow a provider pattern and are wired into:
  - config parsing / agent listing
  - command construction for execution
  - docs and UI visibility
- There is no known Factory-specific provider, transport, or auth flow in the current codebase.

## Proposed Solution

Add a **Factory Droid provider integration** with a minimal-but-complete first version:

1. Add Factory as a first-class provider in agent config.
2. Implement command execution adapter for Factory Droid tasks.
3. Support model/agent selection and required auth configuration.
4. Surface Factory in CLI/UI agent lists.
5. Document setup and troubleshooting.

## User Stories

### US1: Configure Factory provider
**As a** `ckrv` user with Factory access,
**I want** to configure a Factory Droid agent in my agents config,
**So that** I can assign tasks to Factory from within `ckrv`.

### US2: Run orchestration with mixed providers
**As a** user with multiple AI tools,
**I want** to run plans where some tasks use Factory and others use existing providers,
**So that** I can optimize cost/speed/quality per task batch.

### US3: Debug failures quickly
**As a** user,
**I want** clear Factory-specific error messages (auth missing, CLI not found, unsupported mode),
**So that** I can fix setup issues without digging through internals.

## Technical Approach

### Option A — Native CLI wrapper (recommended)
Implement Factory integration similarly to existing CLI-based providers by mapping `ckrv` task payloads to Factory CLI invocations.

**Pros**
- Fits current architecture and mental model
- Fastest path to deliver
- Easy to keep isolated per worktree/container

**Cons**
- Depends on Factory CLI stability and availability in runtime
- May require prompt/IO contract normalization

### Option B — HTTP/API transport provider
Use Factory APIs directly from Rust (if available and stable).

**Pros**
- Better programmatic control and richer telemetry potential
- Less dependence on shell command formatting

**Cons**
- Higher implementation complexity and maintenance burden
- Unknown API maturity and auth workflow details

### Decision

**Start with Option A (CLI wrapper)** for issue #36 scope. Keep provider boundary clean so API transport can be added later if needed.

## Implementation Notes

- Keep integration at orchestration layer (non-goal: becoming a coding agent itself).
- Reuse existing provider abstraction traits/modules instead of one-off branching.
- Add capability flags where Factory behavior differs (streaming, interactive mode, tool-use constraints).
- Ensure errors include actionable setup hints.

Potential integration surface (to verify during implementation):
- `crates/ckrv-sandbox/src/agent/*` (provider wiring)
- config schema / defaults where providers are declared
- UI/API agent listing endpoints if provider catalog is surfaced
- docs: agent setup + examples

## Open Questions

- [ ] What is the exact Factory CLI binary name and invocation contract for non-interactive execution?
- [ ] Which auth mechanism is expected in local and Docker runtime (env var, login session, token file)?
- [ ] Does Factory support fully headless execution and deterministic output capture suitable for `ckrv` pipelines?
- [ ] Are there rate-limit or concurrency constraints that require scheduler changes?
- [ ] Does Factory expose model/droid selection that should map to existing `model` fields?

## Success Criteria

| Metric | Target |
|--------|--------|
| Provider availability | Factory appears in supported provider list and config docs |
| Basic execution | A sample task can run through Factory provider end-to-end |
| Error quality | Common setup failures return actionable messages |
| Parity | Mixed-provider plans still execute without regression |

## Next Steps

- [x] Create issue-linked brainstorming notes and task breakdown
- [ ] Validate Factory CLI/auth details and finalize command contract
- [ ] Draft implementation spec or directly execute via tasks (depending on scope confidence)

## References

- Issue: https://github.com/FnSK4R17s/chakravarti-cli/issues/36
- Product: https://factory.ai/
- Vision alignment: `guiding_docs/vision.md`
