---
command: spec list
generated_from: commands/spec.rs
last_commit: 2a2da7f
---

# ckrv spec list

List all specifications.

## Description

List all specifications found in the specs/ directory.

Displays a table of all spec directories with their names, statuses, and file paths. Useful for getting an overview of all features being tracked.

The repository must be initialized with `ckrv init` before listing specs.

## Examples

```bash
# List all specs
ckrv spec list

# List specs with JSON output
ckrv spec list --json
```
