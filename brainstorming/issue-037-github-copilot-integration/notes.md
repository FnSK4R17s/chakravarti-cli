# Add Github Copilot integration

**Issue**: [#37](https://github.com/FnSK4R17s/chakravarti-cli/issues/37)
**Created**: 2026-02-23
**Status**: Draft | In Progress | Ready for Spec | Archived

## Problem Statement

<!-- What problem are we solving? Why does it matter? -->

Chakravarti currently supports Claude Code and OpenAI Codex as AI agents. Adding GitHub Copilot as an agent provider would give users more choice and leverage Microsoft's extensive code generation capabilities.

## Current State

<!-- How does it work today? What are the pain points? -->

- Chakravarti supports Claude Code (native), Claude Code + OpenRouter, Claude Code + GLM, and OpenAI Codex
- Agent integrations are implemented in `ckrv-sandbox/src/agent/`
- Each agent implements the `AgentProvider` trait

## Proposed Solution

<!-- High-level approach. What will we build? -->

Implement GitHub Copilot integration following the existing agent provider pattern:
1. Add Copilot agent provider in `ckrv-sandbox/src/agent/`
2. Support both Copilot CLI and Copilot API (if available)
3. Configure via `agents.yaml`

## User Stories

### US1: Use Copilot as Agent
**As a** developer,
**I want** to use GitHub Copilot as the AI agent for Chakravarti tasks,
**So that** I can leverage Microsoft's code generation alongside my existing tools.

### US2: Configure Copilot Provider
**As a** user,
**I want** to configure Copilot in agents.yaml,
**So that** I can specify Copilot with appropriate model options.

## Technical Approach

### Options Considered

| Option | Pros | Cons |
|--------|------|------|
| Copilot CLI (github/cli) | Native CLI, familiar UX | Limited compared to API |
| Copilot API | Full capabilities | May require paid subscription |
| Both (CLI fallback to API) | Most compatible | More complex implementation |

### Decision

<!-- Which option and why? -->

Start with Copilot CLI integration (github/cli ext), as it's the simplest path to working implementation and matches the "good first issue" label. Can add API support later.

## Implementation Notes

- Follow existing agent pattern in `ckrv-sandbox/src/agent/`
- Agent name: "copilot"
- Need to check if gh CLI has extension for Copilot or use direct API
- Consider auth flow (GitHub authentication already required)

## Open Questions

- [ ] Does gh CLI have Copilot commands? Or need to use Copilot API directly?
- [ ] What models does Copilot support? (codex, gpt-4o, etc.)
- [ ] Is there a streaming mode for interactive agent tasks?
- [ ] What are the rate limits and pricing?

## Success Criteria

| Metric | Target |
|--------|--------|
| Agent works with `ckrv run --agent copilot` | Yes |
| Supports task execution | Yes |
| Streams output | Yes |

## Next Steps

- [ ] Research gh CLI Copilot commands or API
- [ ] Create agent provider struct
- [ ] Implement AgentProvider trait
- [ ] Add configuration support
- [ ] Test with sample task

## References

- [GitHub Copilot CLI](https://docs.github.com/en/copilot/github-copilot-cli)
- [Existing agent implementations](crates/ckrv-sandbox/src/agent/)
