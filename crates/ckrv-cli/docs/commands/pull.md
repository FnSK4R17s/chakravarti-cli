---
command: pull
generated_from: crates/ckrv-cli/src/lib.rs
last_commit: 34d5c95
---

# ckrv pull

Pull results from a completed cloud job.

## Description

Downloads all changes made during cloud execution and applies them to the local repository. Creates or updates the feature branch.

Jobs must be in a 'completed' state to pull.

## Arguments

| Argument | Required | Description |
|----------|----------|-------------|
| `<job-id>` | Yes | Job ID to pull results from |

## Options

| Flag | Description |
|------|-------------|
| `--branch <BRANCH>` | Create/update specific branch |
| `--force` | Overwrite existing changes |
| `--verbose`, `-v` | Enable verbose logging |

## Examples

```bash
# Pull results to current directory
ckrv pull <job-id>

# Pull and create new branch
ckrv pull <job-id> --branch feature/new
```
