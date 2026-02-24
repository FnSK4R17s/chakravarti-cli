---
last_commit: 508766e
last_updated: 2026-02-15
related_files:
  - README.md
  - CONTRIBUTING.md
  - Cargo.toml
---

# Getting Started with Chakravarti CLI

## Prerequisites

Before developing Chakravarti CLI, ensure you have:

- **Rust 1.75+**: Install via [rustup](https://rustup.rs/)
- **just**: Task runner - install via [just installation](https://github.com/casey/just#installation)
- **Docker**: For sandboxed execution ([Docker Desktop](https://www.docker.com/products/docker-desktop/) or [Podman](https://podman.io/))
- **Git 2.20+**: For worktree operations
- **Node.js 18+**: For UI development (optional)

## Quick Setup

```bash
# Clone the repository
git clone https://github.com/FnSK4R17s/chakravarti-cli.git
cd chakravarti-cli

# Quick install (recommended for containers/dev environments - no Docker needed)
just install-quick

# Or full install (includes Docker agent images for sandboxed execution)
just install
```

### Install Options

| Command | Docker Images | Use Case |
|---------|---------------|----------|
| `just install-quick` | Skipped | Containers, dev environments, CI without Docker |
| `just install` | Built | Local development with full agent support |
| `CKRV_SKIP_DOCKER=true just install` | Skipped | Environment variable override |

```bash
# Run tests
just test

# Run linters
just lint
```

### Quick start: Mistral Vibe

1. **Install** — `curl -LsSf https://mistral.ai/vibe/install.sh | bash`
   (or `uv tool install mistral-vibe` if you have uv; requires Python ≥3.12)
2. **Get a Mistral API key** — https://console.mistral.ai → API Keys
3. **Add agent config** — paste into `~/.config/chakravarti/agents.yaml`:
   ```yaml
   agents:
     - id: mistral-vibe
       name: Mistral Vibe
       agent_type: mistral_vibe
       enabled: true
       vibe:
         max_turns: 50
   ```
4. **Set API key and verify**:
   ```bash
   export MISTRAL_API_KEY="sk-..."
   ckrv task run --agent mistral-vibe -p "Say hello"
   ```

See [Agent Guide](agent-guide.md#mistral-vibe-integration) for full details and troubleshooting.

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
just ui-dev
```

### Running Tests

```bash
# All tests
just test

# Specific crate
cargo test -p ckrv-core

# With coverage (requires cargo-llvm-cov)
cargo llvm-cov --workspace
```

### Code Quality

```bash
# Format code
just fmt

# Lint
just lint

# Generate docs
just docs
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
just fmt
just lint
just test
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
