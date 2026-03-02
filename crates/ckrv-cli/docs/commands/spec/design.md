---
command: spec design
generated_from: commands/spec.rs
last_commit: 2a2da7f
---

# ckrv spec design

Generate technical design document from a specification.

## Description

Generate a technical design document from an existing specification.

Produces a design.md file alongside the spec containing:
- Architecture decisions and component diagrams
- Data models and API contracts
- Implementation strategy and dependencies

If no spec path is provided, auto-detects the spec from the current Git branch name. Use --force to regenerate an existing design document.

## Arguments

| Argument | Required | Description |
|----------|----------|-------------|
| `<spec>` | No | Path to the spec file (auto-detects from current branch if not provided) |

## Options

| Flag | Description |
|------|-------------|
| `--force`, `-f` | Force regeneration of design even if it exists |

## Examples

```bash
# Generate design from the current branch spec
ckrv spec design

# Generate design for a specific spec
ckrv spec design specs/auth-oauth2/spec.md

# Force regeneration of an existing design
ckrv spec design --force
```
