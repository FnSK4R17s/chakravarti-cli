---
command: code
generated_from: lib.rs
last_commit: f92f604
---

# ckrv code

Code workflow: spec, tasks, plan, run, diff.

## Description

Code workflow commands — mirrors the Code page tabs in the Web UI.

Groups the full development pipeline under a single namespace:
- spec — create and manage feature specifications
- tasks — generate implementation tasks from a spec
- plan — generate an execution plan from tasks
- run — execute the plan with AI agents
- diff — review changes before promoting

Use `ckrv code <subcommand> --help` for details on each step.

## Subcommands

| Subcommand | Description |
|------------|-------------|
| `spec` | Create or manage feature specifications |
| `tasks` | Generate implementation tasks from a specification |
| `plan` | Generate execution plan from tasks (in Docker) |
| `run` | Run a job based on a specification |
| `diff` | View changes between current branch and base |

## Examples

```bash
# Create a new spec and generate tasks
ckrv code spec new "Add user authentication"
ckrv code tasks

# Plan and run
ckrv code plan
ckrv code run

# Review changes
ckrv code diff
```
