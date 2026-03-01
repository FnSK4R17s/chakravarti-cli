---
command: run
generated_from: lib.rs
last_commit: f92f604
hidden: true
---

# ckrv run

Run a job based on a specification.

> **Note**: This is a legacy top-level command. Prefer `ckrv code run` for the unified Code workflow.

## Description

Run a job based on a specification.

Executes the implementation plan using AI agents in isolated Docker sandboxes. Each task is executed in sequence with full logging and progress tracking.

Results are committed to a feature branch for review.

## Arguments

| Argument | Required | Description |
|----------|----------|-------------|
| `<spec>` | No | Path to the specification file (auto-detects from branch if not provided) |

## Options

| Flag | Description |
|------|-------------|
| `--optimize`, `-o` | Optimization strategy: cost, time, or balanced (default: balanced) |
| `--executor-model`, `-e` | Override the AI model/agent to use |
| `--agent <AGENT>` | Agent to use: claude, codex, or kilo (default: claude) |
| `--cloud` | Execute job in Chakravarti Cloud instead of locally |
| `--credential <NAME>` | Git credential name for cloud execution (private repos) |

## Examples

```bash
# Run all tasks for a specification
ckrv run my-feature

# Run with specific agent
ckrv run my-feature --agent claude-3.5

# Dry run (show what would be done)
ckrv run my-feature --dry-run
```
