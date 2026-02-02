---
command: ckrv pull
generated_from: crates/ckrv-cli/src/lib.rs
last_commit: 0ad833d
---

# ckrv pull

Pull results from a completed cloud job.

## Description

Pull results from a completed cloud job.

Downloads all changes made during cloud execution and applies them to the local repository. Creates or updates the feature branch.

Jobs must be in a 'completed' state to pull.

## Arguments

| Argument | Required | Description |
|----------|----------|-------------|
| `<job-id>` | Yes | Job ID to pull results from |

## Options

| Flag | Description |
|------|-------------|
| `--branch <name>` | Create or checkout specific branch |

## Examples

```bash
# Pull results to current directory
ckrv pull <job-id>

# Pull and create new branch
ckrv pull <job-id> --branch feature/new
```
