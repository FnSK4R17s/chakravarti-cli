---
command: report
generated_from: lib.rs
last_commit: f92f604
hidden: true
---

# ckrv report

View the metrics report for a job.

## Description

View the metrics report for a job.

Displays detailed timing, token usage, and cost estimates for a completed job. Includes per-model breakdowns and step-level metrics.

Use --detailed for per-step timing information.

## Arguments

| Argument | Required | Description |
|----------|----------|-------------|
| `<job_id>` | Yes | Job ID to view report for |

## Options

| Flag | Description |
|------|-------------|
| `--detailed` | Show detailed per-step breakdown |

## Examples

```bash
# View job report
ckrv report <job-id>

# Detailed per-step breakdown
ckrv report <job-id> --detailed

# Output as JSON
ckrv report <job-id> --json
```
