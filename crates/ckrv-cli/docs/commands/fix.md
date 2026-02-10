---
command: fix
generated_from: crates/ckrv-cli/src/lib.rs
last_commit: 1b27ca2
---

# ckrv fix

Fix verification errors with AI

## Description

Fix verification errors with AI.

Analyzes failed tests, lint errors, or build issues and uses AI to automatically generate fixes. Runs in an isolated Docker sandbox.

Best used after `ckrv verify` identifies issues.

## Options

| Flag | Description |
|------|-------------|
| `--check` | Re-run verification after fixing |
| `--error` | Specific error message to fix (from UI) |
| `--lint` | Fix only lint errors |
| `--test` | Fix only test failures |
| `--type` | Fix only type errors |

## Examples

```bash
# Fix all errors
ckrv fix

# Fix and re-verify
ckrv fix --check

# Fix only test failures
ckrv fix --test
```
