---
command: ckrv logs
generated_from: crates/ckrv-cli/src/lib.rs
last_commit: 039d181
---

# ckrv logs

Stream or view logs from a cloud job

## Description

Stream or view logs from a cloud job.

Shows real-time output from running jobs or historical logs from completed jobs. Supports filtering by task or agent.

Use --follow for continuous streaming.

## Arguments

| Argument | Required | Description |
|----------|----------|-------------|
| `<job-id>` | Yes | Cloud job ID to view logs for |

## Options

| Flag | Description |
|------|-------------|
| `--follow` | Stream logs continuously |
| `--task` | Filter by task number |

## Examples

```bash
# Stream logs from running job
ckrv logs <job-id> --follow

# View completed job logs
ckrv logs <job-id>

# Filter by task
ckrv logs <job-id> --task 3
```
