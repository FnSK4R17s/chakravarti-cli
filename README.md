# Chakravarti CLI

> Spec-driven Agent Orchestration Engine

[![Rust](https://img.shields.io/badge/rust-1.75+-orange.svg)](https://www.rust-lang.org/)
[![License](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)

**Code like an Architect, not a Typist.**

Chakravarti (`ckrv`) is an autonomous coding engine that turns high-level specifications into shipping code. It orchestrates AI agents to plan, implement, and verify features in parallel, using isolated Git worktrees and Docker sandboxes to ensure safety and code integrity.

## Philosophy

Most AI coding tools are "autocomplete on steroids" or "single-file refactors." Chakravarti is an **Orchestrator**.

1.  **Design First**: You define the *What* and *Why* in a Specification (`spec.yaml`).
2.  **AI Planning**: The system analyzes your codebase and breaks the spec into actionable, dependency-aware batches.
3.  **Parallel Execution**: Agents execute tasks in parallel using isolated **Git Worktrees**. No working directory pollution.
4.  **Auto-Integration**: Completed code is automatically committed, merged, and propagated to dependent tasks.
5.  **Verification**: Every step runs in a Docker sandbox to ensure tests pass before merging.

## Quick Start

### 1. Initialize
Set up Chakravarti in your repository. This creates the `.specs` directory and workflows.
```bash
ckrv init
```

### 2. Define a Spec
Describe your feature in natural language. The AI will generate a structured specification.
```bash
# Create a new branch and spec for your feature
git checkout -b feature/dark-mode
ckrv spec new "Implement dark mode with system preference detection and a toggle switch"
```

### 3. Generate Tasks
Ask the AI to analyze the codebase and generate a detailed implementation plan (`tasks.yaml`).
```bash
ckrv spec tasks
```
Review the generated `tasks.yaml` and `plan.yaml` in `.specs/feature/dark-mode/` to ensure the architecture is sound.

### 4. Orchestrate Execution
Run the full job. Chakravarti will:
- Plan execution batches.
- Spawn parallel agents in Docker containers.
- Execute tasks in git worktrees.
- Commit and merge successful changes.
```bash
ckrv run
```

### 5. Review and Promote
Inspect the changes. Since they are already merged into your feature branch (or worktree branch), you can view them with standard git tools or use the CLI.

```bash
# Check the diff
ckrv diff <job_id>

# Push changes to the remote branch
ckrv promote <job_id>
```

## Primary Commands

| Command | Description |
|---------|-------------|
| `ckrv init` | Initialize Chakravarti in the current repository. |
| `ckrv spec` | Create specs, generate tasks, and validate plans. |
| `ckrv run` | Execute the orchestration engine (Plan -> Execute -> Merge). |
| `ckrv diff` | View the diff statistics or content of a job run. |
| `ckrv promote` | Promote the changes from a completed job to a target branch. |

## Architecture

```
chakravarti-cli/
├── crates/
│   ├── ckrv-cli/      # Usage interface
│   ├── ckrv-core/     # Orchestration, Workflow logic
│   ├── ckrv-git/      # Worktree and Git management
│   ├── ckrv-sandbox/  # Docker execution environment
│   └── ckrv-agent/    # AI Agent definitions
```

## Development

```bash
# Build
cargo build --workspace

# Run locally
cargo run -p ckrv-cli -- --help
```

## License

MIT License - see [LICENSE](LICENSE) for details.
