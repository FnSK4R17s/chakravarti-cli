---
command: term
generated_from: lib.rs
last_commit: f92f604
---

# ckrv term

Spawn an interactive AI agent terminal.

## Description

Spawn an interactive AI agent terminal session.

Quickly launch any configured agent (Claude, OpenRouter, Z.AI, Codex, Kilo Code) with the correct environment variables automatically configured.

Without arguments, presents an interactive selection menu with options for common flags. Use `--` to pass arguments directly for scripting.

## Options

| Flag | Description |
|------|-------------|
| `--agent`, `-a` | Agent ID to spawn directly (skips interactive agent selection) |
| `--list`, `-l` | List available agents and exit |
| `--worktree` | Run agent in an isolated git worktree |
| `--sandbox` | Run agent in a Docker sandbox container |
| `--name <NAME>` | Name for this session (enables resume with --resume) |
| `--resume [NAME]` | Resume a session. Optionally pass a session name |
| `--list-sessions` | List all sessions and exit |
| `--cleanup <NAME>` | Clean up a session (removes worktree and state) |
| `--json` | Output in JSON format (for --list and --list-sessions) |
| `-- <ARGS>` | Additional arguments to pass to the agent binary |

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
