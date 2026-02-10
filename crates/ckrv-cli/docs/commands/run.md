---
command: run
generated_from: crates/ckrv-cli/src/lib.rs
last_commit: 1b27ca2
---

# ckrv run

Run a job based on a specification

## Description

Run a job based on a specification.

Executes the implementation plan using AI agents in isolated Docker sandboxes. Each task is executed in sequence with full logging and progress tracking.

Results are committed to a feature branch for review.

## Arguments

| Argument | Required | Description |
|----------|----------|-------------|
| `spec` | No | Path to the specification file. If not provided, will detect from branch name |

## Options

| Flag | Description |
|------|-------------|
| `--agent` | Agent to use for execution: claude or codex (default: claude) |
| `--cloud` | Execute job in Chakravarti Cloud instead of locally |
| `--credential` | Git credential name to use for cloud execution (for private repos) |
| `--executor-model`, `-e` | Override the AI model/agent to use for execution |
| `--optimize`, `-o` | Optimization strategy (default: balanced) |

## Examples

```bash
# Run all tasks for a specification
ckrv run my-feature

# Run with specific agent
ckrv run my-feature --agent claude-3.5

# Dry run (show what would be done)
ckrv run my-feature --dry-run
```
