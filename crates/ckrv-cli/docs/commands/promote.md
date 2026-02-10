---
command: promote
generated_from: crates/ckrv-cli/src/lib.rs
last_commit: 1b27ca2
---

# ckrv promote

Create a pull request for the current branch

## Description

Create a pull request for the current branch.

Pushes the feature branch and creates a pull request on GitHub/GitLab. Auto-generates PR title and description from the specification.

Requires remote repository access and appropriate permissions.

## Options

| Flag | Description |
|------|-------------|
| `--base`, `-b` | Target branch for the PR (default: main or master) |
| `--draft` | Create as draft PR |
| `--open` | Open PR URL in browser after creation |
| `--push` | Push branch to remote before creating PR |
| `--remote` | Remote name (default: origin) |
| `--skip-verify` | Skip verification checks |

## Examples

```bash
# Create PR with auto-generated description
ckrv promote

# Create as draft PR
ckrv promote --draft

# Push and create PR
ckrv promote --push
```
