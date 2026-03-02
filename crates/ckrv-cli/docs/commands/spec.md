---
command: spec
generated_from: lib.rs
last_commit: 2a2da7f
hidden: true
---

# ckrv spec

Create or manage feature specifications.

> **Note**: This is a legacy top-level command. Prefer `ckrv code spec` for the unified Code workflow.

## Description

Create or manage feature specifications.

Specifications are the source of truth for AI-driven development. They define what needs to be built, the requirements, and acceptance criteria.

## Subcommands

| Subcommand | Description |
|------------|-------------|
| `new` | Create a new specification using AI |
| `clarify` | Resolve clarifications in an existing spec |
| `design` | Generate technical design document |
| `init` | Initialize an empty spec directory with templates |
| `tasks` | Generate implementation tasks |
| `validate` | Validate a specification file |
| `list` | List all specifications |

## Examples

```bash
# Create a new specification
ckrv spec new "Add user authentication"

# List all specifications
ckrv spec list

# Validate a specification
ckrv spec validate my-feature
```
