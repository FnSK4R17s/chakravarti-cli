---
last_commit: c1bb442
last_updated: 2026-01-21
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
| `LocalSandbox` | Direct execution (dev) |
| `AgentProvider` | AI agent interface |
| `AllowList` | Command security |

## Execution Isolation

All agent execution runs inside Docker containers:
- Isolated filesystem
- No network by default
- Command allow-list enforced
- Secrets via env vars only

## Usage

```rust
use ckrv_sandbox::{DockerSandbox, Sandbox, ExecuteConfig};

let sandbox = DockerSandbox::new(config)?;

let result = sandbox.execute(ExecuteConfig {
    command: vec!["claude", "--prompt", "..."],
    workdir: worktree_path,
    env: HashMap::new(),
})?;

if result.success {
    // Process output
}
```

## Agent System

See [Agent Guide](/crates/docs/agent-guide.md) for details on adding agents.

```rust
use ckrv_sandbox::{AgentType, create_agent};

let agent = create_agent(AgentType::Claude);
let cmd = agent.build_command(prompt, workdir, &config);
```

## Module Structure

```
src/
├── executor.rs     # Sandbox trait and implementations
├── docker.rs       # Docker client wrapper
├── allowlist.rs    # Command allow-list
├── env.rs          # Environment detection
├── agent/          # Agent providers
│   ├── mod.rs      # AgentProvider trait
│   ├── claude.rs   # Claude Code provider
│   └── codex.rs    # OpenAI Codex provider
└── error.rs        # Error types
```

## Traits

### Sandbox

```rust
pub trait Sandbox {
    fn execute(&self, config: ExecuteConfig) -> Result<ExecuteResult>;
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

## Dependencies

| Crate | Purpose |
|-------|---------|
| `bollard` | Docker API client |
| `ckrv-git` | Worktree paths |
| `tokio` | Async runtime |
