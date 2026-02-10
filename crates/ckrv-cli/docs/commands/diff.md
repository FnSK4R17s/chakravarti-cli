---
command: diff
generated_from: crates/ckrv-cli/src/lib.rs
last_commit: 1b27ca2
---

# ckrv diff

View changes between current branch and base

## Description

View changes between current branch and base.

Shows a summary of modified, added, and deleted files compared to the base branch. Helps verify what will be included in a pull request.

Output can be formatted as JSON for programmatic use.

## Options

| Flag | Description |
|------|-------------|
| `--base`, `-b` | Base branch to compare against (default: main or master) |
| `--color` | Color mode for diff output (default: auto) |
| `--files` | Show file list only |
| `--stat` | Show diff statistics only |
| `--summary` | Generate AI summary of changes |

## Examples

```bash
# Show diff summary
ckrv diff

# Show diff against specific branch
ckrv diff --base main

# Output as JSON
ckrv diff --json
```
