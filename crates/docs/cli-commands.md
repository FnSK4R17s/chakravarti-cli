---
last_commit: 508766e
last_updated: 2026-02-15
related_files:
  - crates/ckrv-cli/src/lib.rs
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
- `clarify`: Resolve clarifications in an existing spec
- `design`: Generate technical design document
- `init <name>`: Initialize an empty spec directory with templates
- `tasks`: Generate implementation tasks
- `validate`: Validate a specification file
- `list`: List all specs

**Examples:**
```bash
ckrv spec new "Add user authentication with OAuth2"
ckrv spec tasks
ckrv spec design
```

---

### `plan`

Generate execution plan from tasks using AI (runs in Docker).

```bash
ckrv plan [OPTIONS] [SPEC]
```

**Options:**
- `--force, -f`: Force regeneration even if plan.yaml exists

---

### `run`

Execute the orchestration engine.

```bash
ckrv run [OPTIONS] [SPEC]
```

**Options:**
- `--agent`: Agent to use for execution (claude, codex, kilo, or opencode)
- `--cloud`: Execute job in Chakravarti Cloud
- `--credential`: Git credential name for cloud execution
- `--executor-model, -e`: Override the AI model/agent
- `--optimize, -o`: Optimization strategy (default: balanced)

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
- `--base, -b <branch>`: Compare against specific branch (default: main)
- `--color`: Color mode for diff output (default: auto)
- `--files`: List changed files only
- `--stat`: Show file statistics only
- `--summary`: Generate AI summary of changes

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
- `--type`: Run type checking only
- `--fix`: Auto-fix linting issues
- `--save`: Save results to verification.yaml
- `--continue-on-failure`: Run all checks even if some fail

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
- `--test`: Fix test failures only
- `--type`: Fix type errors only
- `--check`: Re-run verification after fix
- `--error`: Specific error message to fix (from UI)

---

### `promote`

Push changes and create Pull Request.

```bash
ckrv promote [OPTIONS]
```

**Options:**
- `--base, -b <branch>`: Target branch (default: main)
- `--draft`: Create as draft PR
- `--open`: Open PR URL in browser
- `--push`: Push to remote first
- `--remote`: Remote name (default: origin)
- `--skip-verify`: Skip verification checks

---

### `term`

Spawn an interactive AI agent terminal session.

```bash
ckrv term [OPTIONS] [-- ARGS...]
```

**Options:**
- `--agent, -a`: Agent ID to spawn directly (skips selection)
- `--list, -l`: List available agents and exit

**Examples:**
```bash
ckrv term                              # Interactive selection
ckrv term --agent my-openrouter-agent  # Direct agent spawn
ckrv term -- --dangerously-skip-permissions  # Pass args through
ckrv term --list                       # List agents
```

---

### `status`

Show current workflow status (hidden command).

```bash
ckrv status [OPTIONS]
```

**Options:**
- `--json`: Output in JSON format

---

### `logs`

Stream or view logs from a cloud job.

```bash
ckrv logs <JOB_ID> [OPTIONS]
```

**Options:**
- `--follow, -f`: Stream logs in real-time
- `--json`: Output as JSON
- `--tail, -n <N>`: Number of recent lines (default: 100)

---

### `pull`

Pull results from a completed cloud job.

```bash
ckrv pull <JOB_ID> [OPTIONS]
```

**Options:**
- `--apply`: Apply diff to current worktree (default: true)
- `--output`: Output diff to file instead of applying

---

### `ui`

Launch the Web UI dashboard.

```bash
ckrv ui [OPTIONS]
```

**Options:**
- `--port <N>`: Server port (default: 3000)

---

### `cloud`

Cloud execution commands.

```bash
ckrv cloud <SUBCOMMAND>
```

**Subcommands:**
- `login`: Authenticate with Chakravarti Cloud
- `logout`: Clear stored credentials
- `whoami`: Display current authenticated user
- `credentials`: Manage git credentials for private repos

---

### Hidden Commands

The following commands exist but are hidden from `--help`:
- `task`: Manage individual tasks
- `status`: Show workflow status
- `report`: Generate execution report

## Exit Codes

| Code | Meaning |
|------|---------|
| `0` | Success |
| `1` | General failure |
| `2` | User cancellation |
| `3` | Configuration error |
| `4` | Dependency missing |
