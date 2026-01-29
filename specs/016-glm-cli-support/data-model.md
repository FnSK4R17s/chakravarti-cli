# Data Model: GLM Coding Plan CLI Support

**Feature**: 016-glm-cli-support  
**Date**: 2026-01-29

## Entities

### RunnerConfig (Extended)

New fields to add to existing `RunnerConfig` struct in `ckrv-core/src/runner.rs`:

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `glm_api_key` | `Option<String>` | No | Z.AI API key for GLM Coding Plan |
| `glm_model` | `Option<String>` | No | Model ID (glm-4.7, glm-4.5-air) |
| `glm_timeout_ms` | `Option<u32>` | No | Custom timeout (default: 3000000) |

### GLMConfig (Existing - UI)

Already exists in `ckrv-ui/src/api/agents.rs`:

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `api_key` | `Option<String>` | Yes | Z.AI API key |
| `model` | `String` | Yes | Model identifier |
| `timeout_ms` | `Option<u32>` | No | Timeout in milliseconds |

### AgentType (Existing)

Already has `ClaudeGLM` variant in UI. No changes needed.

## Relationships

```
┌─────────────────┐     loads      ┌──────────────────┐
│  agents.yaml    │ ──────────────>│   AgentConfig    │
└─────────────────┘                └──────────────────┘
                                           │
                                           │ extracts
                                           ▼
                                   ┌──────────────────┐
                                   │    GLMConfig     │
                                   └──────────────────┘
                                           │
                                           │ populates
                                           ▼
                                   ┌──────────────────┐
                                   │  RunnerConfig    │
                                   │  (glm_* fields)  │
                                   └──────────────────┘
                                           │
                                           │ injects env vars
                                           ▼
                                   ┌──────────────────┐
                                   │ Docker Container │
                                   │ or Local Process │
                                   └──────────────────┘
```

## Environment Variables Injected

When `glm_api_key` is present in `RunnerConfig`:

| Variable | Value | Purpose |
|----------|-------|---------|
| `ANTHROPIC_BASE_URL` | `https://api.z.ai/api/anthropic` | Z.AI endpoint |
| `ANTHROPIC_AUTH_TOKEN` | `<glm_api_key>` | Authentication |
| `ANTHROPIC_API_KEY` | `""` (empty) | Prevent Claude direct auth |
| `API_TIMEOUT_MS` | `<glm_timeout_ms>` or `3000000` | Request timeout |
| `ANTHROPIC_DEFAULT_SONNET_MODEL` | `<glm_model>` | Model selection |
| `ANTHROPIC_DEFAULT_OPUS_MODEL` | `<glm_model>` | Model selection |
| `ANTHROPIC_DEFAULT_HAIKU_MODEL` | `<glm_model>` | Model selection |

## Validation Rules

1. **glm_model** must be a valid GLM model ID (glm-4.7, glm-4.5-air, or custom)
2. **glm_api_key** is required if any GLM field is set
3. **glm_timeout_ms** defaults to 3000000 if not specified
4. GLM and OpenRouter are mutually exclusive (only one can be active per execution)
