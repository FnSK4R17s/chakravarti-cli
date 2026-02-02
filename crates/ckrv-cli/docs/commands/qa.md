---
command: ckrv qa
generated_from: crates/ckrv-cli/src/lib.rs
last_commit: 0ad833d
---

# ckrv qa

QA code review and bug analysis.

## Description

QA code review and bug analysis.

AI-powered code review and quality assurance. Analyzes changes for potential bugs, security issues, and code quality improvements.

Subcommands: review, bugs, report

## Subcommands

| Subcommand | Description |
|------------|-------------|
| `review` | Review code quality of changes |
| `bugs` | Analyze for potential bugs |
| `report` | Generate full QA report |

## Options

| Flag | Description |
|------|-------------|
| `--base <branch>` | Branch to compare against (default: main) |
| `--output <file>` | Save report to file |
| `--json` | Output in JSON format |
| `--full` | Include all analysis types (quality, bugs, security) |

## Examples

```bash
# Review current changes
ckrv qa review

# Analyze for bugs
ckrv qa bugs

# Generate QA report
ckrv qa report
```
