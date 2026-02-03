---
command: run
generated_from: crates/ckrv-cli/src/lib.rs
last_commit: 34d5c95
---

# ckrv run

Run a job based on a specification.

## Description

Executes the implementation plan using AI agents in isolated Docker sandboxes. Each task is executed in sequence with full logging and progress tracking.

Results are committed to a feature branch for review.

## Arguments

| Argument | Required | Description |
|----------|----------|-------------|
| `<spec>` | Yes | Name of the specification to execute |

## Options

| Flag | Description |
|------|-------------|
| `--agent <AGENT>` | AI agent to use (e.g., claude-3.5, codex) |
| `--dry-run` | Show what would be done without executing |
| `--resume` | Resume from last checkpoint |
| `--verbose`, `-v` | Enable verbose logging |
| `--json` | Output format: JSON instead of human-readable |

## Examples

```bash
# Run all tasks for a specification
ckrv run my-feature

# Run with specific agent
ckrv run my-feature --agent claude-3.5

# Dry run (show what would be done)
ckrv run my-feature --dry-run
```
