# Quickstart: Agent Orchestration

This guide shows you how to use the new Orchestrator to automate coding tasks.

## 1. Prerequisites

- **Docker** running.
- **Claude Code** credentials configured (`~/.claude.json`).
- A CLI installed with this feature.

## 2. Running a Task

The simplest way is to run a task with the default "SWE" (Software Engineering) workflow.

```bash
# General syntax
ckrv task "Your instruction here"

# Example: Create a new file
ckrv task "Create a README.md file explaining this project"
```

## 3. What Happens?

1. **Sandbox Created**: A Docker container spins up.
2. **Worktree Created**: A new git worktree is created in `.ckrv/tasks/<id>/workspace`.
3. **Planning**: The agent thinks about the task and generates a `plan.md`.
4. **Execution**: The agent writes the code.
5. **Success**: You are given the path to the modified workspace.

## 4. Custom Workflows

You can define your own workflows in `.ckrv/workflows/my-workflow.yml`.

```yaml
version: '1.0'
name: 'review-only'
steps:
  - id: review
    name: Code Review
    prompt: |
      Review the files in current directory.
      Output your review to review.md.
    outputs:
      - name: review_file
        type: file
        source: review.md
```

Run it:

```bash
ckrv task --workflow review-only "Check for security bugs"
```
