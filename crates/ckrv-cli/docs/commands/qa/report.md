---
command: qa report
generated_from: commands/qa.rs
last_commit: 2a2da7f
---

# ckrv qa report

Generate full QA report.

## Description

Generate a comprehensive QA report covering all analysis categories.

Produces a Markdown report that includes a change summary, file-level breakdown (when --full is used), and all QA findings ranked by severity. The report header contains branch name, base branch, and timestamp.

Requires a configured QA agent. If no agent is found, exits with code 4. Use --full to include per-file statistics (lines added/removed, change type) in the report.

## Options

| Flag | Description |
|------|-------------|
| `--base <BRANCH>` | Branch to compare against (default: main) |
| `--full` | Include all analysis types with per-file details |
| `--output`, `-o` | Output file path |

## Examples

```bash
# Generate a standard report against main
ckrv qa report

# Generate a full report with per-file details
ckrv qa report --full

# Save the full report to a file against a custom base
ckrv qa report --full --base develop --output qa-report.md
```
