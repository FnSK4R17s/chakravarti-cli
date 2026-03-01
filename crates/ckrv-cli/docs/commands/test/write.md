---
command: test write
generated_from: commands/test.rs
last_commit: f92f604
---

# ckrv test write

Write new tests using test writer agent.

## Description

Write new tests using the configured test writer agent.

Analyzes changed files against the base branch and invokes an AI agent to generate tests for uncovered code. The agent runs inside a Docker sandbox for isolation.

Requires a test writer agent to be configured. Use --run to automatically execute the generated tests after writing.

## Options

| Flag | Description |
|------|-------------|
| `--base <BRANCH>` | Branch to compare against (default: main) |
| `--run` | Run tests after writing |

## Examples

```bash
# Write tests for changes against main
ckrv test write

# Write tests and run them immediately
ckrv test write --run

# Write tests against a specific branch
ckrv test write --base develop
```
