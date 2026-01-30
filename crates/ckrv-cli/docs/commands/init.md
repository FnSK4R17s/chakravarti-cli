---
command: ckrv init
generated_from: crates/ckrv-cli/src/lib.rs
last_commit: 039d181
---

# ckrv init

Initialize Chakravarti in the current repository

## Description

Initialize Chakravarti in the current repository.

Creates the `.chakravarti/` directory with default configuration files including `config.yaml` for project settings and initializes the specs directory.

This is typically the first command to run when setting up a new project for AI-driven development with Chakravarti.

## Options

| Flag | Description |
|------|-------------|
| `--force` | Force reinitialization even if already initialized |

## Examples

```bash
# Initialize in current directory
ckrv init

# Initialize with verbose output
ckrv init --verbose
```
