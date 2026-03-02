---
command: spec validate
generated_from: commands/spec.rs
last_commit: 2a2da7f
---

# ckrv spec validate

Validate a specification file.

## Description

Validate a specification file for correctness and completeness.

Checks that the spec contains all required sections, validates field formats, and reports any errors or warnings. Returns a non-zero exit code if validation fails.

If no path is provided, auto-detects the spec from the current Git branch name. Supports JSON output for CI integration.

## Arguments

| Argument | Required | Description |
|----------|----------|-------------|
| `<path>` | No | Path to the spec file (auto-detects from current branch if not provided) |

## Examples

```bash
# Validate the spec detected from the current branch
ckrv spec validate

# Validate a specific spec file
ckrv spec validate specs/auth-oauth2/spec.md

# Validate with JSON output for CI
ckrv spec validate --json
```
