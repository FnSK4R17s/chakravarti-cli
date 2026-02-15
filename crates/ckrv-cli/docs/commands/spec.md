---
command: spec
generated_from: crates/ckrv-cli/src/lib.rs
last_commit: 508766e
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
| `new` | Create a new specification using AI from a natural language description |
| `clarify` | Resolve clarifications in an existing spec |
| `design` | Generate technical design document from a specification |
| `init` | Initialize an empty spec directory with templates |
| `tasks` | Generate implementation tasks from a specification |
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
