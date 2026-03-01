# Add Cursor CLI integration

**Issue**: [#32](https://github.com/FnSK4R17s/chakravarti-cli/issues/32)
**Created**: 2026-02-23
**Status**: In Progress

## Problem Statement

Chakravarti CLI currently supports launching and orchestrating multiple agent runtimes, but Cursor CLI is not integrated as a first-class provider in `ckrv term` / related execution flows. Users who prefer Cursor CLI must work around this manually, which breaks the “single CLI for multi-agent workflows” goal.

## Current State

- `ckrv term` supports agent types including Claude, Codex, GLM/OpenRouter variants, and Kilo Code.
- Agent configuration and spawn logic are centralized, but there is no Cursor-specific agent type/config block.
- Users cannot define Cursor CLI as a managed agent in `~/.config/chakravarti/agents.yaml` and run it through existing command paths.

Pain points:
- Inconsistent user experience compared to existing agent integrations.
- No unified role assignment/selection UX for Cursor.
- Harder migration for teams standardizing on Chakravarti CLI.

## Proposed Solution

Add Cursor CLI integration as a native agent type with parity to existing agents where possible:

1. Extend agent model/config to include `cursor` agent type.
2. Implement binary resolution and command construction for Cursor CLI in term spawn flow.
3. Support pass-through args and optional custom binary path.
4. Ensure listing/selection/help text include Cursor.
5. Add docs and acceptance coverage for configuration and expected behavior.

## User Stories

### US1: Configure Cursor as an agent
**As a** Chakravarti user,
**I want** to declare Cursor CLI in my agent config,
**So that** I can launch it from `ckrv term` like other agents.

### US2: Use Cursor from term with args
**As a** power user,
**I want** to pass extra flags/arguments through `ckrv term`,
**So that** I can preserve my existing Cursor CLI workflows.

### US3: Maintain consistent multi-agent UX
**As a** project maintainer,
**I want** Cursor to appear in listing, selection, and docs,
**So that** team members can discover and use it consistently.

## Technical Approach

### Options Considered

| Option | Pros | Cons |
|--------|------|------|
| Add Cursor as first-class `AgentType` variant | Strong typing, clean UX parity, future extensibility | Requires touching serialization + spawn logic + docs |
| Treat Cursor as generic custom binary only | Minimal code change | Poor discoverability, weak validation, inconsistent UX |

### Decision

Use a first-class `AgentType::Cursor` integration. This aligns with existing architecture and keeps user-facing behavior consistent with other built-in providers.

## Implementation Notes

- Update agent enum + serde handling for `cursor`.
- Extend command builder to resolve Cursor binary (respect `binary_path`, fallback to `cursor` in PATH).
- Reuse existing env var and extra args patterns.
- Ensure `ckrv term --list` labels Cursor clearly.
- Update docs/examples in CLI help/skill docs as needed.

## Open Questions

- [ ] Do we need Cursor-specific env vars to be auto-populated, or is binary-only launch sufficient for first release?
- [ ] Should Cursor integration be available in `ckrv run` executor flows now, or term-only initially?

## Success Criteria

| Metric | Target |
|--------|--------|
| Cursor agent can be configured and selected via `ckrv term` | 100% |
| Cursor launch works with passthrough args | 100% |
| Docs/examples include Cursor integration path | Complete |

## Next Steps

- [ ] Generate implementation tasks from this brainstorm.
- [ ] Implement enum/config + spawn command support.
- [ ] Add/update docs and usage examples.
- [ ] Validate via smoke run in local environment.

## References

- https://github.com/FnSK4R17s/chakravarti-cli/issues/32
- `crates/ckrv-cli/src/commands/term.rs`
- `crates/ckrv-cli/src/services/agent_lookup.rs`
