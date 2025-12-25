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
git clone https://github.com/your-org/chakravarti-cli
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
├── ckrv-model/     # LLM provider abstraction
├── ckrv-git/       # Git operations (git2 + shell)
├── ckrv-sandbox/   # Execution isolation
├── ckrv-verify/    # Test/lint verification
└── ckrv-metrics/   # Cost and timing
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
4. **Planner** generates execution plan
5. **Router** selects appropriate model
6. **Provider** calls LLM API
7. **Sandbox** executes generated commands
8. **Verify** runs tests and lints
9. **Git** manages worktrees and branches
10. **Metrics** tracks costs and timing

### Key Traits

- `ModelProvider` - LLM API abstraction
- `WorktreeManager` - Git worktree lifecycle
- `Sandbox` - Command execution isolation
- `Verifier` - Test/lint runner
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

## Adding a New Provider

1. Create module in `crates/ckrv-model/src/`
2. Implement `ModelProvider` trait
3. Add to router detection in `ModelRouter::new()`
4. Add tests
5. Update pricing catalog

## Code of Conduct

- Be respectful and inclusive
- Focus on constructive feedback
- Help others learn and grow

## License

By contributing, you agree that your contributions will be licensed under the MIT License.
