---
command: ckrv diff
generated_from: crates/ckrv-cli/src/lib.rs
last_commit: 0ad833d
---

# ckrv diff

View changes between current branch and base.

## Description

View changes between current branch and base.

Shows a summary of modified, added, and deleted files compared to the base branch. Helps verify what will be included in a pull request.

Output can be formatted as JSON for programmatic use.

## Options

| Flag | Description |
|------|-------------|
| `--base <branch>` | Compare against specific branch (default: main) |
| `--json` | Output as JSON |
| `--stat` | Show file statistics only |
| `--files` | List changed files only |

## Examples

```bash
# Show diff summary
ckrv diff

# Show diff against specific branch
ckrv diff --base main

# Output as JSON
ckrv diff --json
```
