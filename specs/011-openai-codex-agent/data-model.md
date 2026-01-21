# Data Model: OpenAI Codex CLI Agent

**Feature**: 011-openai-codex-agent

## Entities

### AgentType (Enum)

Represents the available agent implementations.

| Variant | Description |
|---------|-------------|
| `Claude` | Claude Code CLI (Anthropic) |
| `Codex` | OpenAI Codex CLI |

### AgentConfig (Struct)

User configuration for agent selection.

| Field | Type | Description | Validation |
|-------|------|-------------|------------|
| `agent_type` | `AgentType` | Selected agent | Required, defaults to Claude |
| `model` | `Option<String>` | Model override | Optional |
| `api_key_env` | `String` | Environment variable name for API key | Must be set in environment |

### AgentProvider (Trait)

Interface for agent implementations.

| Method | Returns | Description |
|--------|---------|-------------|
| `name()` | `&str` | Human-readable agent name |
| `build_command(prompt, workdir)` | `Vec<String>` | CLI command and arguments |
| `required_env_vars()` | `Vec<&str>` | Required environment variables |
| `config_mounts()` | `Vec<Mount>` | Docker mounts for config files |
| `parse_output(output)` | `Result<AgentOutput>` | Parse CLI output |

### AgentOutput (Struct)

Normalized output from agent execution.

| Field | Type | Description |
|-------|------|-------------|
| `success` | `bool` | Whether execution succeeded |
| `stdout` | `String` | Standard output |
| `stderr` | `String` | Standard error |
| `files_modified` | `Vec<String>` | List of modified files |
| `token_usage` | `Option<TokenUsage>` | API token consumption |

### TokenUsage (Struct)

Token consumption metrics.

| Field | Type | Description |
|-------|------|-------------|
| `input_tokens` | `u64` | Tokens in prompt |
| `output_tokens` | `u64` | Tokens in response |
| `model` | `String` | Model used |

## State Transitions

```
AgentConfig (from CLI/config)
        │
        ▼
AgentProvider::build_command()
        │
        ▼
Docker Execution
        │
        ▼
AgentProvider::parse_output()
        │
        ▼
AgentOutput (normalized result)
```

## Relationships

```
AgentConfig 1:1 AgentType
AgentType 1:1 AgentProvider (trait implementation)
AgentProvider 1:N AgentOutput (per execution)
```
