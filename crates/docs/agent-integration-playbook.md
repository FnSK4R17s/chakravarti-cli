---
last_commit: 2a2da7f
last_updated: 2026-03-02
related_files:
  - crates/ckrv-transport/src/types/agents.rs
  - crates/ckrv-transport/src/handlers/agents.rs
  - crates/ckrv-transport/src/axum/agents.rs
  - crates/ckrv-tauri/src/commands/agents.rs
  - crates/ckrv-tauri/src/main.rs
  - crates/ckrv-cli/src/services/agent_lookup.rs
  - crates/ckrv-sandbox/src/agent/mod.rs
  - crates/ckrv-ui/frontend/src/components/AgentManager.tsx
  - crates/ckrv-ui/frontend/src/lib/api.ts
  - crates/ckrv-ui/frontend/src/types/api.generated.ts
  - docker/Dockerfile.agent
---

# Agent Integration Playbook

> Full-stack guide for adding a new coding agent to Chakravarti — from Rust types to frontend UI to Docker container.

For the single-crate sandbox-layer quickstart, see [Agent Extensibility Guide](agent-guide.md). This playbook covers the **complete cross-crate integration** including both backends (Axum + Tauri), the frontend, type generation, persistence, and container wiring.

---

## 1. Architecture & Data Flow

### How agent config flows through the stack

```
┌─────────────────────────────────────────────────────────────────────┐
│  User Config                                                         │
│  ~/.config/chakravarti/agents.yaml                                   │
└──────────────────┬──────────────────────────────────────────────────┘
                   │ serde_yaml
                   ▼
┌─────────────────────────────────────────────────────────────────────┐
│  ckrv-transport  (shared business logic)                             │
│  types/agents.rs   → AgentType enum, AgentConfig struct, req/res     │
│  handlers/agents.rs → list, upsert, delete, set-role, test, models   │
└──────┬──────────────────────┬───────────────────────────────────────┘
       │                      │
       ▼                      ▼
┌──────────────┐    ┌──────────────────┐    ┌─────────────────────────┐
│  Axum routes │    │  Tauri commands   │    │  CLI (ckrv-cli)         │
│  axum/       │    │  ckrv-tauri/      │    │  services/              │
│  agents.rs   │    │  commands/        │    │  agent_lookup.rs        │
│              │    │  agents.rs        │    │  (parallel AgentType    │
│  HTTP JSON   │    │  IPC JSON         │    │   enum for CLI lookup)  │
└──────┬───────┘    └────────┬─────────┘    └──────────┬──────────────┘
       │                     │                         │
       ▼                     ▼                         ▼
┌──────────────────────────────────────┐    ┌─────────────────────────┐
│  React Frontend                       │    │  ckrv-sandbox           │
│  AgentManager.tsx  (config UI)        │    │  AgentProvider trait    │
│  AgentCliModal.tsx (terminal)         │    │  Docker execution       │
│  api.ts            (fetch intercept)  │    └─────────────────────────┘
│  api.generated.ts  (ts-rs types)      │
└───────────────────────────────────────┘
```

### Execution path (when an agent actually runs code)

```
ckrv run --agent <name>
  → CLI loads agents.yaml via agent_lookup.rs
  → Resolves AgentConfig for the named agent
  → ckrv-sandbox creates AgentProvider from AgentType
  → Provider.build_command() generates CLI invocation
  → Provider.config_mounts() provides Docker bind-mounts
  → Docker container executes the agent CLI
  → Provider.parse_output() normalizes the result
```

### Transport layer architecture

The **ckrv-transport** crate is the single source of truth for agent types and business logic. Both Axum and Tauri are thin wrappers:

- **Axum** (`crates/ckrv-transport/src/axum/agents.rs`): HTTP route handlers that call shared handler functions.
- **Tauri** (`crates/ckrv-tauri/src/commands/agents.rs`): `#[tauri::command]` wrappers that call the same shared handler functions.
- **Frontend** (`crates/ckrv-ui/frontend/src/lib/api.ts`): A global fetch interceptor routes `/api/*` calls through Tauri IPC when running as a desktop app, or through HTTP when running as a web app.

---

## 2. Complete File-Path Inventory

Every file where agent references appear, grouped by layer:

### Transport types & logic (source of truth)

| File | What lives here |
|------|-----------------|
| `crates/ckrv-transport/src/types/agents.rs` | `AgentType` enum, `AgentConfig`, provider-specific config structs (`OpenRouterConfig`, `GlmConfig`, `KiloCodeConfig`), model types (`OpenRouterModel`, `GlmModel`, `KiloCodeModel`), all request/response types |
| `crates/ckrv-transport/src/handlers/agents.rs` | `AgentsFile` (persistence format), `AgentFileConfig` (with `binary_path`, `extra_args`, `env_vars`), all handler functions: `list_agents_handler`, `upsert_agent_handler`, `delete_agent_handler`, `set_default_agent_handler`, `set_qa_agent_handler`, `set_test_writer_agent_handler`, `test_agent_handler`, `get_openrouter_models_handler`, `get_kilo_models_handler`, `get_glm_models_handler` |
| `crates/ckrv-transport/src/types/mod.rs` | Re-exports agent types |

### Axum backend

| File | What lives here |
|------|-----------------|
| `crates/ckrv-transport/src/axum/agents.rs` | HTTP route definitions, Axum-specific request body structs, `routes()` function |

### Tauri backend

| File | What lives here |
|------|-----------------|
| `crates/ckrv-tauri/src/commands/agents.rs` | `#[tauri::command]` wrappers, response wrapper structs (`ListAgentsWrapped`, `ModelsWrapped`, `KiloModelsWrapped`) |
| `crates/ckrv-tauri/src/main.rs` | `generate_handler![]` macro registration (lines 81-89) |

### CLI

| File | What lives here |
|------|-----------------|
| `crates/ckrv-cli/src/services/agent_lookup.rs` | **Parallel** `AgentType` enum, `AgentConfig`, config loading, `find_default_agent()`, `find_qa_agent()`, `find_test_writer_agent()` |
| `crates/ckrv-cli/src/commands/run.rs` | `--agent` CLI flag |

### Sandbox (Docker execution)

| File | What lives here |
|------|-----------------|
| `crates/ckrv-sandbox/src/agent/mod.rs` | `AgentProvider` trait, `AgentType` enum (sandbox-local), `create_agent()` factory |
| `crates/ckrv-sandbox/src/agent/claude.rs` | `ClaudeProvider` implementation |
| `crates/ckrv-sandbox/src/agent/codex.rs` | `CodexProvider` implementation |
| `crates/ckrv-sandbox/src/agent/kilo.rs` | `KiloCodeProvider` implementation |

### Frontend

| File | What lives here |
|------|-----------------|
| `crates/ckrv-ui/frontend/src/types/api.generated.ts` | Auto-generated TypeScript types from `ts-rs` |
| `crates/ckrv-ui/frontend/src/components/AgentManager.tsx` | Agent management UI (add/edit/delete/test/role-assign) |
| `crates/ckrv-ui/frontend/src/components/AgentCliModal.tsx` | Interactive terminal for agent execution |
| `crates/ckrv-ui/frontend/src/lib/api.ts` | `endpointToCommand` mapping (Tauri IPC routing) |

### Docker

| File | What lives here |
|------|-----------------|
| `docker/Dockerfile.agent` | Multi-agent container (Claude + Codex + Kilo) |
| `docker/Dockerfile.claude` | Claude-only container |
| `docker/Dockerfile.codex` | Codex-only container |
| `docker/Dockerfile.kilo` | Kilo-only container |

### Configuration

| File | What lives here |
|------|-----------------|
| `~/.config/chakravarti/agents.yaml` | User's agent configurations (runtime) |

---

## 3. Endpoint / IPC Parity Matrix

Every agent-related operation must work identically in both Axum (web) and Tauri (desktop). The frontend fetch interceptor in `api.ts` maps HTTP endpoints to Tauri `invoke()` commands.

| Operation | Axum Route | Tauri Command | `api.ts` Mapping | Status |
|-----------|-----------|---------------|------------------|--------|
| List agents | `GET /agents` | `list_agents` | `/api/agents` → `list_agents` | Parity |
| Upsert agent | `POST /agents/upsert` | `upsert_agent` | `/api/agents/upsert` → `upsert_agent` | Parity |
| Delete agent | `POST /agents/delete` | `delete_agent` | `/api/agents/delete` → `delete_agent` | Parity |
| Set default | `POST /agents/set-default` | `set_default_agent` | `/api/agents/set-default` → `set_default_agent` | Parity |
| Set QA agent | `POST /agents/set-qa` | `set_qa_agent` | `/api/agents/set-qa` → `set_qa_agent` | Parity |
| Set test writer | `POST /agents/set-test-writer` | `set_test_writer_agent` | `/api/agents/set-test-writer` → `set_test_writer_agent` | Parity |
| Test agent | `POST /agents/test` | `test_agent` | `/api/agents/test` → `test_agent` | Parity |
| OpenRouter models | `GET /agents/models` | `get_openrouter_models` | `/api/agents/models` → `get_openrouter_models` | Parity |
| Kilo models | `GET /agents/kilo-models` | `get_kilo_models` | `/api/agents/kilo-models` → `get_kilo_models` | Parity |
| GLM models | `GET /agents/glm-models` | **Missing** | `/api/agents/glm-models` → `get_glm_models` | **GAP** |

> **Known issue**: `get_glm_models` has an Axum route and a handler but is **not registered** in Tauri's `generate_handler![]` macro in `crates/ckrv-tauri/src/main.rs`. The frontend `api.ts` also lacks a mapping for `/api/agents/glm-models`. Desktop GLM model listing will return a 501. Fix: implement the Tauri command wrapper and register it.

---

## 4. Integration Checklist — Adding a New Agent

Use this checklist end-to-end when integrating a new agent (e.g., `Amp`, `Cursor`, `Windsurf`).

### 4.1 Transport types (`ckrv-transport`)

- [ ] Add variant to `AgentType` enum in `crates/ckrv-transport/src/types/agents.rs`
- [ ] Create provider-specific config struct (e.g., `AmpConfig`) with `#[derive(Debug, Clone, Serialize, Deserialize)]` and `#[cfg_attr(feature = "typescript", derive(TS))]`
- [ ] Add `Option<AmpConfig>` field to `AgentConfig` struct
- [ ] If the agent has a model catalog API, create a model type (e.g., `AmpModel`) with `ts-rs` derive
- [ ] If needed, create request/response types for new endpoints

### 4.2 Handler logic (`ckrv-transport`)

- [ ] Update `test_agent_handler()` in `crates/ckrv-transport/src/handlers/agents.rs` to handle the new `AgentType` variant (CLI binary detection, version check)
- [ ] If the agent has a model API: implement `get_amp_models_handler()` with fallback list
- [ ] Update `ensure_defaults()` if the new agent should be auto-created
- [ ] Add unit tests for serialization roundtrip and handler logic

### 4.3 Axum routes (`ckrv-transport`)

- [ ] If new endpoint needed: add route function in `crates/ckrv-transport/src/axum/agents.rs`
- [ ] Register the route in the `routes()` function

### 4.4 Tauri commands (`ckrv-tauri`)

- [ ] Add `#[tauri::command]` wrapper in `crates/ckrv-tauri/src/commands/agents.rs`
- [ ] If new model type: add response wrapper struct (e.g., `AmpModelsWrapped`)
- [ ] **Register** the command in `generate_handler![]` in `crates/ckrv-tauri/src/main.rs` (lines 81-89)
- [ ] Import new types from `ckrv_transport::types`

### 4.5 CLI agent lookup (`ckrv-cli`)

- [ ] Add variant to the **parallel** `AgentType` enum in `crates/ckrv-cli/src/services/agent_lookup.rs`
- [ ] If provider-specific config: add config struct mirroring transport types
- [ ] Add the field to CLI's `AgentConfig` struct

### 4.6 Sandbox provider (`ckrv-sandbox`)

- [ ] Create `crates/ckrv-sandbox/src/agent/amp.rs` implementing `AgentProvider` trait
- [ ] Implement `name()`, `agent_type()`, `build_command()`, `required_env_vars()`, `config_mounts()`, `parse_output()`
- [ ] Add variant to sandbox's `AgentType` enum and `create_agent()` factory in `crates/ckrv-sandbox/src/agent/mod.rs`
- [ ] Add `mod amp;` declaration

### 4.7 TypeScript types (generated)

- [ ] Run `cargo test -p ckrv-transport --features typescript export_typescript_types -- --ignored` to regenerate `crates/ckrv-ui/frontend/src/types/api.generated.ts`
- [ ] Verify the new `AgentType` variant appears in the generated TypeScript union
- [ ] Verify any new config/model types are exported

### 4.8 Frontend (`ckrv-ui/frontend`)

- [ ] Update `AgentManager.tsx`: add agent type to the type selector dropdown
- [ ] Add provider-specific config form fields (model selector, API key input, etc.)
- [ ] If new model endpoint: add TanStack Query hook (`useQuery`) to fetch models
- [ ] Add icon/label mapping for the new agent type
- [ ] Update `api.ts` `endpointToCommand` mapping if new endpoint was added
- [ ] Test in both web (Axum) and desktop (Tauri) modes

### 4.9 Docker container

- [ ] Create `docker/Dockerfile.amp` for standalone container, or update `docker/Dockerfile.agent` to include the new CLI
- [ ] Create non-root user (required — agent CLIs reject security flags as root)
- [ ] Install the agent CLI binary
- [ ] Set up config directories with correct ownership
- [ ] Verify with `docker build -t ckrv-amp docker/ -f docker/Dockerfile.amp`

### 4.10 Persistence

- [ ] Verify `agents.yaml` serialization/deserialization roundtrip with the new agent type
- [ ] Verify `agent_type` field uses `serde(rename_all = "snake_case")` consistently
- [ ] If `AgentFileConfig` (in handlers) has fields not in `AgentConfig` (in types), ensure both are updated

### 4.11 Tests

- [ ] Add serialization test in `crates/ckrv-transport/src/types/agents.rs`
- [ ] Add handler test in `crates/ckrv-transport/src/handlers/agents.rs`
- [ ] Add `AgentProvider` tests in `crates/ckrv-sandbox/src/agent/`
- [ ] Run `cargo test --workspace` — all existing tests must pass
- [ ] Manually test agent via UI: add agent → select model → test connection → assign role

---

## 5. Common Pitfalls

### 5.1 Generated type drift

The `api.generated.ts` file is produced by `ts-rs` from Rust structs with `#[cfg_attr(feature = "typescript", derive(TS))]`. If you change a Rust type but forget to regenerate, the frontend will silently use stale types.

**Fix**: Always regenerate after changing any `#[derive(TS)]` type:
```bash
cargo test -p ckrv-transport --features typescript export_typescript_types -- --ignored
```

> **Note**: The generated file (`api.generated.ts`) may lag behind the actual runtime types used in `AgentManager.tsx`. The component currently defines its own inline TypeScript interfaces that may differ from the generated ones. Always check both.

### 5.2 Missing `<select>` items in the frontend

The `AgentManager.tsx` agent type dropdown is a hardcoded list. Adding a new `AgentType` variant in Rust without updating the TSX select options means users can't create agents of that type through the UI.

**Fix**: Search for `agent_type` or `AgentType` in `AgentManager.tsx` and update every switch/select/conditional.

### 5.3 Missing Tauri command registration

Adding a `#[tauri::command]` function is not enough. It must also be listed in the `generate_handler![]` macro in `main.rs`. A missing registration causes a runtime error (Tauri returns "command not found") that only manifests in the desktop app.

**Fix**: After adding any Tauri command, immediately add it to `generate_handler![]` in `crates/ckrv-tauri/src/main.rs`.

### 5.4 Missing `api.ts` endpoint mapping

If you add a new Axum route (e.g., `GET /agents/amp-models`) but don't add a mapping in `api.ts`'s `endpointToCommand`, the desktop app will return a 501 for that endpoint while the web app works fine.

**Fix**: For every new Axum route, add a corresponding entry in `endpointToCommand` in `crates/ckrv-ui/frontend/src/lib/api.ts`.

### 5.5 Parallel `AgentType` enum in CLI

The `ckrv-cli` crate has its **own** `AgentType` enum in `agent_lookup.rs` that must stay in sync with the canonical enum in `ckrv-transport/src/types/agents.rs`. Missing a variant here means the CLI can't look up or match agents of the new type.

**Fix**: When adding a variant to the transport enum, immediately mirror it in `crates/ckrv-cli/src/services/agent_lookup.rs`.

### 5.6 Docker user permissions

Agent CLIs (especially Claude Code) refuse to run certain flags (like `--dangerously-skip-permissions`) when running as root/sudo. Always create a non-root user in Dockerfiles.

**Fix**: Every agent Dockerfile must include:
```dockerfile
RUN useradd -m -s /bin/bash -d /home/agentname agentname
USER agentname
```

### 5.7 Auth credential mounts

Each agent stores credentials differently. Forgetting to mount the right host path into the container means the agent can't authenticate.

| Agent | Host Path | Container Path | Env Vars |
|-------|-----------|---------------|----------|
| Claude | `~/.claude/` | `/home/claude/.claude/` | — |
| Claude+OpenRouter | — | — | `ANTHROPIC_BASE_URL`, `ANTHROPIC_AUTH_TOKEN` |
| Claude+GLM | — | — | `ANTHROPIC_BASE_URL`, `ANTHROPIC_AUTH_TOKEN`, `API_TIMEOUT_MS` |
| Codex | `~/.codex/`, `~/.config/openai/` | `/home/codex/.codex/`, `/home/codex/.config/openai/` | `OPENAI_API_KEY` |
| Kilo | `~/.config/kilo/` | `/home/kilo/.config/kilo/` | — (file-based via `kilo auth`) |

### 5.8 Response wrapper mismatch

Tauri commands return raw structs, but the frontend expects a specific JSON shape. Axum handlers wrap responses in `{ "agents": [...] }` or `{ "models": [...] }`. Tauri wrapper structs (`ListAgentsWrapped`, `ModelsWrapped`, etc.) must produce the same shape.

**Fix**: When adding a new list endpoint, create a `#[derive(Serialize)]` wrapper struct in the Tauri commands file that matches the Axum JSON envelope.

---

## 6. Validation Protocol

### Targeted checks before submitting

```bash
# 1. Compile all crates
cargo build --workspace

# 2. Run all tests
cargo test --workspace

# 3. Regenerate TypeScript types and verify no drift
cargo test -p ckrv-transport --features typescript export_typescript_types -- --ignored
git diff crates/ckrv-ui/frontend/src/types/api.generated.ts
# If diff is non-empty, the generated types changed — commit the update.

# 4. Check that frontend compiles
cd crates/ckrv-ui/frontend && npx tsc --noEmit && cd -

# 5. Verify Tauri builds (if Tauri deps available)
cd crates/ckrv-tauri && cargo build && cd -

# 6. Lint
cargo clippy --workspace -- -D warnings

# 7. Format
cargo fmt --all -- --check
```

### Parity smoke test

After implementing a new agent, verify these scenarios in **both** web and desktop modes:

1. **List**: New agent type appears in agent list
2. **Create**: Can create an agent of the new type via the UI
3. **Model select**: If model endpoint exists, models load in the dropdown
4. **Test**: "Test Connection" works and reports success/failure
5. **Role assign**: Can set the agent as default / QA / test-writer
6. **Delete**: Can delete the agent
7. **CLI**: `ckrv run --agent <new-agent-id>` resolves the agent

### CI expectations

- `cargo test --workspace` must pass (includes serialization roundtrip tests)
- `cargo clippy --workspace` must pass with no warnings
- Frontend `tsc --noEmit` must pass
- `api.generated.ts` must be up-to-date (CI can diff against freshly generated output)

---

## 7. New Agent Onboarding Template

Copy this template and fill in the blanks to integrate a new agent. Replace `Amp` / `amp` with your agent name.

### Step 1: Transport types

```rust
// In crates/ckrv-transport/src/types/agents.rs

// 1a. Add to AgentType enum
pub enum AgentType {
    // ... existing variants ...
    /// Amp coding agent
    Amp,
}

// 1b. Create config struct
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "typescript", derive(TS))]
#[cfg_attr(feature = "typescript", ts(export))]
pub struct AmpConfig {
    pub api_key: Option<String>,
    pub model: String,
    // ... agent-specific fields ...
}

// 1c. Add field to AgentConfig
pub struct AgentConfig {
    // ... existing fields ...
    pub amp: Option<AmpConfig>,
}

// 1d. If model catalog needed, create model type
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "typescript", derive(TS))]
#[cfg_attr(feature = "typescript", ts(export))]
pub struct AmpModel {
    pub id: String,
    pub name: String,
    pub context_length: Option<u32>,
}
```

### Step 2: Handler

```rust
// In crates/ckrv-transport/src/handlers/agents.rs

// 2a. Update test_agent_handler match arm
AgentType::Amp => {
    // Check if `amp` CLI is installed
    match tokio::process::Command::new("amp")
        .arg("--version")
        .output()
        .await
    {
        Ok(output) if output.status.success() => {
            let version = String::from_utf8_lossy(&output.stdout);
            Ok(TestAgentResponse {
                success: true,
                message: format!("Amp CLI found: {}", version.trim()),
            })
        }
        _ => Ok(TestAgentResponse {
            success: false,
            message: "Amp CLI not found. Install: npm install -g @amp/cli".into(),
        }),
    }
}

// 2b. If model endpoint needed:
pub async fn get_amp_models_handler() -> Result<Vec<AmpModel>, TransportError> {
    // Fetch from API or run `amp models` command
    // Include a hardcoded fallback list
    todo!()
}
```

### Step 3: Axum route

```rust
// In crates/ckrv-transport/src/axum/agents.rs

// 3a. Add route function
async fn get_amp_models() -> impl IntoResponse {
    match get_amp_models_handler().await {
        Ok(models) => Json(serde_json::json!({ "models": models })).into_response(),
        Err(e) => e.into_response(),
    }
}

// 3b. Register in routes()
pub fn routes() -> Router<AppState> {
    Router::new()
        // ... existing routes ...
        .route("/agents/amp-models", get(get_amp_models))
}
```

### Step 4: Tauri command

```rust
// In crates/ckrv-tauri/src/commands/agents.rs

// 4a. Add wrapper struct
#[derive(Serialize)]
pub struct AmpModelsWrapped {
    models: Vec<AmpModel>,
}

// 4b. Add command
#[tauri::command]
pub async fn get_amp_models() -> Result<AmpModelsWrapped, String> {
    get_amp_models_handler()
        .await
        .map(|models| AmpModelsWrapped { models })
        .map_err(|e| e.to_string())
}
```

```rust
// In crates/ckrv-tauri/src/main.rs — add to generate_handler![]
commands::agents::get_amp_models,
```

### Step 5: CLI enum sync

```rust
// In crates/ckrv-cli/src/services/agent_lookup.rs
pub enum AgentType {
    // ... existing variants ...
    Amp,
}
```

### Step 6: Sandbox provider

```rust
// Create crates/ckrv-sandbox/src/agent/amp.rs
impl AgentProvider for AmpProvider {
    fn name(&self) -> &str { "Amp" }
    fn agent_type(&self) -> AgentType { AgentType::Amp }
    fn build_command(&self, prompt: &str, workdir: &Path, config: &AgentConfig) -> Vec<String> {
        vec!["amp".into(), "--auto".into(), "--prompt".into(), prompt.into()]
    }
    fn required_env_vars(&self) -> Vec<&str> { vec!["AMP_API_KEY"] }
    fn config_mounts(&self, host_home: &str, container_home: &str) -> Vec<Mount> {
        vec![Mount {
            target: Some(format!("{container_home}/.amp")),
            source: Some(format!("{host_home}/.amp")),
            typ: Some(bollard::models::MountTypeEnum::BIND),
            read_only: Some(true),
            ..Default::default()
        }]
    }
    fn parse_output(&self, stdout: &str, stderr: &str, exit_code: i32) -> Result<AgentOutput> {
        Ok(AgentOutput { success: exit_code == 0, stdout: stdout.into(), stderr: stderr.into(), exit_code })
    }
}
```

### Step 7: Regenerate TS types

```bash
cargo test -p ckrv-transport --features typescript export_typescript_types -- --ignored
```

### Step 8: Frontend

```typescript
// In crates/ckrv-ui/frontend/src/components/AgentManager.tsx

// 8a. Add to agent type selector options
{ value: 'amp', label: 'Amp' }

// 8b. Add config form fields (conditional on agent_type === 'amp')
{agentType === 'amp' && (
  <AmpConfigFields config={ampConfig} onChange={setAmpConfig} />
)}

// 8c. If model endpoint: add query hook
const { data: ampModels } = useQuery({
  queryKey: ['amp-models'],
  queryFn: () => fetch('/api/agents/amp-models').then(r => r.json()),
  enabled: agentType === 'amp',
});
```

```typescript
// In crates/ckrv-ui/frontend/src/lib/api.ts — add mapping
'/api/agents/amp-models': 'get_amp_models',
```

### Step 9: Docker

```dockerfile
# docker/Dockerfile.amp
FROM node:22-slim
RUN apt-get update && apt-get install -y git curl ca-certificates && rm -rf /var/lib/apt/lists/*
RUN npm install -g @amp/cli
RUN useradd -m -s /bin/bash -d /home/amp amp && \
    mkdir -p /home/amp/.amp && \
    chown -R amp:amp /home/amp
RUN mkdir -p /workspace && chown amp:amp /workspace
WORKDIR /workspace
ENV HOME=/home/amp
USER amp
CMD ["/bin/bash"]
```

### Step 10: YAML config example

```yaml
# Add to ~/.config/chakravarti/agents.yaml
agents:
  - id: amp-default
    name: Amp Agent
    agent_type: amp
    level: 4
    is_default: false
    enabled: true
    description: Amp coding agent
    amp:
      api_key: null  # Set via env or UI
      model: amp-default-model
```

### Step 11: Tests

```rust
// In crates/ckrv-transport/src/types/agents.rs
#[test]
fn test_amp_agent_type_serialization() {
    let agent_type = AgentType::Amp;
    let json = serde_json::to_string(&agent_type).unwrap();
    assert_eq!(json, "\"amp\"");
}
```

### Step 12: Verify

Run the full validation protocol from Section 6.
