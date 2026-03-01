---
name: chakravarti-cli
description: Spec-driven agent orchestration. Create specs, plan tasks, run jobs, and review changes.
license: MIT
compatibility: Claude Code, Cursor, any CLI-capable agent
metadata:
  version: "0.1.0"
  auto-generated: true
  generated-at: "2026-03-01T14:11:43Z"
---

# Chakravarti CLI

Command-line interface for Chakravarti

## Commands

### ckrv code

Code workflow commands — mirrors the Code page tabs in the Web UI.

Groups the full development pipeline under a single namespace:
- spec   — create and manage feature specifications
- tasks  — generate implementation tasks from a spec
- plan   — generate an execution plan from tasks
- run    — execute the plan with AI agents
- diff   — review changes before promoting

Use `ckrv code <subcommand> --help` for details on each step.

```bash
ckrv code
```

**Examples**:

# Create a new spec and generate tasks
ckrv code spec new "Add user authentication"
ckrv code tasks

# Plan and run
ckrv code plan
ckrv code run

# Review changes
ckrv code diff

#### ckrv code diff

View changes between current branch and base.

Shows a summary of modified, added, and deleted files compared to the base branch. Helps verify what will be included in a pull request.

Output can be formatted as JSON for programmatic use.

```bash
ckrv code diff [OPTIONS]
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
ckrv code diff

# Show diff against specific branch
ckrv code diff --base main

# Output as JSON
ckrv code diff --json

#### ckrv code plan

Generate execution plan from tasks using AI.

Analyzes the specification and tasks file to create a detailed implementation plan. Runs in a Docker container for isolation.

The plan breaks down work into atomic steps that AI agents can execute.

```bash
ckrv code plan [SPEC] [OPTIONS]
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

# Generate plan for auto-detected spec
ckrv code plan

# Generate plan for a specific spec
ckrv code plan my-feature

# Force regeneration
ckrv code plan --force

#### ckrv code run

Run a job based on a specification.

Executes the implementation plan using AI agents in isolated Docker sandboxes. Each task is executed in sequence with full logging and progress tracking.

Results are committed to a feature branch for review.

```bash
ckrv code run [SPEC] [OPTIONS]
```

**Arguments**:

| Name | Required | Description |
|------|----------|-------------|
| `spec` | No | Path to the specification file. If not provided, will detect from branch name |

**Options**:

| Flag | Description |
|------|-------------|
| `--agent` | Agent to use for execution: claude, codex, or kilo |
| `--cloud` | Execute job in Chakravarti Cloud instead of locally |
| `--credential` | Git credential name to use for cloud execution (for private repos) |
| `--executor-model`, `-e` | Override the AI model/agent to use for execution |
| `--optimize`, `-o` | Optimization strategy |

**Examples**:

# Run all tasks for auto-detected spec
ckrv code run

# Run with specific agent
ckrv code run my-feature --agent claude

# Run with cost optimization
ckrv code run --optimize cost

#### ckrv code spec

Create or manage feature specifications.

Specifications are the source of truth for AI-driven development. They define what needs to be built, the requirements, and acceptance criteria.

Subcommands: new, list, validate, clarify, design, init, tasks

```bash
ckrv code spec
```

**Examples**:

# Create a new specification
ckrv code spec new "Add user authentication"

# List all specifications
ckrv code spec list

# Validate a specification
ckrv code spec validate my-feature

##### ckrv code spec clarify

Resolve open clarifications and ambiguities in an existing specification.

Reviews the spec for unclear requirements, missing details, or
conflicting constraints, then interactively resolves them using AI.
Updates the spec file in-place with the resolved clarifications.

If no spec path is provided, auto-detects the spec from the
current Git branch name.

```bash
ckrv code spec clarify [SPEC]
```

**Arguments**:

| Name | Required | Description |
|------|----------|-------------|
| `spec` | No | Path to the spec file (optional - auto-detects from current branch if not provided) |

**Examples**:

# Clarify the spec detected from the current branch
ckrv spec clarify

# Clarify a specific spec file
ckrv spec clarify specs/auth-oauth2/spec.md

##### ckrv code spec design

Generate a technical design document from an existing specification.

Produces a design.md file alongside the spec containing:
- Architecture decisions and component diagrams
- Data models and API contracts
- Implementation strategy and dependencies

If no spec path is provided, auto-detects the spec from the
current Git branch name. Use --force to regenerate an existing
design document.

```bash
ckrv code spec design [SPEC] [OPTIONS]
```

**Arguments**:

| Name | Required | Description |
|------|----------|-------------|
| `spec` | No | Path to the spec file (optional - auto-detects from current branch if not provided) |

**Options**:

| Flag | Description |
|------|-------------|
| `--force`, `-f` | Force regeneration of design even if it exists |

**Examples**:

# Generate design from the current branch spec
ckrv spec design

# Generate design for a specific spec
ckrv spec design specs/auth-oauth2/spec.md

# Force regeneration of an existing design
ckrv spec design --force

##### ckrv code spec init

Initialize a new, empty specification directory with starter templates.

Creates a named directory under specs/ containing a blank spec.md
template with the standard sections (overview, requirements,
acceptance criteria) ready to be filled in.

Use this when you want to manually author a spec rather than
generating one with AI via `ckrv spec new`.

```bash
ckrv code spec init <NAME>
```

**Arguments**:

| Name | Required | Description |
|------|----------|-------------|
| `name` | Yes | Name for the new spec directory |

**Examples**:

# Initialize a new spec directory
ckrv spec init my-feature

# Initialize with a hyphenated name
ckrv spec init user-auth-oauth2

##### ckrv code spec list

List all specifications found in the specs/ directory.

Displays a table of all spec directories with their names,
statuses, and file paths. Useful for getting an overview of
all features being tracked.

The repository must be initialized with `ckrv init` before
listing specs.

```bash
ckrv code spec list
```

**Examples**:

# List all specs
ckrv spec list

# List specs with JSON output
ckrv spec list --json

##### ckrv code spec new

Create a new feature specification from a natural language description.

Generates a structured spec.md file in the specs/ directory containing:
- Feature overview and goals
- Acceptance criteria
- Technical requirements and constraints
A short name is auto-generated from the description if not provided.

Requires an active AI provider configuration. The AI may ask
clarifying questions if the description is ambiguous.

```bash
ckrv code spec new <DESCRIPTION> [OPTIONS]
```

**Arguments**:

| Name | Required | Description |
|------|----------|-------------|
| `description` | Yes | Natural language description of the feature (e.g., "Add user authentication") |

**Options**:

| Flag | Description |
|------|-------------|
| `--name`, `-n` | Optional short name for the spec (auto-generated from description if not provided) |

**Examples**:

# Create a spec from an inline description
ckrv spec new "Add user authentication with OAuth2"

# Create a spec with an explicit short name
ckrv spec new "Add user authentication" --name auth-oauth2

# Create a spec with a detailed multi-word description
ckrv spec new "Implement rate limiting for the public API endpoints"

##### ckrv code spec tasks

Generate implementation tasks from an existing specification.

Analyzes the spec and produces a tasks.md file containing a set
of discrete, actionable implementation tasks with dependency
ordering. Each task includes a title, description, and prompt
suitable for agent execution.

If no spec path is provided, auto-detects the spec from the
current Git branch name. Use --force to regenerate tasks even
if a tasks file already exists.

```bash
ckrv code spec tasks [SPEC] [OPTIONS]
```

**Arguments**:

| Name | Required | Description |
|------|----------|-------------|
| `spec` | No | Path to the spec file (optional - auto-detects from current branch if not provided) |

**Options**:

| Flag | Description |
|------|-------------|
| `--force`, `-f` | Force regeneration of tasks even if they exist |

**Examples**:

# Generate tasks from the current branch spec
ckrv spec tasks

# Generate tasks for a specific spec
ckrv spec tasks specs/auth-oauth2/spec.md

# Force regeneration of existing tasks
ckrv spec tasks --force

##### ckrv code spec validate

Validate a specification file for correctness and completeness.

Checks that the spec contains all required sections, validates
field formats, and reports any errors or warnings. Returns a
non-zero exit code if validation fails.

If no path is provided, auto-detects the spec from the current
Git branch name. Supports JSON output for CI integration.

```bash
ckrv code spec validate [PATH]
```

**Arguments**:

| Name | Required | Description |
|------|----------|-------------|
| `path` | No | Path to the spec file (optional - auto-detects from current branch if not provided) |

**Examples**:

# Validate the spec detected from the current branch
ckrv spec validate

# Validate a specific spec file
ckrv spec validate specs/auth-oauth2/spec.md

# Validate with JSON output for CI
ckrv spec validate --json

#### ckrv code tasks

Generate implementation tasks from a specification.

Analyzes the specification and produces a structured task breakdown that can be used for planning and execution.

This is a convenience alias for `ckrv code spec tasks`.

```bash
ckrv code tasks [SPEC] [OPTIONS]
```

**Arguments**:

| Name | Required | Description |
|------|----------|-------------|
| `spec` | No | Path to the spec file (optional - auto-detects from current branch if not provided) |

**Options**:

| Flag | Description |
|------|-------------|
| `--force`, `-f` | Force regeneration of tasks even if they exist |

**Examples**:

# Generate tasks for auto-detected spec
ckrv code tasks

# Generate tasks for a specific spec
ckrv code tasks path/to/spec

# Force regeneration
ckrv code tasks --force


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

Analyze changed files for potential bugs and error-handling gaps.

Scans the diff against the base branch and filters findings to only bug-related categories: potential bugs and missing or incorrect error handling. Each finding includes a severity level and a suggested fix.

Requires a configured QA agent. If no agent is found, exits with code 4. Full bug analysis with deeper heuristics requires Docker sandbox integration.

```bash
ckrv qa bugs [OPTIONS]
```

**Options**:

| Flag | Description |
|------|-------------|
| `--base` | Branch to compare against (default: main) |

**Examples**:

# Scan for bugs against the default base branch (main)
ckrv qa bugs

# Scan for bugs against a specific branch
ckrv qa bugs --base develop

# Get machine-readable bug list
ckrv qa bugs --json

#### ckrv qa report

Generate a comprehensive QA report covering all analysis categories.

Produces a Markdown report that includes a change summary, file-level breakdown (when --full is used), and all QA findings ranked by severity. The report header contains branch name, base branch, and timestamp.

Requires a configured QA agent. If no agent is found, exits with code 4. Use --full to include per-file statistics (lines added/removed, change type) in the report.

```bash
ckrv qa report [OPTIONS]
```

**Options**:

| Flag | Description |
|------|-------------|
| `--base` | Branch to compare against (default: main) |
| `--full` | Include all analysis types |
| `--output`, `-o` | Output file path |

**Examples**:

# Generate a standard report against main
ckrv qa report

# Generate a full report with per-file details
ckrv qa report --full

# Save the full report to a file against a custom base
ckrv qa report --full --base develop --output qa-report.md

#### ckrv qa review

Review code quality of changes against a base branch.

Analyzes modified files for code quality issues including style violations, missing documentation, complexity concerns, and best-practice deviations. Produces a structured report with severity-ranked findings.

Results can be saved to a file with --output or printed to stdout. When --json is used, output is a machine-readable QaReviewOutput object.

Requires a configured QA agent. If no agent is found, exits with code 4. Exits with code 1 if critical issues are detected.

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
| `--agent` | Agent to use for execution: claude, codex, kilo, or qwen |
| `--cloud` | Execute job in Chakravarti Cloud instead of locally |
| `--credential` | Git credential name to use for cloud execution (for private repos) |
| `--executor-model`, `-e` | Override the AI model/agent to use for execution |
| `--optimize`, `-o` | Optimization strategy |


**Examples**:

# Review changes against the default base branch (main)
ckrv qa review

# Review changes against a specific branch
ckrv qa review --base develop

# Save review report to a file
ckrv qa review --output qa-review.md


### ckrv term

Spawn an interactive AI agent terminal session.

Quickly launch any configured agent (Claude, OpenRouter, Z.AI, Codex, Kilo Code, Qwen Code) with the correct environment variables automatically configured.

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
| `--cleanup` | Clean up a session (removes worktree and state) |
| `--list`, `-l` | List available agents and exit |
| `--list-sessions` | List all sessions and exit |
| `--name` | Name for this session (enables resume with --resume) |
| `--resume` <STRING> | Resume a session. Optionally pass a session name, or omit to select interactively |
| `--sandbox` | Run agent in a Docker sandbox container |
| `--worktree` | Run agent in an isolated git worktree |

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

Check test coverage of changed files.

Scans files changed between the current branch and base branch to determine which source files have corresponding tests. Reports a coverage percentage based on file-level test presence.

Warns if coverage drops below 80%. Use `ckrv test plan` to see exactly which files need tests.

```bash
ckrv test coverage [OPTIONS]
```

**Options**:

| Flag | Description |
|------|-------------|
| `--base` | Branch to compare against (default: main) |

**Examples**:

# Check coverage against main
ckrv test coverage

# Check coverage against a specific branch
ckrv test coverage --base develop

# Check coverage with JSON output
ckrv test coverage --json

#### ckrv test plan

Analyze changes and generate a test plan.

Compares the current branch against the base branch, identifies changed files, and determines which files lack test coverage. Produces a structured plan with proposed tests prioritized by impact.

The plan is saved to `.specs/<branch>/test-plan.yaml` for use by the test writer agent.

```bash
ckrv test plan [OPTIONS]
```

**Options**:

| Flag | Description |
|------|-------------|
| `--base` | Branch to compare against (default: main) |

**Examples**:

# Generate test plan against main
ckrv test plan

# Generate test plan against a specific branch
ckrv test plan --base develop

# Generate test plan with JSON output
ckrv test plan --json

#### ckrv test run

Run existing tests in a sandboxed environment.

Detects the project's test framework automatically and executes the full test suite. Results are displayed with pass/fail counts and a summary report.

Exits with code 1 if any test fails. Use --json for machine-readable output.

```bash
ckrv test run [OPTIONS]
```

**Options**:

| Flag | Description |
|------|-------------|
| `--base` | Branch to compare against (default: main) |

**Examples**:

# Run tests comparing against main
ckrv test run

# Run tests comparing against a specific branch
ckrv test run --base develop

# Run tests with JSON output
ckrv test run --json

#### ckrv test write

Write new tests using the configured test writer agent.

Analyzes changed files against the base branch and invokes an AI agent to generate tests for uncovered code. The agent runs inside a Docker sandbox for isolation.

Requires a test writer agent to be configured. Use --run to automatically execute the generated tests after writing.

```bash
ckrv test write [OPTIONS]
```

**Options**:

| Flag | Description |
|------|-------------|
| `--base` | Branch to compare against (default: main) |
| `--run` | Run tests after writing |

**Examples**:

# Write tests for changes against main
ckrv test write

# Write tests and run them immediately
ckrv test write --run

# Write tests against a specific branch
ckrv test write --base develop


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


## Global Options

These options apply to all commands:

| Flag | Description |
|------|-------------|
| `--json` | Output format: JSON instead of human-readable |
| `--quiet, -q` | Suppress non-essential output |
| `--verbose, -v` | Enable verbose logging |
| `--help, -h` | Print help |
| `--version, -V` | Print version |
