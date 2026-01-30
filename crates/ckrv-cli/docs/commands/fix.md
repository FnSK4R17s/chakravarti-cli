---
command: ckrv fix
generated_from: crates/ckrv-cli/src/lib.rs
last_commit: 039d181
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
| `--agent` | Agent to use for fixing |
| `--tests-only` | Fix only test failures |

## Examples

```bash
# Fix all errors
ckrv fix

# Fix with specific agent
ckrv fix --agent claude-3.5

# Fix only test failures
ckrv fix --tests-only
```
