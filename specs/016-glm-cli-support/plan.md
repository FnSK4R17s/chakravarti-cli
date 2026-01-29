# Implementation Plan: GLM Coding Plan CLI Support

**Branch**: `016-glm-cli-support` | **Date**: 2026-01-29 | **Spec**: [spec.md](./spec.md)
**Input**: Feature specification from `/specs/016-glm-cli-support/spec.md`

## Summary

Add GLM Coding Plan support to CLI commands (`ckrv run`, `ckrv task`) by extending `RunnerConfig` in `ckrv-core` with GLM fields and environment variable injection. This restores CLI/UI parity by mirroring the existing OpenRouter pattern.

## Technical Context

**Language/Version**: Rust 1.75+  
**Primary Dependencies**: tokio, bollard (Docker), serde  
**Storage**: YAML configuration (`~/.config/chakravarti/agents.yaml`)  
**Testing**: cargo test, manual Docker tests  
**Target Platform**: Linux (Docker required)  
**Project Type**: Monorepo with Rust crates  
**Constraints**: Must follow existing OpenRouter pattern exactly

## Constitution Check

*GATE: All principles verified ✅*

| Principle | Requirement | Status |
|-----------|-------------|--------|
| I. Code Quality Excellence | Full typing, zero lint errors, single responsibility | ✅ Extends existing typed structs |
| II. Testing Standards | TDD approach planned, coverage targets defined | ✅ Unit tests for new config fields |
| III. Reliability First | Error handling strategy, idempotency considered | ✅ Validation before execution |
| IV. Security by Default | No hardcoded secrets, input validation planned | ✅ API keys from config only |
| V. Deterministic CLI Behavior | Machine-readable output, explicit exit codes | ✅ Uses existing patterns |

## Project Structure

### Documentation (this feature)

```text
specs/016-glm-cli-support/
├── spec.md              # Feature specification ✅
├── plan.md              # This file ✅
├── research.md          # Pattern analysis ✅
├── data-model.md        # Data model ✅
├── quickstart.md        # User guide ✅
├── contracts/           # Type definitions ✅
│   └── types.ts
└── checklists/
    └── requirements.md  # Quality checklist ✅
```

### Source Code (affected files)

```text
crates/
├── ckrv-core/src/
│   └── runner.rs            # [MODIFY] Add glm_* fields + env var injection
├── ckrv-cli/src/
│   ├── commands/
│   │   ├── run.rs           # [MODIFY] Load GLM config, pass to runner
│   │   └── task.rs          # [MODIFY] Load GLM config, pass to runner
│   └── services/
│       └── agent_lookup.rs  # [VERIFY] GLMConfig loading
└── ckrv-ui/src/
    └── api/agents.rs        # [VERIFY] Types already exist
```

---

## Proposed Changes

### Phase 1: Core Runner Extension

#### [MODIFY] [runner.rs](file:///apps/chakravarti-cli/crates/ckrv-core/src/runner.rs)

**1. Add GLM fields to `RunnerConfig` struct (after line 34):**

```rust
/// Z.AI API key (for Claude Code + GLM Coding Plan mode).
pub glm_api_key: Option<String>,
/// GLM model ID (for Claude Code + GLM Coding Plan mode).
pub glm_model: Option<String>,
/// GLM timeout in ms (default: 3000000).
pub glm_timeout_ms: Option<u32>,
```

**2. Update `Default` impl (after line 48):**

```rust
glm_api_key: None,
glm_model: None,
glm_timeout_ms: None,
```

**3. Add GLM env var injection in `run_steps_local()` (after OpenRouter block ~line 325):**

```rust
// Set GLM Coding Plan environment variables if configured
if let Some(ref api_key) = self.config.glm_api_key {
    let timeout = self.config.glm_timeout_ms.unwrap_or(3000000);
    
    tracing::info!(
        model = ?self.config.glm_model,
        "Using GLM Coding Plan for Claude Code"
    );
    
    cfg = cfg.env("ANTHROPIC_BASE_URL", "https://api.z.ai/api/anthropic");
    cfg = cfg.env("ANTHROPIC_AUTH_TOKEN", api_key);
    cfg = cfg.env("ANTHROPIC_API_KEY", "");
    cfg = cfg.env("API_TIMEOUT_MS", timeout.to_string());
    
    if let Some(ref model) = self.config.glm_model {
        cfg = cfg.env("ANTHROPIC_DEFAULT_SONNET_MODEL", model);
        cfg = cfg.env("ANTHROPIC_DEFAULT_OPUS_MODEL", model);
        cfg = cfg.env("ANTHROPIC_DEFAULT_HAIKU_MODEL", model);
    }
}
```

**4. Add GLM env var injection in sandbox path `run_steps_sandboxed()` (after OpenRouter block ~line 400):**

Same pattern as above but using Docker env vars.

---

### Phase 2: CLI Command Updates

#### [MODIFY] [run.rs](file:///apps/chakravarti-cli/crates/ckrv-cli/src/commands/run.rs)

Add GLM config loading after OpenRouter loading:

```rust
// Load GLM config if agent is ClaudeGLM
if matches!(agent_config.agent_type, AgentType::ClaudeGLM) {
    if let Some(ref glm_config) = agent_config.glm {
        runner_config.glm_api_key = glm_config.api_key.clone();
        runner_config.glm_model = Some(glm_config.model.clone());
        runner_config.glm_timeout_ms = glm_config.timeout_ms;
    }
}
```

#### [MODIFY] [task.rs](file:///apps/chakravarti-cli/crates/ckrv-cli/src/commands/task.rs)

Same pattern as run.rs for task execution.

---

## Verification Plan

### Automated Tests

#### 1. Unit Tests (Rust)

**Command**: `cd crates/ckrv-core && cargo test runner`

Add tests:
- `test_runner_config_glm_fields` - Verify GLMConfig fields serialize/deserialize
- `test_runner_config_glm_defaults` - Verify default values are None

#### 2. Build Verification

**Command**: `cargo build --workspace`

### Manual Verification

#### 3. CLI Test

1. Add GLM agent to `~/.config/chakravarti/agents.yaml`
2. Run `ckrv task run --agent "test-glm" -p "create hello.txt"`
3. Verify logs show "Using GLM Coding Plan"
4. Verify task completes (requires valid Z.AI key)

#### 4. Parity Test

1. Run same task via UI with GLM agent
2. Run same task via CLI with GLM agent
3. Compare logs and behavior

---

## Complexity Tracking

No constitution violations. This is a straightforward pattern extension.

| Aspect | Complexity | Notes |
|--------|------------|-------|
| New code | Low | ~60 lines of Rust across 3 files |
| Testing | Low | 2 unit tests + manual verification |
| Risk | Low | Follows proven OpenRouter pattern exactly |
