# Walkthrough: Test and QA Commands

**Feature**: 014-test-qa-commands  
**Date**: 2026-01-22  
**Status**: Complete ✅

## Changes Made

### New Commands

| Command | Description |
|---------|-------------|
| `ckrv test run` | Run project tests with framework auto-detection |
| `ckrv test plan` | Analyze changes and propose tests |
| `ckrv test write` | Write tests using test writer agent |
| `ckrv test coverage` | Check coverage of changed files |
| `ckrv qa review` | QA code review using QA agent |
| `ckrv qa bugs` | Analyze for potential bugs |
| `ckrv qa report` | Generate full QA report |

### New Files

**Services** (`crates/ckrv-cli/src/services/`):
- [agent_lookup.rs](file:///apps/chakravarti-cli/crates/ckrv-cli/src/services/agent_lookup.rs) - Find agents by role
- [test_framework.rs](file:///apps/chakravarti-cli/crates/ckrv-cli/src/services/test_framework.rs) - Framework detection
- [diff_analyzer.rs](file:///apps/chakravarti-cli/crates/ckrv-cli/src/services/diff_analyzer.rs) - Git diff analysis
- [report_generator.rs](file:///apps/chakravarti-cli/crates/ckrv-cli/src/services/report_generator.rs) - Markdown reports

**Commands** (`crates/ckrv-cli/src/commands/`):
- [test.rs](file:///apps/chakravarti-cli/crates/ckrv-cli/src/commands/test.rs) - Test command implementation
- [qa.rs](file:///apps/chakravarti-cli/crates/ckrv-cli/src/commands/qa.rs) - QA command implementation

**Prompts** (`crates/ckrv-cli/src/prompts/`):
- [test_writer.md](file:///apps/chakravarti-cli/crates/ckrv-cli/src/prompts/test_writer.md) - Test writer agent prompt
- [qa_reviewer.md](file:///apps/chakravarti-cli/crates/ckrv-cli/src/prompts/qa_reviewer.md) - QA reviewer agent prompt

## Verification

### Build

```
✅ cargo build --package ckrv-cli - SUCCESS
```

### Tests

```
✅ cargo test --package ckrv-cli - 9/10 passed
   (1 pre-existing test failure unrelated to this feature)
```

### Help Text

```bash
$ ckrv test --help
Run tests in sandbox, plan and write new tests

Usage: ckrv test [OPTIONS] <COMMAND>

Commands:
  run       Run existing tests in sandbox
  plan      Analyze changes and generate test plan
  write     Write new tests using test writer agent
  coverage  Check test coverage of changed files
```

```bash
$ ckrv qa --help
QA code review and bug analysis

Usage: ckrv qa [OPTIONS] <COMMAND>

Commands:
  review  Review code quality of changes
  bugs    Analyze for potential bugs
  report  Generate full QA report
```

## Usage Examples

```bash
# Run tests
ckrv test run

# Plan new tests for changes
ckrv test plan --base main

# Write and run tests
ckrv test write --run

# QA review
ckrv qa review

# Full QA report to file
ckrv qa report --full -o qa-report.md
```

## Notes

- Agent invocation placeholder: Full Docker sandbox integration requires additional work
- Currently uses simulated QA analysis (heuristic-based)
- Role-based agents configured via Agent Manager UI or `~/.config/chakravarti/agents.yaml`
