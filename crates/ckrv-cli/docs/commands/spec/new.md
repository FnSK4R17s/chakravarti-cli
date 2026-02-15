---
command: spec new
generated_from: crates/ckrv-cli/src/commands/spec.rs
last_commit: 508766e
---

# ckrv spec new

Create a new specification using AI from a natural language description

## Arguments

| Argument | Required | Description |
|----------|----------|-------------|
| `description` | Yes | Natural language description of the feature (e.g., "Add user authentication") |

## Options

| Flag | Description |
|------|-------------|
| `--name`, `-n` | Optional short name for the spec (auto-generated from description if not provided) |
