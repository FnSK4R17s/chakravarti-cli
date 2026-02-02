---
command: ckrv init
generated_from: crates/ckrv-cli/src/lib.rs
last_commit: 0ad833d
---

# ckrv init

Initialize Chakravarti in the current repository.

## Description

Initialize Chakravarti in the current repository.

Creates the `.chakravarti/` directory with default configuration files including `config.yaml` for project settings and initializes the specs directory.

This is typically the first command to run when setting up a new project for AI-driven development with Chakravarti.

## Options

| Flag | Description |
|------|-------------|
| `--verbose`, `-v` | Enable verbose logging |

## Examples

```bash
# Initialize in current directory
ckrv init

# Initialize with verbose output
ckrv init --verbose
```
