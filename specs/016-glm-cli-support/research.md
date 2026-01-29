# Research: GLM Coding Plan CLI Support

**Date**: 2026-01-29  
**Feature**: 016-glm-cli-support  
**Status**: Complete

## Existing Pattern Analysis (MANDATORY)

**Similar Feature**: OpenRouter Integration  
**Search Command**: `grep -rn "openrouter" crates/ --include="*.rs" | cut -d: -f1 | sort -u`

### OpenRouter Implementation Locations

| Crate | File | Purpose |
|-------|------|---------|
| ckrv-core | runner.rs | CLI execution - RunnerConfig fields + env var injection |
| ckrv-cli | commands/run.rs | Config loading from agents.yaml |
| ckrv-cli | commands/task.rs | Task execution with agent selection |
| ckrv-cli | services/agent_lookup.rs | Agent config file loading |
| ckrv-ui | api/agents.rs | UI types (AgentType, OpenRouterConfig) |
| ckrv-ui | api/terminal.rs | Interactive terminal env var setup |
| ckrv-ui | services/engine.rs | Batch execution env var setup |
| ckrv-ui | api/plans.rs | Plan generation with agent |
| ckrv-ui | api/qa.rs | QA commands with agent |
| ckrv-ui | api/test.rs | Test commands with agent |

### GLM Current Implementation Locations

**Search Command**: `grep -rn "glm\|GLM\|ClaudeGLM" crates/ --include="*.rs" | cut -d: -f1 | sort -u`

| Crate | File | Status |
|-------|------|--------|
| ckrv-ui | api/agents.rs | ✅ Implemented |
| ckrv-ui | api/terminal.rs | ✅ Implemented |
| ckrv-ui | services/engine.rs | ✅ Implemented |
| ckrv-core | runner.rs | ❌ **MISSING** |
| ckrv-cli | commands/task.rs | ❌ **MISSING** |
| ckrv-cli | services/agent_lookup.rs | ⚠️ Partial (type exists but no env var handling) |

### CLI/UI Parity Check

| Path | OpenRouter | GLM | Gap |
|------|------------|-----|-----|
| **CLI Execution** (`ckrv run`, `ckrv task`) | ✅ Full | ❌ Missing | Critical |
| **UI Batch Execution** (`engine.rs`) | ✅ Full | ✅ Full | None |
| **UI Terminal** (`terminal.rs`) | ✅ Full | ✅ Full | None |

**Conclusion**: GLM MUST be added to:
1. `ckrv-core/src/runner.rs` - Add glm_* fields to RunnerConfig + env var injection
2. `ckrv-cli/src/commands/task.rs` - Load GLM config and pass to runner
3. `ckrv-cli/src/services/agent_lookup.rs` - Ensure GLMConfig is handled

## Research Questions

### Q1: How is OpenRouter config structured in RunnerConfig?

**Answer**: Found in `ckrv-core/src/runner.rs` lines 29-34:
```rust
pub struct RunnerConfig {
    // ... other fields ...
    pub openrouter_api_key: Option<String>,
    pub openrouter_model: Option<String>,
    pub openrouter_base_url: Option<String>,
}
```

**GLM Pattern**: Same pattern with glm_ prefix:
```rust
pub glm_api_key: Option<String>,
pub glm_model: Option<String>,
pub glm_timeout_ms: Option<u32>,  // GLM-specific
```

### Q2: How are OpenRouter env vars injected?

**Answer**: Found in `ckrv-core/src/runner.rs` lines 300-320:
```rust
if let Some(ref api_key) = self.config.openrouter_api_key {
    cfg = cfg.env("ANTHROPIC_BASE_URL", base_url);
    cfg = cfg.env("ANTHROPIC_AUTH_TOKEN", api_key);
    cfg = cfg.env("ANTHROPIC_API_KEY", "");
    // ... model env vars
}
```

**GLM Pattern**: Same structure with Z.AI base URL:
```rust
if let Some(ref api_key) = self.config.glm_api_key {
    cfg = cfg.env("ANTHROPIC_BASE_URL", "https://api.z.ai/api/anthropic");
    cfg = cfg.env("ANTHROPIC_AUTH_TOKEN", api_key);
    cfg = cfg.env("ANTHROPIC_API_KEY", "");
    cfg = cfg.env("API_TIMEOUT_MS", timeout);
    // ... model env vars
}
```

### Q3: Where is agent config loaded for CLI?

**Answer**: `ckrv-cli/src/services/agent_lookup.rs` loads from `~/.config/chakravarti/agents.yaml`

The `find_agent_config()` function returns `AgentConfig` which includes:
- `agent_type: AgentType` (includes `ClaudeGLM`)
- `glm: Option<GLMConfig>` (already exists in UI types)

**Gap**: The CLI commands don't extract GLM fields and pass them to `RunnerConfig`.

## Summary

GLM CLI support requires mirroring the OpenRouter pattern exactly:

1. **Add GLM fields to `RunnerConfig`** (ckrv-core)
2. **Add GLM env var injection** in both sandbox and non-sandbox paths (ckrv-core)
3. **Load GLM config in CLI commands** and pass to RunnerConfig (ckrv-cli)

This is a straightforward pattern extension with ~50 lines of new code.
