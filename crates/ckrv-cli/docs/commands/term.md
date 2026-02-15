---
command: term
generated_from: crates/ckrv-cli/src/lib.rs
last_commit: 508766e
---

# ckrv term

Spawn an interactive AI agent terminal

## Description

Spawn an interactive AI agent terminal session.

Quickly launch any configured agent (Claude, OpenRouter, Z.AI, Codex) with the correct environment variables automatically configured.

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

## Examples

```bash
# Interactive selection with options prompt
ckrv term

# Launch specific agent (skips agent selection)
ckrv term --agent my-openrouter-agent

# Pass flags directly (scripting)
ckrv term -- --dangerously-skip-permissions --continue

# List available agents
ckrv term --list
```
