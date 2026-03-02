---
command: code run
generated_from: commands/code.rs
last_commit: 2a2da7f
---

# ckrv code run

Run a job based on a specification.

## Description

Run a job based on a specification.

Executes the implementation plan using AI agents in isolated Docker sandboxes. Each task is executed in sequence with full logging and progress tracking.

Results are committed to a feature branch for review.

## Arguments

| Argument | Required | Description |
|----------|----------|-------------|
| `<spec>` | No | Path to the specification file (auto-detects from branch if not provided) |

## Options

| Flag | Description |
|------|-------------|
| `--optimize`, `-o` | Optimization strategy: cost, time, or balanced (default: balanced) |
| `--executor-model`, `-e` | Override the AI model/agent to use |
| `--agent <AGENT>` | Agent to use: claude, codex, or kilo (default: claude) |
| `--cloud` | Execute job in Chakravarti Cloud instead of locally |
| `--credential <NAME>` | Git credential name for cloud execution (private repos) |

## Examples

```bash
# Run all tasks for auto-detected spec
ckrv code run

# Run with specific agent
ckrv code run my-feature --agent claude

# Run with cost optimization
ckrv code run --optimize cost
```
