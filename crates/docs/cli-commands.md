---
last_commit: e74f093
last_updated: 2026-01-29
related_files:
  - crates/ckrv-cli/src/commands/mod.rs
  - crates/ckrv-cli/src/main.rs
---

# CLI Command Reference

## Overview

Chakravarti CLI (`ckrv`) provides commands for spec-driven development workflow.

```bash
ckrv [OPTIONS] <COMMAND>
```

## Global Options

| Option | Description |
|--------|-------------|
| `-h, --help` | Print help information |
| `-V, --version` | Print version |
| `--verbose` | Enable verbose logging |
| `--json` | Output in JSON format |

## Commands

### `init`

Initialize Chakravarti in a repository.

```bash
ckrv init [OPTIONS]
```

**Options:**
- `--force`: Overwrite existing configuration

**Exit codes:**
- `0`: Success
- `1`: Already initialized (without --force)

---

### `spec`

Manage feature specifications.

```bash
ckrv spec <SUBCOMMAND>
```

**Subcommands:**
- `new <description>`: Create new spec from description
- `list`: List all specs
- `show`: Display current spec
- `tasks`: Generate implementation tasks

**Examples:**
```bash
ckrv spec new "Add user authentication with OAuth2"
ckrv spec tasks
```

---

### `plan`

Generate or view execution plans.

```bash
ckrv plan [OPTIONS]
```

**Options:**
- `--regenerate`: Force regeneration of plan
- `--show`: Display current plan

---

### `run`

Execute the orchestration engine.

```bash
ckrv run [OPTIONS]
```

**Options:**
- `--batch <N>`: Run specific batch only
- `--parallel <N>`: Max parallel agents (default: 3)
- `--dry-run`: Preview without execution
- `--no-merge`: Skip auto-merge step

**Exit codes:**
- `0`: All tasks succeeded
- `1`: One or more tasks failed
- `2`: User cancelled

---

### `diff`

View changes between branches.

```bash
ckrv diff [OPTIONS]
```

**Options:**
- `--stat`: Show file statistics only
- `--files`: List changed files only
- `--base <branch>`: Compare against specific branch (default: main)

---

### `verify`

Run code quality checks.

```bash
ckrv verify [OPTIONS]
```

> **Note:** This command runs shell commands (e.g., `cargo test`, `cargo clippy`) directly, not via the `ckrv-verify` crate.

**Options:**
- `--lint`: Run linting only
- `--test`: Run tests only
- `--typecheck`: Run type checking only
- `--fix`: Auto-fix linting issues
- `--save`: Save results to verification.yaml

**Exit codes:**
- `0`: All checks passed
- `1`: One or more checks failed

---

### `test`

Run tests and manage test coverage using role-based agents.

```bash
ckrv test <SUBCOMMAND>
```

**Subcommands:**
- `run`: Run existing tests in sandbox
- `plan`: Analyze changes and generate test plan
- `write`: Write new tests using test writer agent
- `coverage`: Check test coverage of changed files

**Options (all subcommands):**
- `--base <branch>`: Branch to compare against (default: main)
- `--json`: Output in JSON format

**Write subcommand options:**
- `--run`: Run tests after writing

**Examples:**
```bash
ckrv test run                    # Run project tests
ckrv test plan                   # Analyze what needs tests
ckrv test write --run            # Write and run new tests
ckrv test coverage --base develop # Check coverage vs develop
```

**Exit codes:**
- `0`: All tests passed
- `1`: Tests failed
- `4`: No test writer agent configured

---

### `qa`

QA code review and bug analysis using role-based agents.

```bash
ckrv qa <SUBCOMMAND>
```

**Subcommands:**
- `review`: Review code quality of changes
- `bugs`: Analyze for potential bugs
- `report`: Generate full QA report

**Options:**
- `--base <branch>`: Branch to compare against (default: main)
- `--output <file>`: Save report to file
- `--json`: Output in JSON format

**Report subcommand options:**
- `--full`: Include all analysis types (quality, bugs, security)

**Examples:**
```bash
ckrv qa review                    # Review changes vs main
ckrv qa bugs --base develop       # Find bugs vs develop
ckrv qa report --full -o qa.md    # Full report to file
```

**Exit codes:**
- `0`: No critical issues
- `1`: Critical issues found
- `4`: No QA agent configured

---

### `fix`

Use AI to fix verification errors.

```bash
ckrv fix [OPTIONS]
```

**Options:**
- `--lint`: Fix lint errors only
- `--check`: Re-run verification after fix

---

### `promote`

Push changes and create Pull Request.

```bash
ckrv promote [OPTIONS]
```

**Options:**
- `--push`: Push to remote first
- `--draft`: Create as draft PR
- `--open`: Open PR URL in browser
- `--base <branch>`: Target branch (default: main)

---

### `status`

Show current workflow status.

```bash
ckrv status [OPTIONS]
```

**Options:**
- `--json`: Output in JSON format

---

### `logs`

View execution logs.

```bash
ckrv logs [OPTIONS]
```

**Options:**
- `--follow`: Stream logs in real-time
- `--batch <N>`: Show logs for specific batch

---

### `report`

Generate execution report.

```bash
ckrv report [OPTIONS]
```

**Options:**
- `--format <fmt>`: Output format (markdown, json, html)

---

### `task`

Manage individual tasks.

```bash
ckrv task <SUBCOMMAND>
```

**Subcommands:**
- `list`: List all tasks
- `show <id>`: Show task details
- `retry <id>`: Retry failed task

---

### `pull`

Pull changes from remote.

```bash
ckrv pull
```

---

### `ui`

Launch the Web UI dashboard.

```bash
ckrv ui [OPTIONS]
```

**Options:**
- `--port <N>`: Server port (default: 3000)
- `--open`: Open in browser

---

### `cloud`

Cloud execution commands.

```bash
ckrv cloud <SUBCOMMAND>
```

**Subcommands:**
- `auth`: Authenticate with cloud
- `sync`: Sync local state to cloud
- `status`: Show cloud job status

## Exit Codes

| Code | Meaning |
|------|---------|
| `0` | Success |
| `1` | General failure |
| `2` | User cancellation |
| `3` | Configuration error |
| `4` | Dependency missing |
