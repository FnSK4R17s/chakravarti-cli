---
last_commit: 1b27ca2
last_updated: 2026-02-10
related_files:
  - README.md
  - CONTRIBUTING.md
  - Cargo.toml
---

# Getting Started with Chakravarti CLI

## Prerequisites

Before developing Chakravarti CLI, ensure you have:

- **Rust 1.75+**: Install via [rustup](https://rustup.rs/)
- **Docker**: For sandboxed execution ([Docker Desktop](https://www.docker.com/products/docker-desktop/) or [Podman](https://podman.io/))
- **Git 2.20+**: For worktree operations
- **Node.js 18+**: For UI development (optional)

## Quick Setup

```bash
# Clone the repository
git clone https://github.com/FnSK4R17s/chakravarti-cli.git
cd chakravarti-cli

# Build all crates
cargo build --workspace

# Run tests
cargo test --workspace

# Build release binary
cargo build --release -p ckrv-cli

# Install locally
cargo install --path crates/ckrv-cli
```

## Development Workflow

### Running the CLI

```bash
# From the workspace root
cargo run -p ckrv-cli -- --help

# Or after installing
ckrv --help
```

### Running the Web UI

```bash
# Start the backend server
cargo run -p ckrv-cli -- ui --port 3000

# For frontend development (in another terminal)
cd crates/ckrv-ui/frontend
npm install
npm run dev
```

### Running Tests

```bash
# All tests
cargo test --workspace

# Specific crate
cargo test -p ckrv-core

# With coverage (requires cargo-llvm-cov)
cargo llvm-cov --workspace
```

### Code Quality

```bash
# Format code
cargo fmt --all

# Lint
cargo clippy --workspace -- -D warnings

# Generate docs
cargo doc --open --no-deps
```

## Project Structure

```
chakravarti-cli/
├── crates/              # Rust workspace crates
│   ├── ckrv-cli/       # CLI entry point
│   ├── ckrv-core/      # Core orchestration
│   ├── ckrv-git/       # Git operations
│   ├── ckrv-sandbox/   # Docker execution
│   ├── ckrv-spec/      # Spec parsing
│   ├── ckrv-model/     # LLM providers
│   ├── ckrv-metrics/   # Telemetry
│   ├── ckrv-verify/    # Verification
│   ├── ckrv-integrations/ # External services
│   ├── ckrv-transport/ # Shared HTTP API types
│   ├── ckrv-tauri/    # Tauri desktop app
│   ├── ckrv-mcp/       # MCP server for AI agents
│   └── ckrv-ui/        # Web dashboard
├── docker/             # Dockerfile definitions
├── docs/               # Additional documentation
├── specs/              # Feature specifications
└── tests/              # Integration tests
```

## Making Your First Contribution

### 1. Find an Issue

Browse [open issues](https://github.com/FnSK4R17s/chakravarti-cli/issues) or check the specs folder for planned features.

### 2. Create a Branch

```bash
git checkout -b feature/your-feature-name
```

### 3. Make Changes

Follow the [TDD approach](../ckrv-core/docs/README.md):
- Write tests first
- Implement the feature
- Ensure all tests pass

### 4. Verify Quality

```bash
# Must all pass
cargo fmt --all --check
cargo clippy --workspace -- -D warnings
cargo test --workspace
cargo doc --deny warnings
```

### 5. Submit PR

Push your branch and create a pull request with:
- Clear description of changes
- Link to related issue
- Test coverage summary

## Key Concepts

- **Spec**: High-level feature description (what to build)
- **Plan**: Generated implementation plan (how to build)
- **Job**: Execution instance with multiple attempts
- **Worktree**: Isolated Git directory for safe execution
- **Agent**: AI provider (Claude, OpenAI, etc.)

## Next Steps

- Read the [Architecture](architecture.md) doc for system design
- Explore [CLI Commands](cli-commands.md) for usage
- See [Agent Guide](agent-guide.md) for adding AI providers
