---
command: ckrv spec
generated_from: crates/ckrv-cli/src/lib.rs
last_commit: 039d181
---

# ckrv spec

Create or manage feature specifications

## Description

Create or manage feature specifications.

Specifications are the source of truth for AI-driven development. They define what needs to be built, the requirements, and acceptance criteria.

Subcommands: new, list, validate, edit, show

## Subcommands

| Subcommand | Description |
|------------|-------------|
| `new` | Create a new specification |
| `list` | List all specifications |
| `validate` | Validate a specification |
| `edit` | Edit an existing specification |
| `show` | Show specification details |

## Examples

```bash
# Create a new specification
ckrv spec new "Add user authentication"

# List all specifications
ckrv spec list

# Validate a specification
ckrv spec validate my-feature
```
