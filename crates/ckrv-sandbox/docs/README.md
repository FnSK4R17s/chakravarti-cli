---
last_commit: f92f604
last_updated: 2026-03-01
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
- **Non-root user** — containers run as a dedicated user, not root

> **Warning**: All agent Docker containers must run as a non-root user.
> Agent CLIs like Claude Code enforce security restrictions when running as
> root (e.g., blocking `--dangerously-skip-permissions`). Each Dockerfile
> creates a dedicated user and switches to it via the `USER` directive.

## Credential Mounting

The sandbox automatically mounts agent credentials from the host:

| Tool | Authentication | How Mounted |
|------|----------------|-------------|
| Claude Code | Claude Subscription | `~/.claude.json`, `~/.claude/` bind-mounted |
| Claude Code | OpenRouter API | `OPENROUTER_API_KEY`, `ANTHROPIC_BASE_URL` env vars |
| Claude Code | GLM Coding Plan | `ZAI_API_KEY`, `ANTHROPIC_BASE_URL` env vars |
| Codex | OpenAI Subscription | `~/.codex/` bind-mounted |
| Kilo Code | File-based auth | `~/.config/kilo/` bind-mounted (read-write) |
| Mistral Vibe | MISTRAL_API_KEY | `MISTRAL_API_KEY` env var |

> **Note**: OpenRouter and GLM authentication is handled by `ckrv-cli` and `ckrv-ui`, which set the appropriate environment variables. The sandbox (`ckrv-sandbox`) provides native Claude, Codex, Kilo Code, and Mistral Vibe providers.

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
    ├── vibe.rs     # Mistral Vibe provider
    └── tests.rs    # Agent unit tests
```

## Usage

### Basic Execution

```rust
use ckrv_sandbox::{DockerSandbox, Sandbox, ExecuteConfig};

let sandbox = DockerSandbox::with_defaults()?;

let result = sandbox.execute(ExecuteConfig::new("claude", workdir)
    .shell("claude --help")
    .env("HOME", "/home/claude")
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
| Mistral Vibe | `MistralVibeProvider` | `vibe` |

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
| `health_check()` | Verify Docker is available |
| `ensure_image()` | Pull image if not present |
| `execute()` | One-shot container execution |
| `execute_streaming()` | Execution with real-time log callback |
| `create_session()` | Start persistent container for multi-command sessions |
| `exec_in_session()` | Run command in existing session |
| `stop_session()` | Cleanup session container |

## Dependencies

| Crate | Purpose |
|-------|---------|
| `bollard` | Docker API client |
| `tokio` | Async runtime |
| `async-trait` | Async trait support |
| `futures-util` | Stream utilities |
