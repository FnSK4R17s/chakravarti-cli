# Research: Agent Orchestration Architecture

**Context**: Porting Rover's agent orchestration (TypeScript) to Chakravarti (Rust).
**Objective**: Define the architecture for running multi-step AI workflows in a secure sandbox.

## 1. Workflow Definition (Schema)

Rover uses a YAML schema for workflows (e.g., `swe.yml`). We will adopt a compatible schema.

**Decision**: Use `serde_yaml` to parse workflow structs.

```rust
struct Workflow {
    version: String,
    name: String,
    steps: Vec<Step>,
}

struct Step {
    id: String,
    name: String,
    prompt: String, // Handlebars template
    outputs: Vec<OutputDefinition>,
}
```

**Rationale**: YAML is human-readable/writable. Handlebars is the standard for prompt templating.

## 2. Sandbox Strategy

**Rover's Approach**:
- Builds/uses a docker image containing `rover-agent` (orchestrator) and `claude-code`.
- Runs `rover-agent` *inside* the container to manage the workflow.

**Chakravarti's Approach (Proposed)**:
- **Host-Optimized Orchestration**: Run the *Orchestrator* on the Host (Chakravarti CLI).
- **Execution**: Use `docker exec` (via `bollard`) to invoke the Agent (e.g., `claude`) inside the container.
- **Artifact Access**: Since the Host mounts the workspace volume, it can directly read/write files (artifacts, outputs) without needing an internal agent binary.

**Rationale**:
- drastically simplifies distribution (no need to bundle/compile a helper binary for the guest OS).
- "Smart Host, Dumb Container" model. The container only needs the AI tool (`npm install -g @anthropic-ai/claude-code`), which is easier to maintain.

**Dependencies**:
- `bollard`: Full Docker API client for Rust (Async).
- `handlebars`: For processing prompt variables (`{{steps.plan.outputs}}`) on the Host.

## 3. Agent Integration (Claude)

**Mechanism**:
1. Host renders prompt with context.
2. Host calls `docker exec claude -p "..."`.
3. Host captures stdout/stderr.
4. Host parses output (JSON/Files).

**Requirement**: The container image must have `claude` installed.
- *Solution*: We can use a standard `node` image and install `claude-code` on startup, or maintain a `chakravarti/sandbox` image.
- *MVP*: Use `node:20-bookworm` (or similar), mount a startup script that does `npm install -g ...` if missing, or use a persistent volume for global implementations.

## 4. State Management

Rover stores state in `.rover/tasks/<id>/`.
We will use `.ckrv/tasks/<id>/`.

- `task.json`: Metadata (Status, ID).
- `workspace/`: The git worktree (mounted to container).
- `artifacts/`: Step outputs (plans, logs).

## Unknowns Resolution

- **Orchestrator Location**: Decided on HOST (simplification).
- **Docker Client**: Decided on `bollard`.
- **Templating**: Decided on `handlebars`.

## Phase 0 Complete
All major architectural decisions made.
