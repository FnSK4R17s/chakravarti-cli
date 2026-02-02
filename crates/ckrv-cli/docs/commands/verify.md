---
command: ckrv verify
generated_from: crates/ckrv-cli/src/lib.rs
last_commit: 0ad833d
---

# ckrv verify

Run tests, lint, and quality checks.

## Description

Run tests, lint, and quality checks.

Validates the current code against project quality standards. Runs the test suite, linters, and any custom verification scripts.

Failed verifications can be fixed with `ckrv fix`.

## Options

| Flag | Description |
|------|-------------|
| `--tests-only` | Run only tests |
| `--lint` | Run linting only |
| `--typecheck` | Run type checking only |
| `--fix` | Auto-fix linting issues |
| `--save` | Save results to verification.yaml |
| `--json` | Output in JSON format |

## Examples

```bash
# Run all verifications
ckrv verify

# Run only tests
ckrv verify --tests-only

# Run in JSON output mode
ckrv verify --json
```
