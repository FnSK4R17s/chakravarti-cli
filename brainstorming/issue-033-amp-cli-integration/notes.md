# Add AMP CLI Integration

**Issue**: [#33](https://github.com/FnSK4R17s/chakravarti-cli/issues/33)
**Created**: 2026-02-23
**Status**: Draft

## Problem Statement

Chakravarti CLI needs to support AMP (ampcode.com) as an additional AI coding agent provider. AMP is a frontier coding agent that can be installed via `curl -fsSL https://ampcode.com/install.sh | bash` and used in the terminal.

## Current State

Chakravarti currently supports the following agents via the `AgentProvider` trait:
- Claude (Anthropic)
- Claude via OpenRouter
- Claude via GLM
- Codex (OpenAI)
- Kilo Code

## Proposed Solution

Implement an `AmpProvider` that conforms to the `AgentProvider` trait, similar to existing providers. AMP CLI is installed via shell script and used as a terminal command.

## User Stories

### US1: Use AMP as execution agent
**As a** user,
**I want** to select AMP as my coding agent,
**So that** I can leverage AMP's frontier coding capabilities for task execution.

## Technical Approach

### Options Considered

| Option | Pros | Cons |
|--------|------|------|
| Direct CLI invocation | Simple implementation | Less control over execution |
| Custom provider wrapper | Full control over behavior | More complex |
| Docker-based execution | Consistent environment | Requires Docker image |

### Decision

Follow the existing pattern used by Claude and Codex providers:
1. Add `Amp` to the `AgentType` enum
2. Create `AmpProvider` struct implementing `AgentProvider` trait
3. Use Docker-based execution for consistency (or direct CLI if simpler)

### Implementation

1. **Update AgentType enum** in `ckrv-sandbox/src/agent/mod.rs`
2. **Create AmpProvider** in `ckrv-sandbox/src/agent/amp.rs`
3. **Add to provider factory** in `create_agent()` function
4. **Update CLI** to support `--agent amp`
5. **Update UI** to include amp in agent dropdown

### Key Implementation Files

| File | Changes |
|------|---------|
| `crates/ckrv-sandbox/src/agent/mod.rs` | Add `Amp` to `AgentType` enum |
| `crates/ckrv-sandbox/src/agent/amp.rs` | New file: `AmpProvider` implementation |
| `crates/ckrv-cli/src/commands/run.rs` | Add "amp" to agent choices |
| `crates/ckrv-ui/frontend/src/types/api.generated.ts` | Add "amp" to `AgentType` |
| `crates/ckrv-ui/frontend/src/components/AgentManager.tsx` | Add amp to dropdown |

### AgentProvider Trait Requirements

```rust
pub trait AgentProvider: Send + Sync {
    fn agent_type(&self) -> AgentType;
    fn name(&self) -> &str;
    async fn execute(&self, config: &AgentConfig) -> Result<AgentOutput, AgentError>;
    fn config_mounts(&self) -> Vec<Mount>;
}
```

### AMP CLI Installation

```bash
curl -fsSL https://ampcode.com/install.sh | bash
```

## Open Questions

- [ ] Does AMP CLI support the same task-based execution model?
- [ ] What is the exact command-line interface for AMP?
- [ ] Should we create a Docker image or use direct CLI?
- [ ] How does AMP handle authentication/API keys?

## Success Criteria

| Metric | Target |
|--------|--------|
| Agent selectable in CLI | `--agent amp` works |
| Agent available in UI | Dropdown includes amp |
| Agent executes tasks | Can run a simple spec |

## Next Steps

- [ ] Research AMP CLI command-line interface
- [ ] Create AmpProvider implementation
- [ ] Add integration tests
- [ ] Update documentation

## References

- AMP Website: https://ampcode.com/
- AMP Owner's Manual: https://ampcode.com/manual
- Agent Guide: crates/docs/agent-guide.md
