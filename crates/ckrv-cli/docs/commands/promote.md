---
command: ckrv promote
generated_from: crates/ckrv-cli/src/lib.rs
last_commit: 0ad833d
---

# ckrv promote

Create a pull request for the current branch.

## Description

Create a pull request for the current branch.

Pushes the feature branch and creates a pull request on GitHub/GitLab. Auto-generates PR title and description from the specification.

Requires remote repository access and appropriate permissions.

## Options

| Flag | Description |
|------|-------------|
| `--push` | Push to remote first |
| `--draft` | Create as draft PR |
| `--open` | Open PR URL in browser |
| `--base <branch>` | Target branch (default: main) |
| `--title <title>` | Custom PR title |

## Examples

```bash
# Create PR with auto-generated description
ckrv promote

# Create as draft PR
ckrv promote --draft

# Create PR with custom title
ckrv promote --title "feat: add user auth"
```
