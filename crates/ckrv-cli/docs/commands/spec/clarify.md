---
command: spec clarify
generated_from: commands/spec.rs
last_commit: f92f604
---

# ckrv spec clarify

Resolve clarifications in an existing spec.

## Description

Resolve open clarifications and ambiguities in an existing specification.

Reviews the spec for unclear requirements, missing details, or conflicting constraints, then interactively resolves them using AI. Updates the spec file in-place with the resolved clarifications.

If no spec path is provided, auto-detects the spec from the current Git branch name.

## Arguments

| Argument | Required | Description |
|----------|----------|-------------|
| `<spec>` | No | Path to the spec file (auto-detects from current branch if not provided) |

## Examples

```bash
# Clarify the spec detected from the current branch
ckrv spec clarify

# Clarify a specific spec file
ckrv spec clarify specs/auth-oauth2/spec.md
```
