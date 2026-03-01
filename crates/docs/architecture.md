---
last_commit: f92f604
last_updated: 2026-03-01
related_files:
  - Cargo.toml
  - crates/ckrv-core/src/lib.rs
  - crates/ckrv-sandbox/src/lib.rs
  - crates/ckrv-mcp/src/lib.rs
  - crates/ckrv-transport/src/lib.rs
---

# Chakravarti CLI Architecture

## Overview

Chakravarti CLI is a spec-driven agent orchestration engine built in Rust. It transforms high-level specifications into shipping code by orchestrating AI agents across isolated Git worktrees.

## Crate Dependency Graph

```mermaid
graph TD
    CLI[ckrv-cli] --> CORE[ckrv-core]
    CLI --> UI[ckrv-ui]
    CLI --> GIT[ckrv-git]
    
    CORE --> SPEC[ckrv-spec]
    CORE --> SANDBOX[ckrv-sandbox]
    CORE --> MODEL[ckrv-model]
    CORE --> METRICS[ckrv-metrics]
    
    SANDBOX --> GIT
    
    UI --> CORE
    UI --> GIT
    UI --> TRANSPORT[ckrv-transport]
    UI --> INTEGRATIONS[ckrv-integrations]
    
    TRANSPORT --> CORE
    
    TAURI[ckrv-tauri] --> TRANSPORT
    TAURI --> UI
    
    MCP[ckrv-mcp] --> CLI
    
    VERIFY[ckrv-verify] --> SANDBOX
```

## Crate Responsibilities

| Crate | Purpose | Status |
|-------|---------|--------|
| `ckrv-cli` | CLI entry point, command handlers, user prompts | ✅ Used |
| `ckrv-core` | Orchestration engine, workflow execution, domain types | ✅ Used |
| `ckrv-git` | Git operations, worktree management, branch handling | ✅ Used |
| `ckrv-sandbox` | Docker execution, agent providers, command allow-list | ✅ Used |
| `ckrv-spec` | Spec file loading, parsing, validation | ✅ Used |
| `ckrv-model` | LLM provider abstraction, routing, cost tracking | ⚠️ **Unused** |
| `ckrv-metrics` | Metrics collection, cost/time tracking, file storage | ✅ Used |
| `ckrv-verify` | Test execution, output parsing, acceptance checking | ⚠️ **Unused** |
| `ckrv-integrations` | External service integrations (GitHub, etc.) | ⚠️ **Stub** |
| `ckrv-ui` | Web dashboard server, static file serving, execution engine | ✅ Used |
| `ckrv-transport` | Shared HTTP API types, handlers, and routes for web/desktop | ✅ Used |
| `ckrv-tauri` | Tauri desktop application (wraps ckrv-transport) | ✅ Used |
| `ckrv-mcp` | MCP server exposing CLI commands as tools for AI agents | ✅ Used |

## Execution Flow

```mermaid
sequenceDiagram
    participant User
    participant CLI as ckrv-cli
    participant Core as ckrv-core
    participant Git as ckrv-git
    participant Sandbox as ckrv-sandbox
    participant Agent as AI Agent

    User->>CLI: ckrv run
    CLI->>Core: Load Spec
    Core->>Core: Generate Plan
    Core->>Git: Create Worktrees
    
    loop For each batch
        Core->>Sandbox: Execute in Docker
        Sandbox->>Agent: Run with prompt
        Agent-->>Sandbox: Code changes
        Sandbox-->>Core: Results
        Core->>Git: Commit & Merge
    end
    
    Core->>Core: Verify
    Core-->>CLI: Complete
    CLI-->>User: Success
```

## Key Abstractions

### Spec → Plan → Job

```
Spec (what to build)
  ↓
Plan (how to build: phases, batches, tasks)
  ↓
Job (execution instance with attempts)
  ↓
Attempt (single execution run with results)
```

### Orchestrator

The `Orchestrator` trait defines the execution contract:

```rust
pub trait Orchestrator {
    fn run(&mut self, spec: Spec) -> OrchestratorResult;
    fn on_event(&self, handler: impl EventHandler);
}
```

### Sandbox

The `Sandbox` trait abstracts execution environments:

```rust
pub trait Sandbox {
    fn execute(&self, config: ExecuteConfig) -> ExecuteResult;
}

// Implementations:
// - DockerSandbox: Containerized execution
// - LocalSandbox: Direct execution (dev mode)
```

### Agent Provider

The `AgentProvider` trait enables multiple AI backends:

```rust
pub trait AgentProvider: Send + Sync {
    fn name(&self) -> &str;
    fn agent_type(&self) -> AgentType;
    fn build_command(&self, prompt: &str, workdir: &Path, config: &AgentConfig) -> Vec<String>;
    fn required_env_vars(&self) -> Vec<&str>;
    fn config_mounts(&self, host_home: &str, container_home: &str) -> Vec<Mount>;
    fn parse_output(&self, stdout: &str, stderr: &str, exit_code: i32) -> Result<AgentOutput>;
}

// Implementations:
// - ClaudeProvider (Claude Code native)
// - CodexProvider (OpenAI Codex)
// - KiloCodeProvider (Kilo Code multi-provider)
// - QwenProvider (Qwen Code)
// OpenRouter and GLM models route through Claude Code CLI via env vars
```

## Data Flow

1. **Input**: User provides spec file or natural language description
2. **Planning**: AI generates structured plan with phases and tasks
3. **Isolation**: Each task executes in isolated Git worktree
4. **Execution**: Agent runs in Docker container with allow-listed commands
5. **Verification**: Tests and linting validate changes
6. **Integration**: Successful changes merge back to main branch
7. **Output**: Clean, tested, documented code

## Security Model

- All execution isolated via Docker containers
- Command allow-list prevents dangerous operations
- No network access unless explicitly configured
- Secrets injected via environment variables only
- Main branch never directly modified
