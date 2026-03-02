---
command: logs
generated_from: lib.rs
last_commit: 2a2da7f
hidden: true
---

# ckrv logs

Stream or view logs from a cloud job.

## Description

Stream or view logs from a cloud job.

Shows real-time output from running jobs or historical logs from completed jobs. Supports filtering by task or agent.

Use --follow for continuous streaming.

## Arguments

| Argument | Required | Description |
|----------|----------|-------------|
| `<job_id>` | Yes | Job ID to get logs for |

## Options

| Flag | Description |
|------|-------------|
| `--follow`, `-f` | Follow log output (stream in real-time) |
| `--json` | Output as JSON |
| `--tail`, `-n` | Number of recent log lines to show (default: 100) |

## Examples

```bash
# Stream logs from running job
ckrv logs <job-id> --follow

# View completed job logs
ckrv logs <job-id>

# Filter by task
ckrv logs <job-id> --task 3
```
