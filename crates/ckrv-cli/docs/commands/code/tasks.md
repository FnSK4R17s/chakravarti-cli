---
command: code tasks
generated_from: commands/code.rs
last_commit: f92f604
---

# ckrv code tasks

Generate implementation tasks from a specification.

## Description

Generate implementation tasks from a specification.

Analyzes the specification and produces a structured task breakdown that can be used for planning and execution.

This is a convenience alias for `ckrv code spec tasks`.

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
# Generate tasks for auto-detected spec
ckrv code tasks

# Generate tasks for a specific spec
ckrv code tasks path/to/spec

# Force regeneration
ckrv code tasks --force
```
