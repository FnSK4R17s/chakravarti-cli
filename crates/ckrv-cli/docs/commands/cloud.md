---
command: cloud
generated_from: lib.rs
last_commit: 2a2da7f
hidden: true
---

# ckrv cloud

Cloud execution commands.

## Description

Cloud execution commands.

Manage remote job execution via Chakravarti Cloud. Submit jobs, monitor progress, and retrieve results from cloud workers.

## Subcommands

| Subcommand | Description |
|------------|-------------|
| `login` | Authenticate with Chakravarti Cloud |
| `logout` | Clear stored cloud credentials |
| `whoami` | Display current authenticated user |
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
