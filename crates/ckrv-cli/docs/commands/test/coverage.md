---
command: test coverage
generated_from: commands/test.rs
last_commit: f92f604
---

# ckrv test coverage

Check test coverage of changed files.

## Description

Check test coverage of changed files.

Scans files changed between the current branch and base branch to determine which source files have corresponding tests. Reports a coverage percentage based on file-level test presence.

Warns if coverage drops below 80%. Use `ckrv test plan` to see exactly which files need tests.

## Options

| Flag | Description |
|------|-------------|
| `--base <BRANCH>` | Branch to compare against (default: main) |

## Examples

```bash
# Check coverage against main
ckrv test coverage

# Check coverage against a specific branch
ckrv test coverage --base develop

# Check coverage with JSON output
ckrv test coverage --json
```
