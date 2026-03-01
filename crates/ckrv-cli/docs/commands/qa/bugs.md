---
command: qa bugs
generated_from: commands/qa.rs
last_commit: f92f604
---

# ckrv qa bugs

Analyze for potential bugs.

## Description

Analyze changed files for potential bugs and error-handling gaps.

Scans the diff against the base branch and filters findings to only bug-related categories: potential bugs and missing or incorrect error handling. Each finding includes a severity level and a suggested fix.

Requires a configured QA agent. If no agent is found, exits with code 4. Full bug analysis with deeper heuristics requires Docker sandbox integration.

## Options

| Flag | Description |
|------|-------------|
| `--base <BRANCH>` | Branch to compare against (default: main) |

## Examples

```bash
# Scan for bugs against the default base branch (main)
ckrv qa bugs

# Scan for bugs against a specific branch
ckrv qa bugs --base develop

# Get machine-readable bug list
ckrv qa bugs --json
```
