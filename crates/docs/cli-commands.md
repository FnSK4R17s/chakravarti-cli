---
last_commit: f92f604
last_updated: 2026-03-01
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
| `-v, --verbose` | Enable verbose logging |
| `-q, --quiet` | Suppress non-essential output |
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

### `code`

Code workflow commands — mirrors the Code page tabs in the Web UI.

```bash
ckrv code <SUBCOMMAND>
```

**Subcommands:**
- `spec`: Create and manage feature specifications
- `tasks`: Generate implementation tasks (alias for `spec tasks`)
- `plan`: Generate execution plan from tasks (in Docker)
- `run`: Execute a job based on a specification
- `diff`: View changes between current branch and base

#### `code spec`

```bash
ckrv code spec <SUBCOMMAND>
```

Subcommands: `new`, `clarify`, `design`, `init`, `tasks`, `validate`, `list`

#### `code tasks`

```bash
ckrv code tasks [SPEC] [--force]
```

Convenience alias for `ckrv code spec tasks`.

#### `code plan`

```bash
ckrv code plan [SPEC] [--force]
```

#### `code run`

```bash
ckrv code run [SPEC] [OPTIONS]
```

**Options:**
- `--agent`: Agent to use for execution (claude, codex, kilo, or factory)
- `--cloud`: Execute job in Chakravarti Cloud
- `--credential`: Git credential name for cloud execution
- `--executor-model, -e`: Override the AI model/agent
- `--optimize, -o`: Optimization strategy (default: balanced)

#### `code diff`

```bash
ckrv code diff [OPTIONS]
```

**Options:**
- `--base, -b <branch>`: Compare against specific branch (default: main)
- `--color`: Color mode for diff output (default: auto)
- `--files`: List changed files only
- `--stat`: Show file statistics only
- `--summary`: Generate AI summary of changes

**Examples:**
```bash
ckrv code spec new "Add user authentication with OAuth2"
ckrv code tasks
ckrv code plan
ckrv code run --agent claude
ckrv code diff
```

**Exit codes:**
- `0`: Success / All tasks succeeded
- `1`: One or more tasks failed
- `2`: User cancelled

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

### `term`

Spawn an interactive AI agent terminal session with optional isolation modes.

```bash
ckrv term [OPTIONS] [-- ARGS...]
```

Quickly launch any configured agent (Claude, OpenRouter, Z.AI, Codex, Kilo Code)
with the correct environment variables automatically configured.

**Isolation Modes:**
- **Default**: Agent runs directly in the current working directory
- **`--worktree`**: Agent runs in an isolated git worktree on a separate branch. After the session, you can view diffs, merge changes, keep for later, or discard.
- **`--sandbox`**: Agent runs inside a Docker container with credential mounts. Changes are isolated to the container filesystem.
- **`--sandbox --worktree`**: Maximum isolation -- worktree for code, container for execution.

**Session Management:**
Use `--name` to create named sessions that can be resumed later with `--resume`.
Session state is stored in `.chakravarti/sessions/<name>.yaml`.

**Options:**
- `--agent, -a <ID>`: Agent ID to spawn directly (skips interactive selection)
- `--list, -l`: List available agents and exit
- `--worktree`: Run agent in an isolated git worktree
- `--sandbox`: Run agent in a Docker sandbox container
- `--name <NAME>`: Name for this session (enables resume with `--resume`)
- `--resume [NAME]`: Resume a session (omit name to select interactively)
- `--list-sessions`: List all sessions and exit
- `--cleanup <NAME>`: Clean up a session (removes worktree and state)
- `--json`: Output in JSON format (for `--list` and `--list-sessions`)

**Examples:**
```bash
ckrv term                                  # Interactive selection
ckrv term --agent my-openrouter-agent      # Direct agent spawn
ckrv term --worktree                       # Isolated worktree mode
ckrv term --sandbox                        # Docker sandbox mode
ckrv term --sandbox --worktree             # Maximum isolation
ckrv term --worktree --name fix-auth       # Named session for resume
ckrv term --resume fix-auth                # Resume a named session
ckrv term --list-sessions                  # List all sessions
ckrv term --cleanup fix-auth               # Remove a session
ckrv term -- --dangerously-skip-permissions --continue  # Pass args through
ckrv term --list                           # List agents
```

---

### `ui`

Launch the Web UI dashboard.

```bash
ckrv ui [OPTIONS]
```

**Options:**
- `--port <N>`: Server port (default: 3000)

---

## Hidden Commands

The following commands exist but are hidden from `ckrv --help`. They are still functional.

**Visible commands** (shown in `ckrv --help`): `init`, `code`, `test`, `qa`, `term`, `ui` (6 total).

### Legacy Top-Level Aliases

Prefer `ckrv code <subcommand>` instead:

- `spec`: Legacy top-level form -- use `ckrv code spec` instead
- `plan`: Legacy top-level form -- use `ckrv code plan` instead
- `run`: Legacy top-level form -- use `ckrv code run` instead
- `diff`: Legacy top-level form -- use `ckrv code diff` instead

### `verify` (hidden)

Run tests, lint, and quality checks.

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

### `fix` (hidden)

Fix verification errors with AI.

```bash
ckrv fix [OPTIONS]
```

**Options:**
- `--lint`: Fix lint errors only
- `--test`: Fix test failures only
- `--type`: Fix type errors only
- `--check`: Re-run verification after fix
- `--error`: Specific error message to fix (from UI)

### `promote` (hidden)

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

### `task` (hidden)

Execute a workflow-based agent task.

```bash
ckrv task <TARGET> [OPTIONS]
```

**Options:**
- `--workflow, -w <NAME>`: Workflow to use (default: swe)
- `--dry-run`: Show plan without executing
- `--continue, -c <ID>`: Continue a previous task by ID
- `--agent`: Agent tool to use (default: claude)
- `--no-sandbox`: Skip Docker sandbox and run agent locally

### `status` (hidden)

Check the status of a job.

```bash
ckrv status <JOB_ID>
```

### `report` (hidden)

View the metrics report for a job.

```bash
ckrv report <JOB_ID> [OPTIONS]
```

**Options:**
- `--detailed`: Show per-step breakdown

### `cloud` (hidden)

Cloud execution commands.

```bash
ckrv cloud <SUBCOMMAND>
```

**Subcommands:**
- `login`: Authenticate with Chakravarti Cloud
- `logout`: Clear stored credentials
- `whoami`: Display current authenticated user
- `credentials`: Manage git credentials for private repos

### `logs` (hidden)

Stream or view logs from a cloud job.

```bash
ckrv logs <JOB_ID> [OPTIONS]
```

**Options:**
- `--follow, -f`: Stream logs in real-time
- `--json`: Output as JSON
- `--tail, -n <N>`: Number of recent lines (default: 100)

### `pull` (hidden)

Pull results from a completed cloud job.

```bash
ckrv pull <JOB_ID> [OPTIONS]
```

**Options:**
- `--apply`: Apply diff to current worktree (default: true)
- `--output`: Output diff to file instead of applying

## Exit Codes

| Code | Meaning |
|------|---------|
| `0` | Success |
| `1` | General failure |
| `2` | User cancellation |
| `3` | Configuration error |
| `4` | Dependency missing |
