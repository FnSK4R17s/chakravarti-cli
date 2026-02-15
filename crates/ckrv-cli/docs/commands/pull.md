---
command: pull
generated_from: crates/ckrv-cli/src/lib.rs
last_commit: 508766e
---

# ckrv pull

Pull results from a completed cloud job

## Description

Pull results from a completed cloud job.

Downloads all changes made during cloud execution and applies them to the local repository. Creates or updates the feature branch.

Jobs must be in a 'completed' state to pull.

## Arguments

| Argument | Required | Description |
|----------|----------|-------------|
| `job_id` | Yes | Job ID to pull results from |

## Options

| Flag | Description |
|------|-------------|
| `--apply` | Apply diff to current worktree (default: true) |
| `--output` | Output diff to file instead of applying |

## Examples

```bash
# Pull results to current directory
ckrv pull <job-id>

# Save diff to file
ckrv pull <job-id> --output changes.patch
```
