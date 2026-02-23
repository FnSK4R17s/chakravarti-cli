# Add Opencode integration

**Issue**: [#35](https://github.com/FnSK4R17s/chakravarti-cli/issues/35)
**Created**: 2026-02-23
**Status**: In Progress

## Problem Statement

Chakravarti currently supports Claude, Codex, and Kilo agent workflows, but has no first-class Opencode integration path. Users who want to run Opencode-based execution need manual setup and inconsistent conventions.

## Current State

- Agent modes are represented in workspace conventions (`.agent`, `.claude`, `.opencode` links/folders), but issue-driven workflows are primarily documented around existing agent integrations.
- No clear end-to-end guidance or CLI behavior definition for selecting Opencode in issue/spec planning and execution loops.
- This creates confusion for contributors and blocks consistent multi-agent experimentation.

## Proposed Solution

Add explicit Opencode integration support across planning and execution workflow surfaces so issue-to-implementation flow is symmetrical with existing agents:

1. Ensure Opencode is a supported agent option where agent selection is exposed.
2. Standardize Opencode-specific setup/validation expectations in docs/skills.
3. Verify workflow parity for planning + run commands and resulting artifacts.

## User Stories

### US1: Use Opencode as a first-class execution agent
**As a** contributor,
**I want** to select Opencode in the same places I select Claude/Codex/Kilo,
**So that** I can run issue planning and implementation without custom workarounds.

### US2: Discover setup quickly
**As a** new maintainer,
**I want** clear setup docs and validation steps,
**So that** I can confirm Opencode integration works before running tasks.

### US3: Preserve workflow consistency
**As a** project owner,
**I want** generated planning artifacts and task execution behavior to remain consistent across agents,
**So that** review and CI expectations stay predictable.

## Technical Approach

### Options Considered

| Option | Pros | Cons |
|--------|------|------|
| Add minimal doc-only mention of Opencode | Fastest initial change | No guardrails, likely drift from actual CLI/runtime behavior |
| Add full agent parity (selection, docs, and validation checks) | Predictable user experience, lower onboarding friction | Larger initial implementation scope |

### Decision

Pursue full agent parity in small increments: first ensure exposed agent selection includes Opencode everywhere needed, then align docs/skills and validate end-to-end flow.

## Implementation Notes

- Reuse existing agent abstraction points rather than introducing Opencode-specific branching where possible.
- Confirm naming consistency (`opencode`) across CLI flags, docs, and generated examples.
- Keep Docker/runtime assumptions explicit where Opencode behavior differs from current agents.
- Preserve backward compatibility for existing workflows and defaults.

## Open Questions

- [ ] Should Opencode be enabled by default in all environments or behind capability detection?
- [ ] Are there model/provider constraints unique to Opencode that require extra configuration UX?
- [ ] Do we need additional telemetry/log annotations to distinguish Opencode runs in debugging output?

## Success Criteria

| Metric | Target |
|--------|--------|
| Agent selection parity | Opencode is available in all relevant CLI/workflow selection points |
| Docs completeness | Setup + usage documented with at least one end-to-end example |
| Workflow reliability | Planning/execution on an Opencode path completes without manual file surgery |

## Next Steps

- [ ] Convert this brainstorm into implementation tasks (`tasks.md`)
- [ ] Implement agent selection + docs updates
- [ ] Validate with a sample issue workflow and capture outcomes

## References

- https://github.com/FnSK4R17s/chakravarti-cli/issues/35
- `.agents/skills/chakravarti-cli/SKILL.md`

## Prompt 1 Summary

Prompt executed: "Please brainstorm on GitHub issue #35. Then summarize what you changed."

Changes made:
- Created `brainstorming/issue-035-opencode-integration/notes.md`
- Added problem framing, solution direction, user stories, options/decision, open questions, and measurable success criteria for issue #35.
