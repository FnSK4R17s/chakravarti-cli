---
last_commit: 6905171
last_updated: 2026-02-05
---

# Chakravarti CLI

**Cross-Platform Orchestration Engine for AI Coding Agents**

*Use all your AI coding subscriptions together.*

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
| `ckrv test` | Run, plan, and write tests using AI agents |
| `ckrv qa` | QA code review and bug analysis |
| `ckrv fix` | Use AI to fix verification errors |
| `ckrv promote` | Push changes and create a Pull Request |
| `ckrv ui` | Launch the Web UI dashboard |
| `ckrv cloud` | Cloud execution commands |
| `ckrv logs` | View execution logs |

## Agents

Chakravarti uses Claude Code CLI as the execution interface:

| Agent | Authentication | Description |
|-------|----------------|-------------|
| Claude Code | Claude Subscription | Default - uses Anthropic's Claude directly |
| Claude Code | OpenRouter API | 12+ models (Gemini, DeepSeek, Qwen, Kimi K2, etc.) |
| Claude Code | GLM Coding Plan | Z.AI's GLM-4.7 and GLM-4.5-Air |
| Codex | OpenAI Subscription | Native Codex CLI integration |

## Features

- 🤖 **Multi-Agent Support** - Claude, Codex, OpenRouter, GLM models
- 🔒 **Isolated Execution** - Git worktree isolation
- 🐳 **Docker Sandboxing** - Safe containerized execution
- 📊 **Metrics Tracking** - Token usage & cost
- 🌐 **Web UI Dashboard** - Visual workflow management
- 🧪 **AI Test Writing** - Auto-generate tests with AI agents
- 🔍 **AI QA Review** - Code review and bug analysis

## Documentation

For full documentation, see the [GitHub repository](https://github.com/FnSK4R17s/chakravarti-cli):

- [Architecture](https://github.com/FnSK4R17s/chakravarti-cli/blob/main/crates/docs/architecture.md)
- [Getting Started](https://github.com/FnSK4R17s/chakravarti-cli/blob/main/crates/docs/getting-started.md)
- [CLI Commands](https://github.com/FnSK4R17s/chakravarti-cli/blob/main/crates/docs/cli-commands.md)
- [Agent Guide](https://github.com/FnSK4R17s/chakravarti-cli/blob/main/crates/docs/agent-guide.md)

## Requirements

- **Rust** 1.75+
- **Docker** (for sandboxed execution)
- **Git** 2.20+

## License

MIT
