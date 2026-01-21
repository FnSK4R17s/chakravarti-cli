# Agent Configuration Contract

**Feature**: 011-openai-codex-agent
**Type**: Internal API (Rust traits and types)

## AgentProvider Trait

```rust
/// Trait defining the interface for AI agent implementations.
/// Each agent (Claude, Codex, etc.) implements this trait.
pub trait AgentProvider: Send + Sync {
    /// Human-readable name for logging and UI display
    fn name(&self) -> &str;
    
    /// Construct the CLI command for agent execution
    /// Returns: [command, arg1, arg2, ...]
    fn build_command(&self, prompt: &str, workdir: &Path) -> Vec<String>;
    
    /// Environment variables required by this agent
    /// e.g., ["ANTHROPIC_API_KEY"] or ["OPENAI_API_KEY"]
    fn required_env_vars(&self) -> Vec<&str>;
    
    /// Docker mounts for agent-specific config files
    /// e.g., ~/.claude.json or ~/.codex/config.json
    fn config_mounts(&self) -> Vec<Mount>;
    
    /// Parse agent output into normalized format
    fn parse_output(&self, output: &str) -> Result<AgentOutput>;
    
    /// Optional: streaming output parser
    fn parse_streaming_line(&self, line: &str) -> Option<StreamEvent>;
}
```

## Configuration Schema

### agents.yaml

```yaml
# Location: ~/.config/chakravarti/agents.yaml
# or project-local: .chakravarti/agents.yaml

default: claude  # or "codex"

agents:
  claude:
    enabled: true
    model: claude-sonnet-4-20250514  # optional
    
  codex:
    enabled: true
    model: gpt-5.2-codex  # optional
```

### Environment Variables

| Variable | Agent | Required | Description |
|----------|-------|----------|-------------|
| `ANTHROPIC_API_KEY` | claude | Yes (if using claude) | Anthropic API key |
| `OPENAI_API_KEY` | codex | Yes (if using codex) | OpenAI API key |
| `CKRV_DEFAULT_AGENT` | any | No | Override default agent |
| `ANTHROPIC_DEFAULT_SONNET_MODEL` | claude | No | Model override |
| `OPENAI_MODEL` | codex | No | Model override |

## CLI Interface

### run command

```bash
ckrv run [OPTIONS]

Options:
  --agent <AGENT>    Agent to use: claude, codex [default: from config]
  --model <MODEL>    Model override for the selected agent
  --dry-run          Simulate execution without running agent
```

### Examples

```bash
# Use default agent (claude)
ckrv run

# Use codex for this execution
ckrv run --agent=codex

# Use specific codex model
ckrv run --agent=codex --model=gpt-4o

# Use codex from environment
CKRV_DEFAULT_AGENT=codex ckrv run
```
