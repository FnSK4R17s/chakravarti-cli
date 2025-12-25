# Chakravarti CLI Development Guidelines

Last updated: 2025-12-25

## Overview

Chakravarti is a spec-driven autonomous code editing CLI. It takes structured YAML/JSON specifications and uses LLMs to generate and apply code changes.

## Technologies

- **Rust 1.75+** - Core language
- **clap** - CLI argument parsing
- **tokio** - Async runtime
- **serde** - Serialization (JSON/YAML)
- **bollard** - Docker API client
- **git2** - Git operations
- **reqwest** - HTTP client for LLM APIs

## Project Structure

```text
chakravarti-cli/
├── Cargo.toml              # Workspace root
├── crates/
│   ├── ckrv-cli/           # Binary - CLI commands
│   ├── ckrv-core/          # Library - Job, Plan, Orchestrator
│   ├── ckrv-spec/          # Library - Spec parsing/validation
│   ├── ckrv-model/         # Library - LLM providers (OpenAI, Anthropic)
│   ├── ckrv-git/           # Library - Git worktrees, branches, diffs
│   ├── ckrv-sandbox/       # Library - Docker/local execution
│   ├── ckrv-verify/        # Library - Test/lint verification
│   └── ckrv-metrics/       # Library - Cost/timing tracking
├── docs/                   # Documentation
├── specs/                  # Feature specifications
└── .github/workflows/      # CI configuration
```

## Commands

```bash
# Build
cargo build --workspace

# Test
cargo test --workspace

# Lint
cargo clippy --workspace -- -D warnings

# Format
cargo fmt --all

# Coverage
cargo tarpaulin --workspace

# Run CLI
cargo run -p ckrv-cli -- --help
```

## CLI Usage

```bash
ckrv init                              # Initialize repository
ckrv spec .specs/feature.yaml          # Validate spec
ckrv run .specs/feature.yaml           # Execute spec
ckrv status <job_id>                   # Check job status
ckrv diff <job_id>                     # View changes
ckrv report <job_id>                   # View metrics
ckrv promote <job_id> --branch name    # Promote to branch
```

## Environment Variables

```bash
OPENAI_API_KEY          # OpenAI API key
ANTHROPIC_API_KEY       # Anthropic API key
CKRV_MODEL_API_KEY      # Custom endpoint key
CKRV_MODEL_ENDPOINT     # Custom endpoint URL
```

## Code Style

- Follow Rust standard conventions
- Use `rustfmt` for formatting
- Pass `clippy` with no warnings
- Document public APIs with doc comments
- Add tests for new functionality

## Testing

- Unit tests in each crate's source files
- Integration tests in `crates/ckrv-cli/tests/`
- Tests marked `#[ignore]` require API keys or Docker

## Key Files

| File | Purpose |
|------|---------|
| `crates/ckrv-core/src/job.rs` | Job lifecycle management |
| `crates/ckrv-core/src/orchestrator.rs` | Execution orchestration |
| `crates/ckrv-model/src/router.rs` | Model selection logic |
| `crates/ckrv-git/src/worktree.rs` | Git worktree management |
| `crates/ckrv-cli/src/commands/run.rs` | Main run command |

## Recent Changes (2025-12-25)

- Completed MVP implementation (174 tasks)
- All 6 user stories implemented
- 226 tests passing
- Documentation complete
- CI/CD configured

<!-- MANUAL ADDITIONS START -->
<!-- MANUAL ADDITIONS END -->
