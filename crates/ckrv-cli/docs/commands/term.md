---
command: term
generated_from: crates/ckrv-cli/src/lib.rs
last_commit: 508766e
---

# ckrv term

Spawn an interactive AI agent terminal

## Description

Spawn an interactive AI agent terminal session.

Quickly launch any configured agent (Claude, OpenRouter, Z.AI, Codex, Cursor) with the correct environment variables automatically configured.

Without arguments, presents an interactive selection menu with options for common flags. Use -- to pass arguments directly for scripting.

## Arguments

| Argument | Required | Description |
|----------|----------|-------------|
| `passthrough_args` | No | Additional arguments to pass to the agent binary |

## Options

| Flag | Description |
|------|-------------|
| `--agent`, `-a` | Agent ID to spawn directly (skips interactive agent selection) |
| `--list`, `-l` | List available agents and exit |

## Cursor agent config example

```yaml
agents:
  - id: cursor-default
    name: Cursor CLI
    agent_type: cursor
    enabled: true
    is_default: false
    binary_path: /usr/local/bin/cursor # optional, defaults to "cursor" from PATH
    extra_args:
      - --model
      - gpt-5
```

## Examples

```bash
# Interactive selection with options prompt
ckrv term

# Launch specific agent (skips agent selection)
ckrv term --agent my-openrouter-agent

# Launch cursor agent with passthrough args
ckrv term --agent cursor-default -- --print "Summarize this repository"

# List available agents
ckrv term --list
```
