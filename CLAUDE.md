# Chakravarti CLI Development Guidelines

Last updated: 2026-01-21

## Overview

Chakravarti is a spec-driven autonomous agent orchestration engine. It transforms high-level specifications into shipping code by orchestrating AI agents across isolated Git worktrees and Docker sandboxes.

## Documentation

**Before making code changes, consult these docs:**

| Document | Purpose |
|----------|---------|
| [Architecture](crates/docs/architecture.md) | Crate dependencies, execution flow, key abstractions |
| [Getting Started](crates/docs/getting-started.md) | Setup, build commands, first contribution |
| [CLI Commands](crates/docs/cli-commands.md) | All commands with options and exit codes |
| [Agent Guide](crates/docs/agent-guide.md) | Adding new AI agent integrations |

**Per-crate documentation** is in `crates/<crate>/docs/README.md`.

## Technologies

- **Rust 1.75+** - Core language
- **clap** - CLI argument parsing
- **tokio** - Async runtime
- **axum** - Web server (for UI)
- **bollard** - Docker API client
- **git2** - Git operations

## Project Structure

```text
chakravarti-cli/
├── crates/
│   ├── ckrv-cli/           # CLI entry point, commands
│   ├── ckrv-core/          # Orchestration engine, domain types
│   ├── ckrv-git/           # Git worktrees, branches, diffs
│   ├── ckrv-sandbox/       # Docker execution, agent providers
│   ├── ckrv-spec/          # Spec parsing/validation
│   ├── ckrv-model/         # LLM provider routing
│   ├── ckrv-metrics/       # Cost/timing tracking
│   ├── ckrv-verify/        # Test/lint verification
│   ├── ckrv-integrations/  # External services (GitHub)
│   └── ckrv-ui/            # Web dashboard server + frontend
│       └── docs/           # Per-crate documentation
├── crates/docs/            # Cross-crate documentation
├── specs/                  # Feature specifications
└── npm/                    # npm package for distribution
```

## Commands

```bash
# Build and install
make install

# Build only
cargo build --workspace

# Test
cargo test --workspace

# Lint
cargo clippy --workspace -- -D warnings

# Format
cargo fmt --all

# Generate docs
cargo doc --open --no-deps

# Run CLI
cargo run -p ckrv-cli -- --help
```

## CLI Usage

```bash
ckrv init                    # Initialize repository
ckrv spec new "description"  # Create spec
ckrv spec tasks              # Generate tasks
ckrv plan                    # Generate execution plan
ckrv run                     # Execute orchestration
ckrv diff                    # View changes
ckrv verify                  # Run tests/lint
ckrv fix                     # AI-powered fixes
ckrv promote --push --open   # Create PR
ckrv ui                      # Launch Web UI
```

## Agents

Chakravarti uses Claude Code CLI as the execution interface:

- **Claude Code (Native)** - Default agent
- **OpenAI Codex** - Native CLI integration  
- **OpenRouter Models** - 12+ models via Claude Code CLI

See [Agent Guide](crates/docs/agent-guide.md) for adding new agents.

## Code Style

- Follow Rust standard conventions
- Use `rustfmt` for formatting
- Pass `clippy` with no warnings
- Document public APIs with doc comments (`///`)
- Add crate-level docs (`//!`) to each `lib.rs`
- Add tests for new functionality

## Key Files

| File | Purpose |
|------|---------|
| `crates/ckrv-core/src/orchestrator.rs` | Execution orchestration |
| `crates/ckrv-core/src/job.rs` | Job lifecycle management |
| `crates/ckrv-sandbox/src/agent/mod.rs` | Agent provider trait |
| `crates/ckrv-git/src/worktree.rs` | Git worktree management |
| `crates/ckrv-cli/src/commands/run.rs` | Main run command |
| `crates/ckrv-ui/src/api/` | Web UI API endpoints |

## Testing

- Unit tests in each crate's source files
- Integration tests in `crates/ckrv-cli/tests/`
- Tests marked `#[ignore]` require API keys or Docker
- Run `cargo test --workspace` before committing

<!-- MANUAL ADDITIONS START -->
<!-- MANUAL ADDITIONS END -->

## Active Technologies
- Rust 1.75+ + Markdown, Mermaid diagrams, cargo doc (012-code-documentation)
- N/A (documentation files only) (012-code-documentation)
