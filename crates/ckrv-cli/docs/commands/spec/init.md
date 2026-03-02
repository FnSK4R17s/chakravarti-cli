---
command: spec init
generated_from: commands/spec.rs
last_commit: 2a2da7f
---

# ckrv spec init

Initialize an empty spec directory with templates.

## Description

Initialize a new, empty specification directory with starter templates.

Creates a named directory under specs/ containing a blank spec.md template with the standard sections (overview, requirements, acceptance criteria) ready to be filled in.

Use this when you want to manually author a spec rather than generating one with AI via `ckrv spec new`.

## Arguments

| Argument | Required | Description |
|----------|----------|-------------|
| `<name>` | Yes | Name for the new spec directory |

## Examples

```bash
# Initialize a new spec directory
ckrv spec init my-feature

# Initialize with a hyphenated name
ckrv spec init user-auth-oauth2
```
