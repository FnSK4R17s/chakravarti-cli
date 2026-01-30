---
command: ckrv verify
generated_from: crates/ckrv-cli/src/lib.rs
last_commit: 039d181
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
| `--lint` | Run only lint checks |
| `--type` | Run only type checks |
| `--test` | Run only tests |
| `--fix` | Auto-fix issues where possible |
| `--continue-on-failure` | Continue on failure (run all checks even if some fail) |
| `--save` | Save results to verification.yaml |

## Examples

```bash
# Run all verifications
ckrv verify

# Run only tests
ckrv verify --tests-only

# Run in JSON output mode
ckrv verify --json
```
