---
last_commit: f92f604
last_updated: 2026-03-01
---

# @ckrv/cli

**Cross-Platform Orchestration Engine for AI Coding Agents**

*Use all your AI coding subscriptions together.*

You write specifications. `ckrv` coordinates multiple AI agents—Claude Code, Codex, Kilo Code, and others—to implement them in parallel.

## Installation

```bash
npm install -g @ckrv/cli
```

Or download a binary from the [latest release](https://github.com/FnSK4R17s/chakravarti-cli/releases/latest).

## Quick Start

```bash
# Initialize in your repository
ckrv init

# Create a spec from description
ckrv code spec new "Add user authentication with OAuth2"

# Generate implementation tasks
ckrv code tasks

# Generate execution plan
ckrv code plan

# Execute orchestration
ckrv code run

# Review and promote
ckrv code diff
ckrv verify
ckrv promote --push --open
```

## Commands

| Command | Description |
|---------|-------------|
| `ckrv init` | Initialize Chakravarti in the current repository |
| `ckrv code` | Code workflow: spec, tasks, plan, run, diff |
| `ckrv verify` | Run tests, linting, and type checking |
| `ckrv test` | Run, plan, and write tests using AI agents |
| `ckrv qa` | QA code review and bug analysis |
| `ckrv fix` | Use AI to fix verification errors |
| `ckrv promote` | Push changes and create a Pull Request |
| `ckrv term` | Spawn interactive AI agent terminal |
| `ckrv ui` | Launch the Web UI dashboard |
| `ckrv cloud` | Cloud execution (login, logout, whoami, credentials) |
| `ckrv logs` | Stream or view cloud job logs |
| `ckrv pull` | Pull results from completed cloud jobs |

## Agents

Chakravarti orchestrates multiple AI coding agents in isolated Docker sandboxes:

| Agent | Authentication | Description |
|-------|----------------|-------------|
| Claude Code | Claude Subscription | Default — uses Anthropic's Claude directly |
| Claude Code | OpenRouter API | 12+ models (Gemini, DeepSeek, Qwen, Kimi K2, etc.) |
| Claude Code | GLM Coding Plan | Z.AI's GLM-4.7 and GLM-4.5-Air |
| Codex | OpenAI Subscription | Native Codex CLI integration |
| Kilo Code | File-based auth | 30+ AI providers (Gemini, DeepSeek, Mistral, Qwen, etc.) |

## Features

- **Multi-Agent Support** — Claude, Codex, Kilo Code, OpenRouter, GLM models
- **Isolated Execution** — Git worktree isolation per task
- **Docker Sandboxing** — Safe containerized execution
- **Metrics Tracking** — Token usage and cost accounting
- **Web UI Dashboard** — Visual workflow management
- **AI Test Writing** — Auto-generate tests with AI agents
- **AI QA Review** — Code review and bug analysis

## Documentation

For full documentation, see the [GitHub repository](https://github.com/FnSK4R17s/chakravarti-cli):

- [Architecture](https://github.com/FnSK4R17s/chakravarti-cli/blob/main/crates/docs/architecture.md)
- [Getting Started](https://github.com/FnSK4R17s/chakravarti-cli/blob/main/crates/docs/getting-started.md)
- [CLI Commands](https://github.com/FnSK4R17s/chakravarti-cli/blob/main/crates/docs/cli-commands.md)
- [Agent Guide](https://github.com/FnSK4R17s/chakravarti-cli/blob/main/crates/docs/agent-guide.md)

## Requirements

- **Docker** (for sandboxed execution)
- **Git** 2.20+

## License

MIT
