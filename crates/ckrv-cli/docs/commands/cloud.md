---
command: cloud
generated_from: crates/ckrv-cli/src/lib.rs
last_commit: 34d5c95
---

# ckrv cloud

Cloud execution commands.

## Description

Manage remote job execution via Chakravarti Cloud. Submit jobs, monitor progress, and retrieve results from cloud workers.

Subcommands: login, submit, status, cancel

## Subcommands

| Subcommand | Description |
|------------|-------------|
| `login` | Authenticate with Chakravarti Cloud |
| `submit` | Submit a job for cloud execution |
| `status` | Check job status |
| `cancel` | Cancel a running job |

## Examples

```bash
# Login to cloud
ckrv cloud login

# Submit a job
ckrv cloud submit my-feature

# Check job status
ckrv cloud status <job-id>
```
