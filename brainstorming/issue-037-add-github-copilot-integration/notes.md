# Add Github Copilot integration

**Issue**: [#37](https://github.com/FnSK4R17s/chakravarti-cli/issues/37)
**Created**: 2026-02-23
**Status**: In Progress

## Problem Statement

Chakravarti currently supports Claude Code, Codex, and GLM pathways, but not GitHub Copilot as an execution provider. Users who already rely on Copilot workflows cannot plug it into the same spec-driven orchestration loop.

## Current State

- Agent provider matrix omits GitHub Copilot.
- Teams with Copilot subscriptions must context-switch outside Chakravarti.
- No uniform telemetry and execution semantics for Copilot-backed runs.

## Proposed Solution

Add a GitHub Copilot provider integration that adheres to existing provider abstractions (agent trait, sandbox execution, config wiring, and CLI/UI surfacing). Start with a minimal viable path focused on parity with existing code-agent contracts.

## User Stories

### US1: Use Copilot as an orchestrated code agent
**As a** developer using Chakravarti,
**I want** run tasks through GitHub Copilot within the same orchestration flow,
**So that** I can keep one planning/execution interface regardless of model vendor.

## Technical Approach

### Options Considered

| Option | Pros | Cons |
|--------|------|------|
| Option A: Native Copilot CLI adapter | Better parity with existing CLI-based providers; simpler security model | Requires robust process contract mapping |
| Option B: API-level bridge | More direct control and metadata | Higher implementation and auth complexity |

### Decision

Choose Option A first: implement a CLI adapter for fastest delivery and lower architectural risk, then evaluate API bridge if needed for advanced capabilities.

## Implementation Notes

- Add provider enum/config entries for Copilot.
- Implement sandbox agent adapter with deterministic command invocation and output parsing.
- Reuse existing verification/promotion pipeline to avoid special casing.

## Open Questions

- [ ] What is the stable non-interactive Copilot invocation contract across OSes?
- [ ] Do we require feature flagging while integration is experimental?

## Success Criteria

| Metric | Target |
|--------|--------|
| End-to-end task success rate with Copilot provider | >= 95% on smoke suite |

## Next Steps

- [ ] Validate provider interface mapping and config schema updates
- [ ] Break implementation into tasks and sequence by risk

## References

- https://github.com/FnSK4R17s/chakravarti-cli/issues/37
- crates/docs/agent-guide.md
