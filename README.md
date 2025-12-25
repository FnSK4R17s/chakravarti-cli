# Chakravarti CLI

> Spec-driven autonomous code editing with LLM integration

[![Rust](https://img.shields.io/badge/rust-1.75+-orange.svg)](https://www.rust-lang.org/)
[![License](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)

Chakravarti is a command-line tool that takes structured specifications and autonomously generates code changes using large language models. It provides a complete workflow from spec definition through execution, verification, and git integration.

## Features

- 📝 **Spec-Driven Development** - Define changes in YAML/JSON specs
- 🤖 **Multi-Model Support** - OpenAI, Anthropic, and custom endpoints
- 🔄 **Automatic Retries** - Intelligent retry with replanning
- 🐳 **Isolated Execution** - Run in Docker containers or local sandboxes
- 🔍 **Verification** - Test and lint changes before committing
- 📊 **Cost Tracking** - Monitor token usage and API costs
- 🌿 **Git Native** - Promote changes to branches seamlessly

## Quick Start

### Installation

```bash
# From source
git clone https://github.com/your-org/chakravarti-cli
cd chakravarti-cli
cargo install --path crates/ckrv-cli
```

### Initialize a Repository

```bash
cd your-project
ckrv init
```

This creates:
- `.specs/` - Directory for spec files
- `.chakravarti/` - Working directory for runs

### Create a Spec

```yaml
# .specs/add-auth.yaml
goal: "Add JWT authentication middleware"
constraints:
  - "Use jsonwebtoken crate"
  - "Support RS256 algorithm"
acceptance:
  - "Middleware validates JWT tokens"
  - "All tests pass"
```

### Run the Spec

```bash
# Set your API key
export OPENAI_API_KEY="sk-..."

# Run the spec
ckrv run .specs/add-auth.yaml

# With optimization
ckrv run .specs/add-auth.yaml --optimize cost
```

### Review and Promote

```bash
# Check job status
ckrv status <job_id>

# View the diff
ckrv diff <job_id>

# See cost report
ckrv report <job_id>

# Promote to a branch
ckrv promote <job_id> --branch feature/add-auth
```

## Commands

| Command | Description |
|---------|-------------|
| `ckrv init` | Initialize a repository for Chakravarti |
| `ckrv run <spec>` | Execute a specification |
| `ckrv status <job>` | Check job status |
| `ckrv diff <job>` | View generated diff |
| `ckrv report <job>` | View cost/metrics report |
| `ckrv promote <job>` | Promote changes to a git branch |

## Configuration

### Environment Variables

```bash
# OpenAI
export OPENAI_API_KEY="sk-..."

# Anthropic
export ANTHROPIC_API_KEY="sk-ant-..."

# Custom endpoint (OpenAI-compatible)
export CKRV_MODEL_API_KEY="your-key"
export CKRV_MODEL_ENDPOINT="https://api.example.com/v1"
```

### Optimization Modes

```bash
--optimize cost      # Minimize API costs
--optimize time      # Minimize execution time
--optimize balanced  # Balance cost and time (default)
```

See [docs/optimization.md](docs/optimization.md) for details.

## Architecture

```
chakravarti-cli/
├── crates/
│   ├── ckrv-cli/      # CLI application
│   ├── ckrv-core/     # Core types and orchestration
│   ├── ckrv-spec/     # Spec parsing and validation
│   ├── ckrv-model/    # LLM provider abstraction
│   ├── ckrv-git/      # Git operations
│   ├── ckrv-sandbox/  # Execution isolation
│   ├── ckrv-verify/   # Testing and linting
│   └── ckrv-metrics/  # Cost and time tracking
└── docs/              # Documentation
```

## Development

```bash
# Build
cargo build --workspace

# Test
cargo test --workspace

# Run locally
cargo run -p ckrv-cli -- --help

# Format
cargo fmt --all

# Lint
cargo clippy --workspace
```

## License

MIT License - see [LICENSE](LICENSE) for details.
