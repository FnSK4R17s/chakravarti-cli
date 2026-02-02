---
command: ckrv plan
generated_from: crates/ckrv-cli/src/lib.rs
last_commit: 0ad833d
---

# ckrv plan

Generate execution plan from tasks (in Docker).

## Description

Generate execution plan from tasks using AI.

Analyzes the specification and tasks file to create a detailed implementation plan. Runs in a Docker container for isolation.

The plan breaks down work into atomic steps that AI agents can execute.

## Arguments

| Argument | Required | Description |
|----------|----------|-------------|
| `<spec-name>` | Yes | Name of the specification to plan |

## Options

| Flag | Description |
|------|-------------|
| `--model <model>` | AI model to use for planning |
| `--yes`, `-y` | Skip confirmation prompt |

## Examples

```bash
# Generate plan for a specification
ckrv plan my-feature

# Generate plan with GLM model
ckrv plan my-feature --model glm-4.7

# Skip confirmation prompt
ckrv plan my-feature --yes
```
