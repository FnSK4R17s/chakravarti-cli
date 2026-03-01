---
command: code diff
generated_from: commands/code.rs
last_commit: f92f604
---

# ckrv code diff

View changes between current branch and base.

## Description

View changes between current branch and base.

Shows a summary of modified, added, and deleted files compared to the base branch. Helps verify what will be included in a pull request.

Output can be formatted as JSON for programmatic use.

## Options

| Flag | Description |
|------|-------------|
| `--base`, `-b` | Base branch to compare against (default: main or master) |
| `--stat` | Show diff statistics only |
| `--files` | Show file list only |
| `--summary` | Generate AI summary of changes |
| `--color <MODE>` | Color mode: auto, always, or never (default: auto) |

## Examples

```bash
# Show diff summary
ckrv code diff

# Show diff against specific branch
ckrv code diff --base main

# Output as JSON
ckrv code diff --json
```
