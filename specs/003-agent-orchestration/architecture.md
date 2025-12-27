# Architecture: Agent Orchestration

## Overview

Chakravarti's Agent Orchestration is a port of the Rover architecture, designed to execute multi-step coding tasks using Large Language Models (LLMs) in a safe, sandboxed environment.

## High-Level Diagram

```mermaid
graph TD
    User[User] -->|ckrv task "..."| CLI[Chakravarti CLI (Host)]
    CLI -->|Parse| Workflow[Workflow Definition (YAML)]
    CLI -->|Create| Sandbox[Docker Sandbox]
    CLI -->|Mount| Workspace[Git Worktree]
    
    subgraph Host
        CLI
        Orchestrator[Orchestration Engine]
        Workflow
    end
    
    subgraph Docker Container
        Agent[Claude Code / Agent Binary]
        WorkspaceMount[Mounted Workspace]
    end
    
    Orchestrator -->|Step 1: Prompt| Agent
    Agent -->|Read/Write| WorkspaceMount
    Agent -->|Output| API[LLM API]
    Agent -->|Response| Orchestrator
    Orchestrator -->|Step 2: Prompt| Agent
```

## Core Components

### 1. Orchestrator (Host-Side)
Unlike Rover, which runs the orchestration logic *inside* the container, Chakravarti runs the Orchestrator on the **Host**. This "Smart Host, Dumb Container" approach simplifies the architecture.
- **Responsibility**: 
  - Manage the task lifecycle.
  - Render prompt templates (`handlebars`).
  - Invoke the agent inside the container (`docker exec`).
  - Parse outputs and manage state.

### 2. Workflow Engine
Workflows are defined in YAML (compatible with Rover's `swe.yml`). 
- **Templating**: Steps can access outputs from previous steps using `{{steps.<id>.outputs.<name>}}`.

### 3. Sandbox (Docker)
Code execution happens strictly inside a Docker container.
- **Image**: A standard image with `claude-code` (or other agent tools) installed.
- **Isolation**: The container has no access to the host filesystem except for:
  - The specific Task Worktree (mounted RW).
  - User credentials (mounted RO).

### 4. Persistence
All state is stored in `.ckrv/tasks/<task-id>/`. This allows for:
- **Resumability**: If a task crashes, it can be resumed.
- **Observability**: Users can inspect intermediate artifacts (`plan.md`, logs).

## Security Model

1.  **Worktree Isolation**: The agent operates on a *copy* (git worktree) of the repo, not the user's main working directory. Malicious deletions only affect the disposable worktree.
2.  **Container Isolation**: Arbitrary code execution (e.g., `npm install`, test runs) occurring during the agent's operation happens inside the container.
3.  **Credential Safety**: Credentials are mounted as Read-Only.

## Rust Implementation Strategy

- **`bollard`**: For async Docker management.
- **`handlebars`**: For prompt rendering.
- **`serde`**: For robust JSON/YAML parsing.
- **Async/Await**: The entire pipeline is async to handle long-running LLM calls and container I/O efficiently.
