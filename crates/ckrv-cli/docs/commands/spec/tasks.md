---
command: spec tasks
generated_from: commands/spec.rs
last_commit: f92f604
---

# ckrv spec tasks

Generate implementation tasks from a specification.

## Description

Generate implementation tasks from an existing specification.

Analyzes the spec and produces a tasks.md file containing a set of discrete, actionable implementation tasks with dependency ordering. Each task includes a title, description, and prompt suitable for agent execution.

If no spec path is provided, auto-detects the spec from the current Git branch name. Use --force to regenerate tasks even if a tasks file already exists.

## Arguments

| Argument | Required | Description |
|----------|----------|-------------|
| `<spec>` | No | Path to the spec file (auto-detects from current branch if not provided) |

## Options

| Flag | Description |
|------|-------------|
| `--force`, `-f` | Force regeneration of tasks even if they exist |

## Examples

```bash
# Generate tasks from the current branch spec
ckrv spec tasks

# Generate tasks for a specific spec
ckrv spec tasks specs/auth-oauth2/spec.md

# Force regeneration of existing tasks
ckrv spec tasks --force
```
