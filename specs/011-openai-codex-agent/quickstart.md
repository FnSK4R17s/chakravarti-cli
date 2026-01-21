# Quickstart: OpenAI Codex CLI Agent

## Prerequisites

1. **OpenAI API Key**: Get from [platform.openai.com](https://platform.openai.com)
2. **Codex CLI**: Installed automatically in Docker sandbox

## Setup

### 1. Set API Key

```bash
export OPENAI_API_KEY="sk-..."
```

### 2. Configure Default Agent (Optional)

Create or edit `~/.config/chakravarti/agents.yaml`:

```yaml
default: codex
```

Or use environment variable:

```bash
export CKRV_DEFAULT_AGENT=codex
```

## Usage

### Per-Execution Agent Selection

```bash
# Use Codex for a specific run
ckrv run --agent=codex

# Compare with Claude
ckrv run --agent=claude
```

### Model Selection

```bash
# Use specific Codex model
ckrv run --agent=codex --model=gpt-5.2-codex

# Use reasoning-heavy model
ckrv run --agent=codex --model=gpt-4o
```

## Verification

Check which agent is configured:

```bash
ckrv config show
# Output:
# default_agent: codex
# configured_agents:
#   - claude (api_key: configured)
#   - codex (api_key: configured)
```

## Troubleshooting

### "OPENAI_API_KEY not set"

```bash
export OPENAI_API_KEY="your-key-here"
# or add to ~/.bashrc / ~/.zshrc
```

### "Codex CLI not found in Docker"

Rebuild the Docker image:

```bash
make docker-build
```

### Agent Not Responding

Check logs:

```bash
ckrv run --agent=codex --verbose
```
