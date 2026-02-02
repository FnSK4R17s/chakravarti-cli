---
command: ckrv cloud
generated_from: crates/ckrv-cli/src/lib.rs
last_commit: 0ad833d
---

# ckrv cloud

Cloud execution commands.

## Description

Cloud execution commands.

Manage remote job execution via Chakravarti Cloud. Submit jobs, monitor progress, and retrieve results from cloud workers.

Subcommands: login, submit, status, cancel

## Subcommands

| Subcommand | Description |
|------------|-------------|
| `login` | Authenticate with cloud |
| `submit <spec>` | Submit a job for execution |
| `status <job-id>` | Check job status |
| `cancel <job-id>` | Cancel a running job |

## Examples

```bash
# Login to cloud
ckrv cloud login

# Submit a job
ckrv cloud submit my-feature

# Check job status
ckrv cloud status <job-id>
```
