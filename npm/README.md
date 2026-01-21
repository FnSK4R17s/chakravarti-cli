---
last_commit: c1bb442
last_updated: 2026-01-21
---

# Chakravarti CLI

Spec-driven Agent Orchestration Engine - turn specifications into shipping code.

## Installation

### Via Make (Recommended)

```bash
# Clone the repository
git clone https://github.com/FnSK4R17s/chakravarti-cli.git
cd chakravarti-cli

# Build and install (includes Docker images)
make install
```

### Via npm (after building)

```bash
cd npm && npm link
```

## Quick Start

```bash
# Initialize in your repository
ckrv init

# Create a spec from description
ckrv spec new "Add user authentication with OAuth2"

# Generate implementation tasks
ckrv spec tasks

# Generate execution plan
ckrv plan

# Execute orchestration
ckrv run

# Review and promote
ckrv diff
ckrv verify
ckrv promote --push --open
```

## Commands

| Command | Description |
|---------|-------------|
| `ckrv init` | Initialize Chakravarti in the current repository |
| `ckrv spec` | Manage specifications (new, list, show, tasks) |
| `ckrv plan` | Generate execution plan from tasks |
| `ckrv run` | Execute orchestration (Plan → Execute → Merge) |
| `ckrv diff` | View changes between branches |
| `ckrv verify` | Run tests, linting, and type checking |
| `ckrv fix` | Use AI to fix verification errors |
| `ckrv promote` | Push changes and create a Pull Request |
| `ckrv ui` | Launch the Web UI dashboard |
| `ckrv cloud` | Cloud execution commands |
| `ckrv logs` | View execution logs |

## Agents

Chakravarti uses Claude Code CLI as the execution interface:

- **Claude Code (Native)** - Default agent
- **OpenAI Codex** - Native CLI integration
- **OpenRouter Models** - Plug in 12+ models via Claude Code CLI

## Features

- 🤖 **Multi-Agent Support** - Claude, Codex, OpenRouter models
- 🔒 **Isolated Execution** - Git worktree isolation
- 🐳 **Docker Sandboxing** - Safe containerized execution
- 📊 **Metrics Tracking** - Token usage & cost
- 🌐 **Web UI Dashboard** - Visual workflow management

## Documentation

- [Architecture](../crates/docs/architecture.md) - System design
- [Getting Started](../crates/docs/getting-started.md) - Setup guide
- [CLI Commands](../crates/docs/cli-commands.md) - Full reference
- [Agent Guide](../crates/docs/agent-guide.md) - Adding agents

## Requirements

- **Rust** 1.75+
- **Docker** (for sandboxed execution)
- **Git** 2.20+

## License

MIT
