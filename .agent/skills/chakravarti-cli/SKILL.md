---
name: chakravarti-cli
description: Spec-driven agent orchestration. Create specs, plan tasks, run jobs, and review changes.
license: MIT
compatibility: Claude Code, Cursor, any CLI-capable agent
metadata:
  version: "0.1.0"
  auto-generated: true
  generated-at: "2026-01-30T04:08:04Z"
---

# Chakravarti CLI

Command-line interface for Chakravarti

## Commands

### ckrv cloud

Cloud execution commands

```bash
ckrv cloud
```

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

View changes between current branch and base

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


### ckrv fix

Fix verification errors with AI

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


### ckrv init

Initialize Chakravarti in the current repository.

Creates a `.chakravarti/` directory with default configuration and a `specs/` directory for feature specifications. The configuration file at `.chakravarti/config.yaml` controls agent settings, workflows, and verification rules.

This command must be run inside a Git repository. Use `--force` to re-initialize an existing Chakravarti project.

```bash
ckrv init [OPTIONS]
```

**Options**:

| Flag | Description |
|------|-------------|
| `--force` | Force reinitialization even if already initialized |

**Examples**:

Examples:
# Basic initialization
ckrv init

# Force re-initialization (overwrites existing config)
ckrv init --force

# Initialize with JSON output for scripting
ckrv --json init


### ckrv logs

Stream or view logs from a cloud job

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


### ckrv plan

Generate execution plan from tasks (in Docker)

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


### ckrv promote

Create a pull request for the current branch

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


### ckrv pull

Pull results from a completed cloud job

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


### ckrv qa

QA code review and bug analysis

```bash
ckrv qa
```

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

Run a job based on a specification

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
| `--agent` | Agent to use for execution: claude or codex |
| `--cloud` | Execute job in Chakravarti Cloud instead of locally |
| `--credential` | Git credential name to use for cloud execution (for private repos) |
| `--executor-model`, `-e` | Override the AI model/agent to use for execution |
| `--optimize`, `-o` | Optimization strategy |


### ckrv spec

Create or manage feature specifications

```bash
ckrv spec
```

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


### ckrv test

Run tests in sandbox, plan and write new tests

```bash
ckrv test
```

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

Start the Web UI dashboard

```bash
ckrv ui [OPTIONS]
```

**Options**:

| Flag | Description |
|------|-------------|
| `--port` | Port to listen on (default: 3000) |


### ckrv verify

Run tests, lint, and quality checks

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


## Global Options

These options apply to all commands:

| Flag | Description |
|------|-------------|
| `--json` | Output format: JSON instead of human-readable |
| `--quiet, -q` | Suppress non-essential output |
| `--verbose, -v` | Enable verbose logging |
| `--help, -h` | Print help |
| `--version, -V` | Print version |
