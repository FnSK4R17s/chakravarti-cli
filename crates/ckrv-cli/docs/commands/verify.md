---
command: verify
generated_from: crates/ckrv-cli/src/lib.rs
last_commit: 34d5c95
---

# ckrv verify

Run tests, lint, and quality checks.

## Description

Validates the current code against project quality standards. Runs the test suite, linters, and any custom verification scripts.

Failed verifications can be fixed with `ckrv fix`.

## Options

| Flag | Description |
|------|-------------|
| `--tests-only` | Run only tests, skip linting |
| `--lint-only` | Run only linting, skip tests |
| `--verbose`, `-v` | Enable verbose logging |
| `--json` | Output format: JSON instead of human-readable |

## Examples

```bash
# Run all verifications
ckrv verify

# Run only tests
ckrv verify --tests-only

# Run in JSON output mode
ckrv verify --json
```
