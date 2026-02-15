---
command: spec tasks
generated_from: crates/ckrv-cli/src/commands/spec.rs
last_commit: 508766e
---

# ckrv spec tasks

Generate implementation tasks from a specification

## Arguments

| Argument | Required | Description |
|----------|----------|-------------|
| `spec` | No | Path to the spec file (optional — auto-detects from current branch if not provided) |

## Options

| Flag | Description |
|------|-------------|
| `--force`, `-f` | Force regeneration of tasks even if they exist |
