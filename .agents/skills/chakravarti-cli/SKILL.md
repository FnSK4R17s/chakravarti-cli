---
name: chakravarti-cli
description: Spec-driven agent orchestration. Create specs, plan tasks, run jobs, and review changes.
license: MIT
compatibility: Claude Code, Cursor, any CLI-capable agent
metadata:
  version: "0.1.0"
  auto-generated: true
  generated-at: "2026-02-15T14:54:20Z"
---

# Chakravarti CLI

Command-line interface for Chakravarti

## Commands

### ckrv cloud

Cloud execution commands.

Manage remote job execution via Chakravarti Cloud. Submit jobs, monitor progress, and retrieve results from cloud workers.

Subcommands: login, submit, status, cancel

```bash
ckrv cloud
```

**Examples**:

# Login to cloud
ckrv cloud login

# Submit a job
ckrv cloud submit my-feature

# Check job status
ckrv cloud status <job-id>

#### ckrv cloud credentials

Manage git credentials for private repositories

```bash
ckrv cloud credentials
```

##### ckrv cloud credentials add

Add a new git credential

```bash
ckrv cloud credentials add [OPTIONS]
```

**Options**:

| Flag | Description |
|------|-------------|
| `--credential-type` | Credential type (pat, deploy_key) |
| `--name` | Name for this credential (e.g., "github-work") |
| `--provider` | Git provider (github, gitlab, bitbucket, generic) |

##### ckrv cloud credentials list

List stored credentials

```bash
ckrv cloud credentials list
```

##### ckrv cloud credentials remove

Remove a credential

```bash
ckrv cloud credentials remove <NAME>
```

**Arguments**:

| Name | Required | Description |
|------|----------|-------------|
| `name` | Yes | Name of the credential to remove |

#### ckrv cloud login

Authenticate with Chakravarti Cloud

```bash
ckrv cloud login [OPTIONS]
```

**Options**:

| Flag | Description |
|------|-------------|
| `--no-browser` | Skip opening browser automatically |

#### ckrv cloud logout

Clear stored cloud credentials

```bash
ckrv cloud logout [OPTIONS]
```

**Options**:

| Flag | Description |
|------|-------------|
| `--force`, `-f` | Force logout without confirmation |

#### ckrv cloud whoami

Display current authenticated user

```bash
ckrv cloud whoami
```


### ckrv diff

View changes between current branch and base.

Shows a summary of modified, added, and deleted files compared to the base branch. Helps verify what will be included in a pull request.

Output can be formatted as JSON for programmatic use.

```bash
ckrv diff [OPTIONS]
```

**Options**:

| Flag | Description |
|------|-------------|
| `--base`, `-b` | Base branch to compare against (default: main or master) |
| `--color` | Color mode for diff output |
| `--files` | Show file list only |
| `--stat` | Show diff statistics only |
| `--summary` | Generate AI summary of changes |

**Examples**:

# Show diff summary
ckrv diff

# Show diff against specific branch
ckrv diff --base main

# Output as JSON
ckrv diff --json


### ckrv fix

Fix verification errors with AI.

Analyzes failed tests, lint errors, or build issues and uses AI to automatically generate fixes. Runs in an isolated Docker sandbox.

Best used after `ckrv verify` identifies issues.

```bash
ckrv fix [OPTIONS]
```

**Options**:

| Flag | Description |
|------|-------------|
| `--check` | Re-run verification after fixing |
| `--error` | Specific error message to fix (from UI) |
| `--lint` | Fix only lint errors |
| `--test` | Fix only test failures |
| `--typecheck` | Fix only type errors |

**Examples**:

# Fix all errors
ckrv fix

# Fix with specific agent
ckrv fix --agent claude-3.5

# Fix only test failures
ckrv fix --tests-only


### ckrv init

Initialize Chakravarti in the current repository.

Creates the `.chakravarti/` directory with default configuration files including `config.yaml` for project settings and initializes the specs directory.

This is typically the first command to run when setting up a new project for AI-driven development with Chakravarti.

```bash
ckrv init [OPTIONS]
```

**Options**:

| Flag | Description |
|------|-------------|
| `--force` | Force reinitialization even if already initialized |

**Examples**:

# Initialize in current directory
ckrv init

# Initialize with verbose output
ckrv init --verbose


### ckrv logs

Stream or view logs from a cloud job.

Shows real-time output from running jobs or historical logs from completed jobs. Supports filtering by task or agent.

Use --follow for continuous streaming.

```bash
ckrv logs <JOB_ID> [OPTIONS]
```

**Arguments**:

| Name | Required | Description |
|------|----------|-------------|
| `job_id` | Yes | Job ID to get logs for |

**Options**:

| Flag | Description |
|------|-------------|
| `--follow`, `-f` | Follow log output (stream in real-time) |
| `--tail`, `-n` | Number of recent log lines to show (default: 100) |

**Examples**:

# Stream logs from running job
ckrv logs <job-id> --follow

# View completed job logs
ckrv logs <job-id>

# Filter by task
ckrv logs <job-id> --task 3


### ckrv plan

Generate execution plan from tasks using AI.

Analyzes the specification and tasks file to create a detailed implementation plan. Runs in a Docker container for isolation.

The plan breaks down work into atomic steps that AI agents can execute.

```bash
ckrv plan [SPEC] [OPTIONS]
```

**Arguments**:

| Name | Required | Description |
|------|----------|-------------|
| `spec` | No | Path to the specification directory. If not provided, will detect from branch name |

**Options**:

| Flag | Description |
|------|-------------|
| `--force`, `-f` | Force regeneration even if plan.yaml already exists |

**Examples**:

# Generate plan for a specification
ckrv plan my-feature

# Generate plan with GLM model
ckrv plan my-feature --model glm-4.7

# Skip confirmation prompt
ckrv plan my-feature --yes


### ckrv promote

Create a pull request for the current branch.

Pushes the feature branch and creates a pull request on GitHub/GitLab. Auto-generates PR title and description from the specification.

Requires remote repository access and appropriate permissions.

```bash
ckrv promote [OPTIONS]
```

**Options**:

| Flag | Description |
|------|-------------|
| `--base`, `-b` | Target branch for the PR (default: main or master) |
| `--draft` | Create as draft PR |
| `--open` | Open PR URL in browser after creation |
| `--push` | Push branch to remote before creating PR |
| `--remote` | Remote name (default: origin) |
| `--skip-verify` | Skip verification checks |

**Examples**:

# Create PR with auto-generated description
ckrv promote

# Create as draft PR
ckrv promote --draft

# Create PR with custom title
ckrv promote --title "feat: add user auth"


### ckrv pull

Pull results from a completed cloud job.

Downloads all changes made during cloud execution and applies them to the local repository. Creates or updates the feature branch.

Jobs must be in a 'completed' state to pull.

```bash
ckrv pull <JOB_ID> [OPTIONS]
```

**Arguments**:

| Name | Required | Description |
|------|----------|-------------|
| `job_id` | Yes | Job ID to pull results from |

**Options**:

| Flag | Description |
|------|-------------|
| `--apply` | Apply diff to current worktree (default: true) |
| `--output` | Output diff to file instead of applying |

**Examples**:

# Pull results to current directory
ckrv pull <job-id>

# Pull and create new branch
ckrv pull <job-id> --branch feature/new


### ckrv qa

QA code review and bug analysis.

AI-powered code review and quality assurance. Analyzes changes for potential bugs, security issues, and code quality improvements.

Subcommands: review, bugs, report

```bash
ckrv qa
```

**Examples**:

# Review current changes
ckrv qa review

# Analyze for bugs
ckrv qa bugs

# Generate QA report
ckrv qa report

#### ckrv qa bugs

Analyze for potential bugs

```bash
ckrv qa bugs [OPTIONS]
```

**Options**:

| Flag | Description |
|------|-------------|
| `--base` | Branch to compare against (default: main) |

#### ckrv qa report

Generate full QA report

```bash
ckrv qa report [OPTIONS]
```

**Options**:

| Flag | Description |
|------|-------------|
| `--base` | Branch to compare against (default: main) |
| `--full` | Include all analysis types |
| `--output`, `-o` | Output file path |

#### ckrv qa review

Review code quality of changes

```bash
ckrv qa review [OPTIONS]
```

**Options**:

| Flag | Description |
|------|-------------|
| `--base` | Branch to compare against (default: main) |
| `--output`, `-o` | Output file path |


### ckrv run

Run a job based on a specification.

Executes the implementation plan using AI agents in isolated Docker sandboxes. Each task is executed in sequence with full logging and progress tracking.

Results are committed to a feature branch for review.

```bash
ckrv run [SPEC] [OPTIONS]
```

**Arguments**:

| Name | Required | Description |
|------|----------|-------------|
| `spec` | No | Path to the specification file. If not provided, will detect from branch name |

**Options**:

| Flag | Description |
|------|-------------|
| `--agent` | Agent to use for execution: claude, codex, kilo, or factory |
| `--cloud` | Execute job in Chakravarti Cloud instead of locally |
| `--credential` | Git credential name to use for cloud execution (for private repos) |
| `--executor-model`, `-e` | Override the AI model/agent to use for execution |
| `--optimize`, `-o` | Optimization strategy |

**Examples**:

# Run all tasks for a specification
ckrv run my-feature

# Run with specific agent
ckrv run my-feature --agent claude-3.5

# Dry run (show what would be done)
ckrv run my-feature --dry-run


### ckrv spec

Create or manage feature specifications.

Specifications are the source of truth for AI-driven development. They define what needs to be built, the requirements, and acceptance criteria.

Subcommands: new, list, validate, edit, show

```bash
ckrv spec
```

**Examples**:

# Create a new specification
ckrv spec new "Add user authentication"

# List all specifications
ckrv spec list

# Validate a specification
ckrv spec validate my-feature

#### ckrv spec clarify

Resolve clarifications in an existing spec

```bash
ckrv spec clarify [SPEC]
```

**Arguments**:

| Name | Required | Description |
|------|----------|-------------|
| `spec` | No | Path to the spec file (optional - auto-detects from current branch if not provided) |

#### ckrv spec design

Generate technical design document from a specification

```bash
ckrv spec design [SPEC] [OPTIONS]
```

**Arguments**:

| Name | Required | Description |
|------|----------|-------------|
| `spec` | No | Path to the spec file (optional - auto-detects from current branch if not provided) |

**Options**:

| Flag | Description |
|------|-------------|
| `--force`, `-f` | Force regeneration of design even if it exists |

#### ckrv spec init

Initialize an empty spec directory with templates

```bash
ckrv spec init <NAME>
```

**Arguments**:

| Name | Required | Description |
|------|----------|-------------|
| `name` | Yes | Name for the new spec directory |

#### ckrv spec list

List all specifications

```bash
ckrv spec list
```

#### ckrv spec new

Create a new specification using AI from a natural language description

```bash
ckrv spec new <DESCRIPTION> [OPTIONS]
```

**Arguments**:

| Name | Required | Description |
|------|----------|-------------|
| `description` | Yes | Natural language description of the feature (e.g., "Add user authentication") |

**Options**:

| Flag | Description |
|------|-------------|
| `--name`, `-n` | Optional short name for the spec (auto-generated from description if not provided) |

#### ckrv spec tasks

Generate implementation tasks from a specification

```bash
ckrv spec tasks [SPEC] [OPTIONS]
```

**Arguments**:

| Name | Required | Description |
|------|----------|-------------|
| `spec` | No | Path to the spec file (optional - auto-detects from current branch if not provided) |

**Options**:

| Flag | Description |
|------|-------------|
| `--force`, `-f` | Force regeneration of tasks even if they exist |

#### ckrv spec validate

Validate a specification file

```bash
ckrv spec validate [PATH]
```

**Arguments**:

| Name | Required | Description |
|------|----------|-------------|
| `path` | No | Path to the spec file (optional - auto-detects from current branch if not provided) |


### ckrv term

Spawn an interactive AI agent terminal session.

Quickly launch any configured agent (Claude, OpenRouter, Z.AI, Codex, Kilo Code, Factory Droid) with the correct environment variables automatically configured.

Without arguments, presents an interactive selection menu with options for common flags. Use -- to pass arguments directly for scripting.

```bash
ckrv term [PASSTHROUGH_ARGS] [OPTIONS]
```

**Arguments**:

| Name | Required | Description |
|------|----------|-------------|
| `passthrough_args` | No | Additional arguments to pass to the agent binary |

**Options**:

| Flag | Description |
|------|-------------|
| `--agent`, `-a` | Agent ID to spawn directly (skips interactive agent selection) |
| `--list`, `-l` | List available agents and exit |

**Examples**:

# Interactive selection with options prompt
ckrv term

# Launch specific agent (skips agent selection)
ckrv term --agent my-openrouter-agent

# Pass flags directly (scripting)
ckrv term -- --dangerously-skip-permissions --continue

# List available agents
ckrv term --list


### ckrv test

Run tests in sandbox, plan and write new tests.

Comprehensive test management with AI assistance. Can run existing tests, analyze coverage gaps, and generate new tests using AI agents.

Subcommands: run, plan, write

```bash
ckrv test
```

**Examples**:

# Run all tests
ckrv test run

# Plan tests for uncovered code
ckrv test plan

# Write new tests with AI
ckrv test write --agent claude-3.5

#### ckrv test coverage

Check test coverage of changed files

```bash
ckrv test coverage [OPTIONS]
```

**Options**:

| Flag | Description |
|------|-------------|
| `--base` | Branch to compare against (default: main) |

#### ckrv test plan

Analyze changes and generate test plan

```bash
ckrv test plan [OPTIONS]
```

**Options**:

| Flag | Description |
|------|-------------|
| `--base` | Branch to compare against (default: main) |

#### ckrv test run

Run existing tests in sandbox

```bash
ckrv test run [OPTIONS]
```

**Options**:

| Flag | Description |
|------|-------------|
| `--base` | Branch to compare against (default: main) |

#### ckrv test write

Write new tests using test writer agent

```bash
ckrv test write [OPTIONS]
```

**Options**:

| Flag | Description |
|------|-------------|
| `--base` | Branch to compare against (default: main) |
| `--run` | Run tests after writing |


### ckrv ui

Start the Web UI dashboard.

Launches a local web server providing a visual interface for managing specifications, viewing execution progress, and reviewing AI agent output.

Opens automatically in your default browser.

```bash
ckrv ui [OPTIONS]
```

**Options**:

| Flag | Description |
|------|-------------|
| `--port` | Port to listen on (default: 3000) |

**Examples**:

# Start UI on default port
ckrv ui

# Start on custom port
ckrv ui --port 8080

# Don't open browser automatically
ckrv ui --no-open


### ckrv verify

Run tests, lint, and quality checks.

Validates the current code against project quality standards. Runs the test suite, linters, and any custom verification scripts.

Failed verifications can be fixed with `ckrv fix`.

```bash
ckrv verify [OPTIONS]
```

**Options**:

| Flag | Description |
|------|-------------|
| `--continue-on-failure` | Continue on failure (run all checks even if some fail) |
| `--fix` | Auto-fix issues where possible |
| `--lint` | Run only lint checks |
| `--save` | Save results to verification.yaml |
| `--test` | Run only tests |
| `--typecheck` | Run only type checks |

**Examples**:

# Run all verifications
ckrv verify

# Run only tests
ckrv verify --tests-only

# Run in JSON output mode
ckrv verify --json


## Global Options

These options apply to all commands:

| Flag | Description |
|------|-------------|
| `--json` | Output format: JSON instead of human-readable |
| `--quiet, -q` | Suppress non-essential output |
| `--verbose, -v` | Enable verbose logging |
| `--help, -h` | Print help |
| `--version, -V` | Print version |
