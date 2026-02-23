# Add GitHub Copilot integration

**Issue**: [#37](https://github.com/FnSK4R17s/chakravarti-cli/issues/37)
**Created**: 2026-02-23
**Status**: In Progress

## Problem Statement

`ckrv` currently supports Claude Code, Codex, and Kilo Code pathways, but does not expose GitHub Copilot as a first-class agent option. Users who pay for Copilot and want to route selected work through GitHub’s stack cannot do so through the same orchestration flow.

This creates two gaps:
1. **Subscription under-utilization** for users with Copilot access.
2. **Less provider diversity** in the orchestration layer, despite the product vision being cross-provider coordination.

## Current State

- Agent abstractions exist in `ckrv-sandbox` via `AgentProvider` and `AgentType`.
- Existing providers include Claude, Codex, and Kilo Code.
- Runtime execution relies on Dockerized CLI invocation patterns and provider-specific mounts/env vars.
- Documentation already includes an “add new agent provider” workflow (`crates/docs/agent-guide.md`).

Pain points for Copilot integration:
- Copilot has multiple possible integration surfaces (CLI, API, or GitHub CLI extension paths), each with different auth/runtime constraints.
- No validated reference implementation currently exists in-repo for Copilot-specific auth/config mounts.

## Proposed Solution

Add a **GitHub Copilot agent provider** following existing provider architecture, with a phased rollout:

1. **Phase A (Architecture + UX readiness)**
   - Add `AgentType::Copilot` and config surface in CLI/UI.
   - Add provider scaffolding with capability/availability checks.
   - Add clear errors/help text when Copilot credentials or binaries are unavailable.

2. **Phase B (Execution path)**
   - Implement deterministic command builder for Copilot backend.
   - Add Docker mount/env strategy for Copilot credentials.
   - Validate non-interactive execution parity with other providers.

3. **Phase C (Hardening + docs)**
   - Add integration tests (command generation + mount behavior + fallback handling).
   - Update docs for setup, troubleshooting, and model selection.

## User Stories

### US1: Select Copilot as an Agent
**As a** developer with Copilot access,
**I want** to configure Copilot as an agent in `ckrv`,
**So that** I can run tasks through GitHub’s coding assistant stack.

### US2: Keep Existing Orchestration UX
**As a** `ckrv` user,
**I want** Copilot to behave like existing agents (configurable/default/testable),
**So that** I do not need a separate execution workflow.

### US3: Clear Failure Modes
**As a** developer,
**I want** actionable setup errors when Copilot prerequisites are missing,
**So that** I can self-remediate quickly.

## Technical Approach

### Options Considered

| Option | Pros | Cons |
|--------|------|------|
| A) Native Copilot provider in `ckrv-sandbox` | Consistent architecture; first-class UX; best long-term parity | Requires solving Copilot CLI/auth/runtime details in Docker |
| B) Route Copilot via Kilo/OpenRouter style indirection | Faster to prototype; less direct implementation effort | Not true “Copilot integration”; weaker UX clarity and branding |
| C) Defer to plugin architecture first, then Copilot | Future-proof for many providers | Delays issue #37 and user value |

### Decision

**Choose Option A** (native provider) with a phased implementation and explicit fallback behavior.

Rationale:
- Aligns with project vision: orchestration across real providers users already pay for.
- Preserves the current mental model (`AgentType` + provider-specific command/mount abstraction).
- Keeps issue #37 scoped to direct user value while allowing refactor later if a plugin architecture emerges.

## Implementation Notes

- Extend `AgentType` parsing/display to include `copilot` aliases.
- Add `CopilotProvider` implementing `AgentProvider`:
  - `name()`, `agent_type()`, `command()`, `args()`, `env_vars()`, `config_mounts()`.
- Add provider wiring in `create_agent(...)` and defaults/selection logic where relevant.
- Update config schemas and validation paths in CLI/UI agent management.
- Ensure docker execution supports credential mount strategy without leaking secrets.
- Include robust “not installed / not authenticated” diagnostics.

## Open Questions

- [ ] Which Copilot execution interface is canonical for v1 (GitHub CLI extension vs dedicated CLI/API wrapper)?
- [ ] What credential file/env strategy is reliable across Linux/macOS/Windows hosts?
- [ ] Do we need per-task model overrides for Copilot, or only provider-level selection initially?
- [ ] Should Copilot support enter as experimental/beta flag in first release?

## Success Criteria

| Metric | Target |
|--------|--------|
| New provider appears in agent list/config | ✅ |
| `ckrv term --agent copilot` style flow resolves command path | ✅ |
| Missing prerequisites produce actionable errors | ✅ |
| Existing Claude/Codex/Kilo behavior remains unchanged | ✅ (no regressions) |
| Docs include setup + troubleshooting for Copilot | ✅ |

## Next Steps

- [x] Create issue-linked brainstorming notes for #37
- [x] Generate implementation task breakdown (`tasks.md`)
- [ ] Validate canonical Copilot runtime/auth approach with a short technical spike
- [ ] Move status to “Ready for Spec” once execution interface choice is finalized

## References

- GitHub Issue #37: https://github.com/FnSK4R17s/chakravarti-cli/issues/37
- Agent provider architecture: `crates/docs/agent-guide.md`
- Project vision constraints: `guiding_docs/vision.md`
