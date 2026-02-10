---
command: verify
generated_from: crates/ckrv-cli/src/lib.rs
last_commit: 1b27ca2
---

# ckrv verify

Run tests, lint, and quality checks

## Description

Run tests, lint, and quality checks.

Validates the current code against project quality standards. Runs the test suite, linters, and any custom verification scripts.

Failed verifications can be fixed with `ckrv fix`.

## Options

| Flag | Description |
|------|-------------|
| `--continue-on-failure` | Continue on failure (run all checks even if some fail) |
| `--fix` | Auto-fix issues where possible |
| `--lint` | Run only lint checks |
| `--save` | Save results to verification.yaml |
| `--test` | Run only tests |
| `--type` | Run only type checks |

## Examples

```bash
# Run all verifications
ckrv verify

# Run only tests
ckrv verify --test

# Run in JSON output mode
ckrv verify --json
```
