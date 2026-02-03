---
command: promote
generated_from: crates/ckrv-cli/src/lib.rs
last_commit: 34d5c95
---

# ckrv promote

Create a pull request for the current branch.

## Description

Pushes the feature branch and creates a pull request on GitHub/GitLab. Auto-generates PR title and description from the specification.

Requires remote repository access and appropriate permissions.

## Options

| Flag | Description |
|------|-------------|
| `--title <TITLE>` | Custom PR title |
| `--draft` | Create as draft PR |
| `--no-push` | Don't push, only create PR |
| `--verbose`, `-v` | Enable verbose logging |
| `--json` | Output format: JSON instead of human-readable |

## Examples

```bash
# Create PR with auto-generated description
ckrv promote

# Create as draft PR
ckrv promote --draft

# Create PR with custom title
ckrv promote --title "feat: add user auth"
```
