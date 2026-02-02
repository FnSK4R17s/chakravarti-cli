---
command: ckrv run
generated_from: crates/ckrv-cli/src/lib.rs
last_commit: 0ad833d
---

# ckrv run

Run a job based on a specification.

## Description

Run a job based on a specification.

Executes the implementation plan using AI agents in isolated Docker sandboxes. Each task is executed in sequence with full logging and progress tracking.

Results are committed to a feature branch for review.

## Arguments

| Argument | Required | Description |
|----------|----------|-------------|
| `<spec-name>` | Yes | Name of the specification to run |

## Options

| Flag | Description |
|------|-------------|
| `--agent <agent>` | AI agent to use for execution |
| `--dry-run` | Show what would be done without executing |
| `--batch <N>` | Run specific batch only |
| `--parallel <N>` | Max parallel agents (default: 3) |
| `--no-merge` | Skip auto-merge step |

## Examples

```bash
# Run all tasks for a specification
ckrv run my-feature

# Run with specific agent
ckrv run my-feature --agent claude-3.5

# Dry run (show what would be done)
ckrv run my-feature --dry-run
```
