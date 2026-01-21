# Research: GLM Coding Plan Agent Integration

**Date**: 2026-01-21  
**Feature**: 013-glm-coding-plan  
**Status**: Complete

## Research Questions

### Q1: How does Z.AI GLM Coding Plan integrate with Claude Code CLI?

**Decision**: Use Anthropic-compatible API redirection (same as OpenRouter pattern)

**Rationale**: Z.AI's documentation at https://docs.z.ai/devpack/tool/claude#manual-configuration shows that GLM Coding Plan uses the same environment variable pattern as OpenRouter:

```json
{
  "env": {
    "ANTHROPIC_AUTH_TOKEN": "your_zai_api_key",
    "ANTHROPIC_BASE_URL": "https://api.z.ai/api/anthropic",
    "API_TIMEOUT_MS": "3000000"
  }
}
```

This is nearly identical to OpenRouter's integration pattern, differing only in:
- Base URL: `https://api.z.ai/api/anthropic` (vs `https://openrouter.ai/api`)
- Models: `glm-4.7`, `glm-4.5-air` (vs OpenRouter's model catalog)

**Alternatives Considered**:
- Native GLM CLI: Z.AI doesn't provide a standalone CLI; Claude Code with API redirect is the official approach
- Separate API client: Would require new implementation; using Claude Code CLI maintains consistency

### Q2: What is the relationship between sandbox AgentType and UI AgentType?

**Decision**: Add GLM support to UI-layer `AgentType` only (not sandbox layer)

**Rationale**: The codebase has two `AgentType` enums:
1. `ckrv-sandbox/src/agent/mod.rs` - For native CLI agents (Claude, Codex)
2. `ckrv-ui/src/api/agents.rs` - For UI/API configuration including API-redirected agents

GLM Coding Plan (like OpenRouter) uses the Claude Code CLI with environment variable overrides. It's not a new CLI agent but a configuration variant. The existing pattern for `ClaudeOpenRouter` in the UI layer confirms this approach.

**Implementation**: Add `ClaudeGLM` variant to UI `AgentType` enum, create `GLMConfig` struct similar to `OpenRouterConfig`.

### Q3: Where does agent configuration get applied to execution?

**Decision**: Extend existing OpenRouter patterns in:
- `crates/ckrv-ui/src/services/engine.rs` (batch execution)
- `crates/ckrv-ui/src/api/terminal.rs` (interactive sessions)
- `crates/ckrv-core/src/runner.rs` (workflow runner - optional)

**Rationale**: These three files handle the environment variable injection for OpenRouter. The pattern is:
1. Check if agent type is OpenRouter
2. Extract API key from config
3. Set `ANTHROPIC_BASE_URL`, `ANTHROPIC_AUTH_TOKEN`, `ANTHROPIC_API_KEY=""`, and model env vars

GLM follows the exact same pattern with different base URL and optional timeout.

### Q4: What tests exist for agent integration?

**Decision**: Follow existing test patterns in:
- `crates/ckrv-sandbox/src/agent/tests.rs` - Unit tests for AgentType, AgentConfig, AgentProvider
- Integration tests would require actual Z.AI API access (manual verification)

**Test Strategy**:
1. Add unit tests for `AgentType::ClaudeGLM` parsing and display
2. Add unit tests for `GLMConfig` validation
3. Manual integration test with Z.AI API key

### Q5: What GLM models are available?

**Decision**: Support these models based on Z.AI documentation:

| Model | Environment Variable Value | Use Case |
|-------|---------------------------|----------|
| GLM-4.7 | `glm-4.7` | Primary coding model (set as opus/sonnet) |
| GLM-4.5-Air | `glm-4.5-air` | Faster/lighter model (set as haiku) |

**Mapping to Claude tiers**:
- `ANTHROPIC_DEFAULT_OPUS_MODEL=glm-4.7`
- `ANTHROPIC_DEFAULT_SONNET_MODEL=glm-4.7`
- `ANTHROPIC_DEFAULT_HAIKU_MODEL=glm-4.5-air` (or same as selected)

## Summary

GLM Coding Plan integration is a straightforward extension of the existing OpenRouter pattern. Key differences:
- Base URL: `https://api.z.ai/api/anthropic`
- Additional env var: `API_TIMEOUT_MS` (optional, default 3000000)
- Model names: `glm-4.7`, `glm-4.5-air`
