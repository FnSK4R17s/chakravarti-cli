# Add Factory Droid integration

**Issue**: [#36](https://github.com/FnSK4R17s/chakravarti-cli/issues/36)
**Created**: 2026-02-23
**Status**: Draft

## Problem Statement

Chakravarti-cli orchestrates multiple AI coding agents (Claude Code, Codex, Kilo). Factory AI offers "Droids" - agent-native software development tools that work across IDE, CLI, Web, Slack, and Linear. Adding Factory Droid as an agent provider increases the flexibility and capability of the orchestration engine.

## Current State

Chakravarti currently supports three agent providers:
- Claude Code (Anthropic)
- OpenAI Codex
- Kilo Code (multi-provider)

Each agent implements the `AgentProvider` trait in `crates/ckrv-sandbox/src/agent/`.

## Proposed Solution

Implement a new `FactoryDroidProvider` that integrates the Factory Droid CLI (`droid`) as an agent provider. The integration will use `droid exec` for headless/non-interactive execution, which is suitable for orchestration.

## User Stories

### US1: Add Factory Droid to agent pool
**As a** user with a Factory API key,
**I want** to use Factory Droids as an execution agent,
**So that** I can leverage Factory's agent capabilities within Chakravarti's orchestration.

### US2: Configure Factory model preference
**As a** user,
**I want** to specify which model Factory uses,
**So that** I can optimize for cost or capability.

## Technical Approach

### Options Considered

| Option | Pros | Cons | Decision |
|--------|------|------|----------|
| Interactive `droid` mode | Full capabilities | Requires TTY, not suitable for orchestration | No |
| Headless `droid exec` | Designed for automation, CI/CD | Limited to single-shot tasks | **Yes** |
| Factory API directly | Full control | Replicates what CLI already does, more work | No |

### Decision

Use `droid exec` (headless mode) - it's the natural fit for Chakravarti's orchestration layer. This aligns with the existing pattern of other agent integrations.

### Implementation Plan

1. Add `Factory` to `AgentType` enum in `mod.rs`
2. Create new `factory.rs` implementing `AgentProvider` trait
3. Add to `create_agent()` match statement
4. Support config file at `~/.factory/config.json` (if exists)

## Implementation Notes

### Agent Configuration
- CLI command: `droid exec "prompt"`
- Environment variable: `FACTORY_API_KEY`
- Config mount: `~/.factory/` directory

### Model Selection
Factory supports multiple models. Consider adding model configuration similar to other providers:
- `FACTORY_MODEL` env var or config option
- Default to Factory's default if not specified

### Output Parsing
`droid exec` outputs to stdout/stderr. Need to parse for success/failure detection.

## Open Questions

- [ ] Does Factory support any special config files we need to mount?
- [ ] What models does Factory support? Should we expose model selection?
- [ ] How does Factory handle API authentication (just API key)?
- [ ] Is there rate limiting we should be aware of?

## Success Criteria

| Metric | Target |
|--------|--------|
| Agent type added to CLI | Yes |
| Executes prompts via `droid exec` | Yes |
| Streams output when enabled | Yes |
| Handles errors gracefully | Yes |

## Next Steps

- [ ] Create factory.rs provider implementation
- [ ] Add tests for command building and output parsing
- [ ] Document in agent guide

## References

- [Factory CLI Docs](https://docs.factory.ai/cli/getting-started/quickstart)
- [droid exec Overview](https://docs.factory.ai/cli/droid-exec/overview)
- [CLI Reference](https://docs.factory.ai/reference/cli-reference)
- Existing agent implementations: `crates/ckrv-sandbox/src/agent/{claude,codex,kilo}.rs`
