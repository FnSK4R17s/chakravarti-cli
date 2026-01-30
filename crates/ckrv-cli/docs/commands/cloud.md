---
command: ckrv cloud
generated_from: crates/ckrv-cli/src/lib.rs
last_commit: 039d181
---

# ckrv cloud

Cloud execution commands

## Description

Cloud execution commands.

Manage remote job execution via Chakravarti Cloud. Submit jobs, monitor progress, and retrieve results from cloud workers.

Subcommands: login, submit, status, cancel

## Subcommands

| Subcommand | Description |
|------------|-------------|
| `login` | Login to Chakravarti Cloud |
| `submit` | Submit a job to cloud |
| `status` | Check job status |
| `cancel` | Cancel a running job |
| `credentials` | Manage git credentials for private repositories |

## Examples

```bash
# Login to cloud
ckrv cloud login

# Submit a job
ckrv cloud submit my-feature

# Check job status
ckrv cloud status <job-id>
```
