---
command: plan
generated_from: lib.rs
last_commit: f92f604
hidden: true
---

# ckrv plan

Generate execution plan from tasks (in Docker).

> **Note**: This is a legacy top-level command. Prefer `ckrv code plan` for the unified Code workflow.

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
# Generate plan for a specification
ckrv plan my-feature

# Generate plan with GLM model
ckrv plan my-feature --model glm-4.7

# Skip confirmation prompt
ckrv plan my-feature --yes
```
