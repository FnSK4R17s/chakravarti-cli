# Add Factory Droid integration

**Issue**: [#36](https://github.com/FnSK4R17s/chakravarti-cli/issues/36)
**Created**: 2026-02-24
**Status**: Draft | In Progress | Ready for Spec | Archived

## Problem Statement

Chakravarti-cli currently supports Claude Code, OpenAI Codex, and Kilo Code as agent providers. The project needs to integrate with Factory Droid to expand agent options for users who prefer Factory's agentic coding capabilities.

## Current State

Chakravarti-cli supports three agent types via the `AgentProvider` trait:
- Claude Code (Anthropic)
- OpenAI Codex
- Kilo Code (multi-provider)

Each agent is implemented as a separate module in `crates/ckrv-sandbox/src/agent/` and registered in the `AgentType` enum.

## Proposed Solution

Add Factory Droid as a new agent provider by:
1. Creating a new `FactoryDroidProvider` struct implementing the `AgentProvider` trait
2. Adding `FactoryDroid` to the `AgentType` enum
3. Adding parsing support in `AgentType::from_str()`
4. Adding the module to `mod.rs`

## User Stories

### US1: Factory agent execution
**As a** user with a Factory subscription,
**I want** to use Factory Droid as my execution agent,
**So that** I can leverage Factory's agentic coding capabilities within Chakravarti-cli.

### US2: Unified agent configuration
**As a** user,
**I want** to configure Factory Droid alongside other agents in `agents.yaml`,
**So that** I can route tasks to Factory based on skill level.

## Technical Approach

### Options Considered

| Option | Pros | Cons |
|--------|------|------|
| Add as new AgentProvider (like kilo.rs) | Follows existing pattern, clean separation | Requires CLI support confirmation |
| Use via OpenRouter | Reuses existing infrastructure | Loses Factory-specific features |
| API-based integration | No CLI dependency | More complex, may not match Factory's model |

### Decision

Add as a new `AgentProvider` implementation. Based on Factory's website, they provide CLI access ("Droids at scale" - "Script and parallelize Droids at massive scale for CI/CD"). This follows the same pattern as Claude, Codex, and Kilo.

## Implementation Notes

Based on the existing agent pattern (see `claude.rs`):
- Create `crates/ckrv-sandbox/src/agent/factory.rs`
- Implement `AgentProvider` trait with:
  - `name()` - "Factory Droid"
  - `agent_type()` - `AgentType::FactoryDroid`
  - `build_command()` - Construct `droid` CLI command
  - `required_env_vars()` - Factory API key if needed
  - `config_mounts()` - Mount `~/.factory` config directory
  - `parse_output()` - Parse Factory output format

Need to verify:
- Factory CLI command name (`droid`? `factory`?)
- Authentication method (API key, config file)
- Available CLI flags for non-interactive execution

## Open Questions

- [ ] What is the exact CLI command for Factory Droid?
- [ ] Does Factory support non-interactive/print mode execution?
- [ ] What environment variables or config files are needed?
- [ ] What is the output format for parsing?

## Success Criteria

| Metric | Target |
|--------|--------|
| Agent registered in AgentType | Yes |
| Agent selectable via --agent flag | Yes |
| Command builds successfully | Yes |
| Executes in Docker sandbox | Yes |

## Next Steps

- [ ] Research Factory CLI documentation
- [ ] Create factory.rs module
- [ ] Add to AgentType enum
- [ ] Test basic execution
- [ ] Update agent-guide.md documentation

## References

- Factory.ai: https://factory.ai/
- Agent Guide: crates/docs/agent-guide.md
- Existing agent implementations: crates/ckrv-sandbox/src/agent/
