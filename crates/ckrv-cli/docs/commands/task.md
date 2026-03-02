---
command: task
generated_from: lib.rs
last_commit: 2a2da7f
hidden: true
---

# ckrv task

Execute a workflow-based agent task.

## Description

Execute a workflow-based agent task.

Initiates a multi-step workflow (e.g., Plan -> Implement) using an AI agent in a sandboxed environment. Tasks are defined in tasks.yaml and executed with configurable agents and isolation levels.

Supports Docker sandboxing, git worktrees, and workflow customization.

## Arguments

| Argument | Required | Description |
|----------|----------|-------------|
| `<target>` | Yes | Task description or ID (e.g., "T001") |

## Options

| Flag | Description |
|------|-------------|
| `--workflow`, `-w` | Workflow to use (name or path to YAML file, default: swe) |
| `--dry-run` | Show plan without executing |
| `--continue-task`, `-c` | Continue a previous task by ID |
| `--agent <AGENT>` | Agent tool to use (default: claude) |
| `--no-sandbox` | Skip Docker sandbox and run agent locally (NOT RECOMMENDED) |
| `--keep-container` | Keep Docker container after execution (for debugging) |
| `--use-worktree <PATH>` | Use existing worktree path instead of creating one |

## Examples

```bash
# Run a task by description
ckrv task "Add user authentication"

# Run a specific task by ID
ckrv task T001

# Dry run (show steps without executing)
ckrv task T001 --dry-run

# Use a custom workflow
ckrv task T001 --workflow custom.yml
```
