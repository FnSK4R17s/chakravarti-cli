# Contributing to Chakravarti

Thank you for your interest in contributing to chakravarti-cli! This document provides guidelines and setup instructions for development.

## Development Setup

### Prerequisites

- **Rust 1.75+**: Install via [rustup](https://rustup.rs/)
- **just**: Task runner - install via [just installation](https://github.com/casey/just#installation)
- **Docker**: For sandboxed execution ([Docker Desktop](https://www.docker.com/products/docker-desktop/) or [Podman](https://podman.io/))
- **Git 2.20+**: For worktree operations
- **Node.js 18+**: For UI development (optional)

### Getting Started

```bash
# Clone the repository
git clone https://github.com/FnSK4R17s/chakravarti-cli
cd chakravarti-cli

# Build and install
just install

# Run tests
just test

# Run the CLI
cargo run -p ckrv-cli -- --help
```

### Agent Setup

`ckrv` invokes AI agents via their CLI tools—no API keys needed. Install the agents you want to use:

- **Claude Code**: [claude.ai/code](https://claude.ai/code) (Claude subscription)
- **Codex**: [OpenAI Codex CLI](https://github.com/openai/codex-cli) (OpenAI subscription)
- **Kilo Code**: 30+ AI providers via file-based auth (`kilo auth`)
- **Gemini CLI**: `GEMINI_API_KEY` + `~/.gemini/` config
- **Cursor**: `~/.cursor/` config (Cursor subscription)
- **Amp**: `~/.amp/` config (Amp authentication)
- **Qwen Code**: `QWEN_API_KEY`
- **Opencode**: `~/.config/opencode/` config
- **Factory Droid**: `~/.factory/` config
- **GitHub Copilot**: `~/.config/github-copilot/` (GitHub Copilot subscription)
- **Mistral Vibe**: `MISTRAL_API_KEY`

Configure agents in `~/.config/chakravarti/agents.yaml`:

```yaml
agents:
  - name: claude-default
    agent_type: claude
    is_default: true
  - name: codex
    agent_type: codex
  - name: kilo
    agent_type: kilo_code
  - name: gemini
    agent_type: gemini
```

## Project Structure

```
crates/
├── ckrv-cli/          # Main CLI binary
├── ckrv-core/         # Core types, job, plan, orchestrator
├── ckrv-spec/         # Spec parsing (YAML/JSON)
├── ckrv-git/          # Git operations (git2 + shell)
├── ckrv-sandbox/      # Docker execution, agent providers
├── ckrv-mcp/          # MCP server for AI agent integration
├── ckrv-transport/    # Shared HTTP API types for web/desktop
├── ckrv-tauri/        # Tauri desktop application
├── ckrv-model/        # LLM provider abstraction (⚠️ unused)
├── ckrv-metrics/      # Cost/time tracking, file storage
├── ckrv-verify/       # Test execution and parsing (⚠️ unused)
├── ckrv-integrations/ # External integrations stub (⚠️ stub)
└── ckrv-ui/           # Web dashboard server + frontend
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

# Documentation
just docs
```

### Building Release

```bash
# Build optimized binary
just build

# Binary location
./target/release/ckrv
```

## Architecture Overview

### Request Flow

1. **CLI** parses arguments and loads spec
2. **Spec** validates and normalizes the specification
3. **Orchestrator** creates job and manages lifecycle
4. **Sandbox** invokes agent CLI (`claude`, `codex`) in Docker
5. **Git** manages worktrees and branches
6. **Metrics** tracks execution time

> **Note**: Steps 4-6 run in a loop with retries until verification passes.

### Key Traits

- `AgentProvider` - AI agent CLI invocation (Claude, Codex)
- `WorktreeManager` - Git worktree lifecycle
- `Sandbox` - Docker container execution
- `Orchestrator` - Job lifecycle management

## Pull Request Process

1. Fork the repository
2. Create a feature branch: `git checkout -b feature/my-feature`
3. Make your changes
4. Add tests for new functionality
5. Ensure all tests pass: `just test`
6. Format code: `just fmt`
7. Check lints: `just lint`
8. Submit pull request

### Commit Messages

Use conventional commits:

- `feat:` New feature
- `fix:` Bug fix
- `docs:` Documentation
- `test:` Tests
- `refactor:` Code refactoring
- `chore:` Maintenance

Example: `feat(sandbox): add Gemini CLI support`

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
