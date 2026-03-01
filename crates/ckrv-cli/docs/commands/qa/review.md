---
command: qa review
generated_from: commands/qa.rs
last_commit: f92f604
---

# ckrv qa review

Review code quality of changes.

## Description

Review code quality of changes against a base branch.

Analyzes modified files for code quality issues including style violations, missing documentation, complexity concerns, and best-practice deviations. Produces a structured report with severity-ranked findings.

Results can be saved to a file with --output or printed to stdout. When --json is used, output is a machine-readable QaReviewOutput object.

Requires a configured QA agent. If no agent is found, exits with code 4. Exits with code 1 if critical issues are detected.

## Options

| Flag | Description |
|------|-------------|
| `--base <BRANCH>` | Branch to compare against (default: main) |
| `--output`, `-o` | Output file path |

## Examples

```bash
# Review changes against the default base branch (main)
ckrv qa review

# Review changes against a specific branch
ckrv qa review --base develop

# Save review report to a file
ckrv qa review --output qa-review.md
```
