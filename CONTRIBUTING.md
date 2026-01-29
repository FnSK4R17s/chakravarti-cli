# Contributing to Chakravarti

Thank you for your interest in contributing to Chakravarti! This document provides guidelines and setup instructions for development.

## Development Setup

### Prerequisites

- Rust 1.75 or later
- Docker (optional, for container sandbox testing)
- Git

### Getting Started

```bash
# Clone the repository
git clone https://github.com/FnSK4R17s/chakravarti-cli
cd chakravarti-cli

# Build all crates
cargo build --workspace

# Run tests
cargo test --workspace

# Run the CLI
cargo run -p ckrv-cli -- --help
```

### Environment Setup

For full functionality, set up API keys:

```bash
# Create secrets directory (already in .gitignore)
mkdir -p .chakravarti/secrets

# Add your keys
echo "OPENAI_API_KEY=sk-..." > .chakravarti/secrets/.env
echo "ANTHROPIC_API_KEY=sk-ant-..." >> .chakravarti/secrets/.env
```

## Project Structure

```
crates/
├── ckrv-cli/       # Main CLI binary
├── ckrv-core/      # Core types, job, plan, orchestrator
├── ckrv-spec/      # Spec parsing (YAML/JSON)
├── ckrv-model/     # LLM provider abstraction (⚠️ unused)
├── ckrv-git/       # Git operations (git2 + shell)
├── ckrv-sandbox/   # Docker execution, agent providers
├── ckrv-verify/    # Test execution and parsing (⚠️ unused)
├── ckrv-metrics/   # Cost/time tracking, file storage
└── ckrv-ui/        # Web dashboard server + frontend
```

## Development Workflow

### Running Tests

```bash
# All tests
cargo test --workspace

# Specific crate
cargo test -p ckrv-core

# With output
cargo test -p ckrv-model -- --nocapture

# Integration tests (require API keys)
cargo test --test integration -- --ignored
```

### Code Quality

```bash
# Format code
cargo fmt --all

# Check formatting
cargo fmt --all -- --check

# Lint
cargo clippy --workspace -- -D warnings

# Documentation
cargo doc --workspace --no-deps
```

### Building Release

```bash
# Build optimized binary
cargo build --release

# Binary location
./target/release/ckrv
```

## Architecture Overview

### Request Flow

1. **CLI** parses arguments and loads spec
2. **Spec** validates and normalizes the specification
3. **Orchestrator** creates job and manages lifecycle
4. **Sandbox** selects agent (Claude, Codex) and executes in Docker
5. **Git** manages worktrees and branches
6. **Metrics** tracks costs and timing

> **Note**: Steps 4-6 run in a loop with retries until verification passes.

### Key Traits

- `AgentProvider` - AI agent abstraction (Claude, Codex)
- `WorktreeManager` - Git worktree lifecycle
- `Sandbox` - Command execution isolation
- `Orchestrator` - Job lifecycle management

## Pull Request Process

1. Fork the repository
2. Create a feature branch: `git checkout -b feature/my-feature`
3. Make your changes
4. Add tests for new functionality
5. Ensure all tests pass: `cargo test --workspace`
6. Format code: `cargo fmt --all`
7. Check lints: `cargo clippy --workspace`
8. Submit pull request

### Commit Messages

Use conventional commits:

- `feat:` New feature
- `fix:` Bug fix
- `docs:` Documentation
- `test:` Tests
- `refactor:` Code refactoring
- `chore:` Maintenance

Example: `feat(router): add budget tracking for cost optimization`

## Adding a New Agent

See [Agent Guide](crates/docs/agent-guide.md) for adding new AI agent integrations.

1. Add `AgentType` variant in `crates/ckrv-sandbox/src/agent/mod.rs`
2. Implement `AgentProvider` trait
3. Register in `create_agent()` factory
4. Add to Docker image if needed
5. Add tests

## Code of Conduct

- Be respectful and inclusive
- Focus on constructive feedback
- Help others learn and grow

## License

By contributing, you agree that your contributions will be licensed under the MIT License.
