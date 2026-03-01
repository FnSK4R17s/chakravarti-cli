---
command: status
generated_from: lib.rs
last_commit: f92f604
hidden: true
---

# ckrv status

Check the status of a job.

## Description

Check the status of a job.

Displays job state, metrics, and execution details for a given job ID. Checks local metrics first, then falls back to cloud job lookup if the job is not found locally.

Shows duration, token usage, cost estimates, and step breakdowns.

## Arguments

| Argument | Required | Description |
|----------|----------|-------------|
| `<job_id>` | Yes | Job ID to check status for |

## Examples

```bash
# Check a local job
ckrv status <job-id>

# Check a cloud job
ckrv status <cloud-job-id>

# Output as JSON
ckrv status <job-id> --json
```
