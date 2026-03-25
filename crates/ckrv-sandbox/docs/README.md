---
last_commit: b41880d
last_updated: 2026-03-25
related_files:
  - src/lib.rs
  - src/executor.rs
  - src/docker.rs
  - src/agent/mod.rs
---

# ckrv-sandbox

Containerized execution and agent abstraction for Chakravarti.

## Overview

This crate provides sandboxed command execution using Docker/Podman. It also contains the agent abstraction layer for supporting multiple AI providers.

## Key Types

| Type | Purpose |
|------|---------|
| `Sandbox` | Execution environment trait |
| `DockerSandbox` | Docker-based execution |
| `LocalSandbox` | Direct execution (dev mode) |
| `DockerClient` | Low-level Docker API wrapper |
| `AgentProvider` | AI agent interface |
| `AllowList` | Command security |

## Execution Isolation

All agent execution runs inside Docker containers:
- Isolated filesystem
- No network by default
- Command allow-list enforced
- Secrets via env vars only
- **Non-root execution** — containers start as root for workspace ownership fixup (`chown`), then drop to the `agent` user (uid 1001) via `su` before running the agent command
- **TTY emulation** — containers run with `tty: true` and terminal env vars (`TERM=xterm-256color`, `COLORTERM=truecolor`, `COLUMNS=120`, `LINES=30`) so agent CLIs detect full terminal capabilities
- **Workspace ownership restoration** — after container exit, a lightweight Alpine container runs `chown -R` to restore host user ownership of workspace files modified by the agent user

> **Warning**: Containers run the agent as a non-root user (`agent`, uid 1001).
> Agent CLIs like Claude Code enforce security restrictions when running as
> root (e.g., blocking `--dangerously-skip-permissions`). The container starts
> as root only to `chown` the mounted workspace, then drops to the `agent` user
> for the actual command.

## Credential Mounting

The sandbox automatically mounts agent credentials from the host:

| Tool | Authentication | How Mounted |
|------|----------------|-------------|
| Claude Code | Claude Subscription | `~/.claude.json`, `~/.claude/` bind-mounted |
| Claude Code | OpenRouter API | `OPENROUTER_API_KEY`, `ANTHROPIC_BASE_URL` env vars |
| Claude Code | GLM Coding Plan | `ZAI_API_KEY`, `ANTHROPIC_BASE_URL` env vars |
| Codex | OpenAI Subscription | `~/.codex/` bind-mounted |
| Kilo Code | File-based auth | `~/.config/kilo/` bind-mounted (read-write) |

> **Note**: OpenRouter and GLM authentication is handled by `ckrv-cli` and `ckrv-ui`, which set the appropriate environment variables. The sandbox (`ckrv-sandbox`) provides native Claude, Codex, and Kilo Code providers.

## Module Structure

```
src/
├── lib.rs          # Public API exports
├── executor.rs     # Sandbox trait and implementations
├── docker.rs       # DockerClient wrapper (24KB)
├── allowlist.rs    # Command allow-list
├── env.rs          # Environment detection
├── error.rs        # Error types
└── agent/          # Agent providers
    ├── mod.rs      # AgentProvider trait, AgentType enum
    ├── claude.rs   # Claude Code provider
    ├── codex.rs    # OpenAI Codex provider
    ├── kilo.rs     # Kilo Code provider
    ├── amp.rs      # Amp CLI provider
    ├── copilot.rs  # GitHub Copilot CLI provider
    ├── cursor.rs   # Cursor CLI provider
    ├── factory.rs  # Factory Droid CLI provider
    ├── gemini.rs   # Gemini CLI provider
    ├── opencode.rs # Opencode CLI provider
    ├── qwen.rs     # Qwen Code CLI provider
    ├── vibe.rs     # Mistral Vibe CLI provider
    └── tests.rs    # Agent unit tests
```

## API Style

Builder methods on `ExecuteConfig`, `BindMount`, `EnvConfig`, `DefaultAllowList`, `AgentConfig`, and `DockerClient` accept `&str` parameters (not `impl Into<String>`). This keeps the API surface concrete and avoids monomorphization bloat.

## Usage

### Basic Execution

```rust
use ckrv_sandbox::{DockerSandbox, Sandbox, ExecuteConfig};

let sandbox = DockerSandbox::with_defaults()?;

let result = sandbox.execute(ExecuteConfig::new("claude", workdir)
    .shell("claude --help")
    .env("HOME", "/home/agent")
).await?;

if result.success() {
    println!("{}", result.combined_output());
}
```

### Streaming Execution

```rust
// Real-time log streaming to UI
sandbox.execute_streaming(config, |log_line| {
    println!("{}", log_line);
}).await?;
```

### Session-Based Execution

```rust
use ckrv_sandbox::DockerClient;

let client = DockerClient::new()?;

// Create persistent session
let container_id = client.create_session(workdir, mount_src, mount_tgt, env).await?;

// Execute multiple commands in same container
client.exec_in_session(&container_id, vec!["ls", "-la"]).await?;
client.exec_in_session(&container_id, vec!["claude", "--prompt", "..."]).await?;

// Cleanup
client.stop_session(&container_id).await?;
```

## Agent System

Supported agents in this crate:

| Agent | Provider Struct | CLI Binary |
|-------|-----------------|------------|
| Claude Code | `ClaudeProvider` | `claude` |
| OpenAI Codex | `CodexProvider` | `codex` |
| Kilo Code | `KiloCodeProvider` | `kilo` |
| Amp | `AmpProvider` | `amp` |
| GitHub Copilot | `GithubCopilotProvider` | `gh copilot` |
| Cursor | `CursorProvider` | `cursor` |
| Factory Droid | `FactoryDroidProvider` | `factory` |
| Gemini CLI | `GeminiProvider` | `gemini` |
| Opencode | `OpencodeProvider` | `opencode` |
| Qwen Code | `QwenProvider` | `qwen-coder` |
| Mistral Vibe | `VibeProvider` | `vibe` |

```rust
use ckrv_sandbox::{AgentType, create_agent, default_agent};

// Create specific agent
let agent = create_agent(AgentType::Claude);
let cmd = agent.build_command(prompt, workdir, &config);

// Create Kilo Code agent
let kilo = create_agent(AgentType::KiloCode);

// Or use default (Claude)
let agent = default_agent();
```

See [Agent Guide](/crates/docs/agent-guide.md) for adding new agents.

## Traits

### Sandbox

```rust
#[async_trait]
pub trait Sandbox {
    async fn execute(&self, config: ExecuteConfig) -> Result<ExecuteResult, SandboxError>;
    async fn health_check(&self) -> Result<(), SandboxError>;
}
```

### AgentProvider

```rust
pub trait AgentProvider: Send + Sync {
    fn name(&self) -> &str;
    fn agent_type(&self) -> AgentType;
    fn build_command(&self, prompt: &str, workdir: &Path, config: &AgentConfig) -> Vec<String>;
    fn required_env_vars(&self) -> Vec<&str>;
    fn config_mounts(&self, host_home: &str, container_home: &str) -> Vec<Mount>;
    fn parse_output(&self, stdout: &str, stderr: &str, exit_code: i32) -> Result<AgentOutput>;
}
```

## DockerClient API

The low-level Docker client provides:

| Method | Purpose |
|--------|---------|
| `new()` | Create client, connect to Docker daemon |
| `set_image(&str)` | Override the default Docker image |
| `health_check()` | Verify Docker is available |
| `ensure_image()` | Pull image if not present |
| `execute()` | One-shot container execution (runs as root, drops to agent user) |
| `execute_streaming()` | Execution with real-time log callback |
| `create_session()` | Start persistent container for multi-command sessions |
| `exec_in_session()` | Run command in existing session |
| `stop_session()` | Cleanup session container |

### Container Lifecycle

1. Container starts as **root** with workspace bind-mounted
2. Runs `chown -R agent:agent /workspace` to fix ownership
3. Drops to `agent` user via `su -s /bin/sh agent -c '...'`
4. After exit, `restore_workspace_ownership()` runs a lightweight Alpine container to `chown` files back to the host user

## Dependencies

| Crate | Purpose |
|-------|---------|
| `bollard` | Docker API client |
| `tokio` | Async runtime |
| `async-trait` | Async trait support |
| `futures-util` | Stream utilities |
