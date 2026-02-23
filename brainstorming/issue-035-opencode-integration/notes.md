# Add Opencode Integration

**Issue**: [#35](https://github.com/FnSK4R17s/chakravarti-cli/issues/35)
**Created**: 2026-02-23
**Status**: Draft

## Problem Statement

Chakravarti-cli orchestrates multiple AI agents (Claude Code, Codex, Kilo) but lacks support for OpenCode, a popular open-source AI coding agent with 50k+ GitHub stars. Users want to add OpenCode to their agent pool for multi-provider orchestration.

## Current State

- Chakravarti supports: Claude, Codex, KiloCode
- Missing: OpenCode
- Agent integration pattern is well-established via `AgentProvider` trait

## Proposed Solution

Add OpenCode as a new agent type following the existing pattern:
1. Add `OpenCode` variant to `AgentType` enum
2. Create `OpenCodeProvider` implementing `AgentProvider` trait
3. Add to factory function
4. Document in agent guide

## User Stories

### US1: OpenCode as Execution Agent
**As a** user with an OpenCode subscription,
**I want** to use OpenCode as an implementation agent,
**So that** I can leverage its 75+ LLM providers in my orchestration workflow.

### US2: Headless Execution
**As a** Chakravarti user,
**I want** OpenCode to run in headless/non-interactive mode,
**So that** it works with the spec-driven "fire and forget" model.

## Technical Approach

### OpenCode CLI Analysis

From opencode.ai/docs:
```bash
# Basic non-interactive execution
opencode run "prompt here"

# With specific model
opencode run "prompt" --model provider/model

# With agent
opencode run "prompt" --agent agent-name
```

### Options Considered

| Option | Pros | Cons |
|--------|------|------|
| OpenCode CLI (headless) | Native CLI, no TUI required | Need to handle session/daemon mode |
| OpenCode MCP | Standard integration | MCP not designed for headless batch |
| Crush (new name) | Active development | Different CLI, needs investigation |

### Decision

Use OpenCode CLI in headless mode. The `opencode run` command accepts prompts directly without requiring TUI. This matches the existing pattern used by Claude (`--print`) and Codex (`--print`).

**Note**: The original opencode-ai/opencode was archived and moved to "Crush". The current opencode.ai is a new Go-based version by Charm. We should support the current CLI.

## Implementation Notes

### Key Files to Modify

1. `crates/ckrv-sandbox/src/agent/mod.rs` - Add `AgentType::OpenCode` variant
2. `crates/ckrv-sandbox/src/agent/opencode.rs` - New file for OpenCodeProvider
3. `crates/ckrv-sandbox/src/agent/mod.rs` - Add to `create_agent()` factory
4. `crates/docs/agent-guide.md` - Document the new agent

### OpenCodeProvider Implementation

```rust
// Based on existing patterns in claude.rs, codex.rs
pub struct OpenCodeProvider;

impl AgentProvider for OpenCodeProvider {
    fn name(&self) -> &str { "OpenCode" }
    fn agent_type(&self) -> AgentType { AgentType::OpenCode }
    
    fn build_command(&self, prompt: &str, workdir: &Path, config: &AgentConfig) -> Vec<String> {
        // opencode run "prompt" --project /path/to/workdir
        vec![
            "opencode".to_string(),
            "run".to_string(),
            prompt.to_string(),
            "--project".to_string(),
            workdir.to_string_lossy().to_string(),
        ]
    }
}
```

### Configuration (agents.yaml)

```yaml
agents:
  - id: opencode-default
    name: OpenCode Default
    agent_type: opencode
    level: 4
    is_default: false
    enabled: true
```

## Open Questions

- [ ] Does OpenCode require a running daemon/server for CLI to work?
- [ ] What is the exit code behavior on success/failure?
- [ ] How does JSON output work for parsing?
- [ ] Should we support the "Crush" rebrand or stick with "opencode"?

## Success Criteria

| Metric | Target |
|--------|--------|
| Agent type added to enum | Yes |
| Provider implementation | Complete |
| Headless execution works | Verified |
| Integration tests pass | Yes |

## Next Steps

- [ ] Research OpenCode CLI headless behavior in detail
- [ ] Implement OpenCodeProvider
- [ ] Add tests
- [ ] Update documentation

## References

- [opencode.ai](https://opencode.ai)
- [OpenCode CLI Docs](https://opencode.ai/docs/cli/)
- [GitHub: opencode-ai/opencode](https://github.com/opencode-ai/opencode) (archived)
- [Crush (successor)](https://github.com/charmbracelet/crush)
- Existing agent implementations: `crates/ckrv-sandbox/src/agent/{claude,codex,kilo}.rs`
