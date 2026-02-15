---
command: spec design
generated_from: crates/ckrv-cli/src/commands/spec.rs
last_commit: 508766e
---

# ckrv spec design

Generate technical design document from a specification

## Arguments

| Argument | Required | Description |
|----------|----------|-------------|
| `spec` | No | Path to the spec file (optional — auto-detects from current branch if not provided) |

## Options

| Flag | Description |
|------|-------------|
| `--force`, `-f` | Force regeneration of design even if it exists |
