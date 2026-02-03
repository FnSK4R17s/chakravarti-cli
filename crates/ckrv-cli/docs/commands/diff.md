---
command: diff
generated_from: crates/ckrv-cli/src/lib.rs
last_commit: 34d5c95
---

# ckrv diff

View changes between current branch and base.

## Description

Shows a summary of modified, added, and deleted files compared to the base branch. Helps verify what will be included in a pull request.

Output can be formatted as JSON for programmatic use.

## Options

| Flag | Description |
|------|-------------|
| `--base <BRANCH>` | Base branch to compare against |
| `--stat` | Show diffstat only |
| `--files` | List changed files only |
| `--summary` | Show summary only |
| `--json` | Output format: JSON instead of human-readable |

## Examples

```bash
# Show diff summary
ckrv diff

# Show diff against specific branch
ckrv diff --base main

# Output as JSON
ckrv diff --json
```
