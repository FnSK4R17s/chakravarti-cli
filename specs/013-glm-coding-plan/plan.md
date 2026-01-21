# Implementation Plan: GLM Coding Plan Agent Support

**Branch**: `013-glm-coding-plan` | **Date**: 2026-01-21 | **Spec**: [spec.md](file:///apps/chakravarti-cli/specs/013-glm-coding-plan/spec.md)
**Input**: Feature specification from `/specs/013-glm-coding-plan/spec.md`

## Summary

Add Z.AI GLM Coding Plan as a new agent type following the existing OpenRouter pattern. This enables users to leverage GLM-4.7 and GLM-4.5-Air models through the Claude Code CLI with API redirection to `https://api.z.ai/api/anthropic`.

## Technical Context

**Language/Version**: Rust 1.75+, TypeScript/React  
**Primary Dependencies**: axum, tokio, serde, bollard (Docker)  
**Storage**: YAML configuration files (`~/.config/chakravarti/agents.yaml`)  
**Testing**: cargo test (Rust), vitest (TypeScript), manual Docker tests  
**Target Platform**: Linux (Docker required for sandbox)  
**Project Type**: Monorepo with Rust backend + React frontend  
**Constraints**: Must work with existing Docker images (ckrv-claude)

## Constitution Check

*GATE: All principles verified ✅*

| Principle | Requirement | Status |
|-----------|-------------|--------|
| I. Code Quality Excellence | Full typing, zero lint errors, single responsibility | ✅ Extend existing typed enums/structs |
| II. Testing Standards | TDD approach planned, coverage targets defined | ✅ Unit tests for new types |
| III. Reliability First | Error handling strategy, idempotency considered | ✅ Validation before execution |
| IV. Security by Default | No hardcoded secrets, input validation planned | ✅ API keys in config file only |
| V. Deterministic CLI Behavior | Machine-readable output, explicit exit codes | ✅ Uses existing execution patterns |

## Project Structure

### Documentation (this feature)

```text
specs/013-glm-coding-plan/
├── spec.md              # Feature specification
├── plan.md              # This file
├── research.md          # Phase 0 research findings ✅
├── data-model.md        # GLMConfig data model ✅
├── quickstart.md        # User documentation ✅
├── contracts/           # API type definitions ✅
│   └── agents.ts
└── checklists/
    └── requirements.md  # Spec quality checklist ✅
```

### Source Code (affected files)

```text
crates/
├── ckrv-ui/src/
│   ├── api/
│   │   └── agents.rs        # [MODIFY] Add ClaudeGLM type + GLMConfig
│   ├── services/
│   │   └── engine.rs        # [MODIFY] Add GLM execution path
│   └── api/
│       └── terminal.rs      # [MODIFY] Add GLM terminal config
└── ckrv-core/src/
    └── runner.rs            # [MODIFY] Add GLM support (optional)
```

---

## Proposed Changes

### Backend: Agent Configuration

#### [MODIFY] [agents.rs](file:///apps/chakravarti-cli/crates/ckrv-ui/src/api/agents.rs)

Add new `ClaudeGLM` variant to `AgentType` enum and create `GLMConfig` struct:

```rust
// Add to AgentType enum (line ~14)
/// Claude Code with Z.AI GLM Coding Plan
ClaudeGLM,

// Add new struct (after OpenRouterConfig)
/// Configuration for Z.AI GLM Coding Plan agents
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GLMConfig {
    /// Z.AI API key
    pub api_key: Option<String>,
    /// Model identifier (e.g., "glm-4.7", "glm-4.5-air")
    pub model: String,
    /// Custom timeout in ms (default: 3000000)
    pub timeout_ms: Option<u32>,
}

// Add to AgentConfig struct (line ~87)
/// GLM Coding Plan configuration (for ClaudeGLM type)
pub glm: Option<GLMConfig>,
```

Update `test_agent()` function to handle ClaudeGLM validation.

---

#### [MODIFY] [engine.rs](file:///apps/chakravarti-cli/crates/ckrv-ui/src/services/engine.rs)

Add GLM execution path in `execute_batch_in_sandbox()` (around line 700-740):

```rust
// After the OpenRouter block, add:
} else if is_glm {
    // GLM Coding Plan path: Use Claude Code CLI with Z.AI env vars
    let model_name = model.as_ref().unwrap();
    let _ = sender.send(LogMessage::new("info", &format!("Using GLM Coding Plan: {}", model_name))).await;
    
    // Get GLM API key from agent config
    let api_key = /* extract from glm config */;
    
    if let Some(key) = api_key {
        // Required env vars for Z.AI
        cfg = cfg.env("ANTHROPIC_BASE_URL", "https://api.z.ai/api/anthropic");
        cfg = cfg.env("ANTHROPIC_AUTH_TOKEN", key);
        cfg = cfg.env("ANTHROPIC_API_KEY", ""); // Must be empty
        cfg = cfg.env("API_TIMEOUT_MS", "3000000"); // Extended timeout
        
        // Set model for all tiers
        cfg = cfg.env("ANTHROPIC_DEFAULT_SONNET_MODEL", model_name);
        cfg = cfg.env("ANTHROPIC_DEFAULT_OPUS_MODEL", model_name);
        cfg = cfg.env("ANTHROPIC_DEFAULT_HAIKU_MODEL", model_name);
    }
    cfg
}
```

---

#### [MODIFY] [terminal.rs](file:///apps/chakravarti-cli/crates/ckrv-ui/src/api/terminal.rs)

Add GLM terminal configuration (around line 129-160):

```rust
// After is_openrouter check, add:
let is_glm = payload.agent.as_ref()
    .map(|a| matches!(a.agent_type, AgentType::ClaudeGLM))
    .unwrap_or(false);

// In the configuration block:
} else if is_glm {
    // GLM Coding Plan configuration
    if let Some(ref agent) = payload.agent {
        if let Some(ref glm_config) = agent.glm {
            env_vars.push("ANTHROPIC_BASE_URL=https://api.z.ai/api/anthropic".to_string());
            
            if let Some(ref api_key) = glm_config.api_key {
                env_vars.push(format!("ANTHROPIC_AUTH_TOKEN={}", api_key));
            }
            
            env_vars.push("ANTHROPIC_API_KEY=".to_string());
            env_vars.push(format!("API_TIMEOUT_MS={}", glm_config.timeout_ms.unwrap_or(3000000)));
            
            if !glm_config.model.is_empty() {
                env_vars.push(format!("ANTHROPIC_DEFAULT_SONNET_MODEL={}", glm_config.model));
                env_vars.push(format!("ANTHROPIC_DEFAULT_OPUS_MODEL={}", glm_config.model));
                env_vars.push(format!("ANTHROPIC_DEFAULT_HAIKU_MODEL={}", glm_config.model));
            }
        }
    }
    println!("Terminal session using GLM Coding Plan");
}
```

---

## Verification Plan

### Automated Tests

#### 1. Unit Tests (Rust)

**Command**: `cd crates/ckrv-ui && cargo test agents`

Tests to add in `agents.rs`:
- `test_glm_config_serialization` - Verify GLMConfig serializes/deserializes correctly
- `test_agent_type_claude_glm` - Verify ClaudeGLM variant works with serde

These tests follow the existing pattern in [tests.rs](file:///apps/chakravarti-cli/crates/ckrv-sandbox/src/agent/tests.rs).

#### 2. Build Verification

**Command**: `cargo build --release`

Ensures all Rust code compiles without errors.

### Manual Verification

#### 3. Configuration Test

1. Run `ckrv ui` to start the dashboard
2. Navigate to Agent Manager
3. Click "Add Agent" and verify "GLM Coding Plan" appears as an option
4. Fill in:
   - Name: "Test GLM Agent"
   - API Key: (valid Z.AI API key)
   - Model: `glm-4.7`
5. Save and verify agent appears in list with correct type badge

#### 4. Terminal Session Test

1. Select the GLM agent in the UI
2. Start an interactive terminal session
3. In the terminal, run: `env | grep ANTHROPIC`
4. Verify output shows:
   ```
   ANTHROPIC_BASE_URL=https://api.z.ai/api/anthropic
   ANTHROPIC_AUTH_TOKEN=<your_key>
   ANTHROPIC_API_KEY=
   ANTHROPIC_DEFAULT_SONNET_MODEL=glm-4.7
   ```

#### 5. Execution Test (requires Z.AI API key)

1. Create a simple spec with one batch: "Create a hello.txt file"
2. Select GLM agent and run execution
3. Verify logs show "Using GLM Coding Plan: glm-4.7"
4. Verify task completes successfully

---

## Complexity Tracking

No constitution violations. This is a straightforward extension of existing patterns.

| Aspect | Complexity | Notes |
|--------|------------|-------|
| New code | Low | ~50 lines of Rust across 3 files |
| Testing | Low | 2 unit tests + manual verification |
| Risk | Low | Follows proven OpenRouter pattern |
