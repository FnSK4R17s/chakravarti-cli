---
command: code plan
generated_from: commands/code.rs
last_commit: f92f604
---

# ckrv code plan

Generate execution plan from tasks (in Docker).

## Description

Generate execution plan from tasks using AI.

Analyzes the specification and tasks file to create a detailed implementation plan. Runs in a Docker container for isolation.

The plan breaks down work into atomic steps that AI agents can execute.

## Arguments

| Argument | Required | Description |
|----------|----------|-------------|
| `<spec>` | No | Path to the specification directory (auto-detects from branch if not provided) |

## Options

| Flag | Description |
|------|-------------|
| `--force`, `-f` | Force regeneration even if plan.yaml already exists |

## Examples

```bash
# Generate plan for auto-detected spec
ckrv code plan

# Generate plan for a specific spec
ckrv code plan my-feature

# Force regeneration
ckrv code plan --force
```
