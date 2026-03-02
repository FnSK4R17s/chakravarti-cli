---
command: code spec
generated_from: commands/code.rs
last_commit: 2a2da7f
---

# ckrv code spec

Create or manage feature specifications.

## Description

Create or manage feature specifications.

Specifications are the source of truth for AI-driven development. They define what needs to be built, the requirements, and acceptance criteria.

Subcommands: new, list, validate, clarify, design, init, tasks

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
ckrv code spec new "Add user authentication"

# List all specifications
ckrv code spec list

# Validate a specification
ckrv code spec validate my-feature
```
