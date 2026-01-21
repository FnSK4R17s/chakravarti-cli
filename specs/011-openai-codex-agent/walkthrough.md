# Walkthrough: OpenAI Codex CLI Agent Implementation

**Feature**: 011-openai-codex-agent  
**Branch**: `011-openai-codex-agent`  
**Completed**: 2026-01-21

## Summary

Successfully implemented OpenAI Codex CLI as an alternative agent to Claude Code, enabling users to choose between AI coding assistants for batch executions.

## Changes Made

### Agent Abstraction Layer (`crates/ckrv-sandbox/src/agent/`)

| File | Purpose |
|------|---------|
| [mod.rs](file:///apps/chakravarti-cli/crates/ckrv-sandbox/src/agent/mod.rs) | `AgentProvider` trait, `AgentType` enum, `AgentConfig`, factory functions |
| [claude.rs](file:///apps/chakravarti-cli/crates/ckrv-sandbox/src/agent/claude.rs) | Claude Code CLI provider implementation |
| [codex.rs](file:///apps/chakravarti-cli/crates/ckrv-sandbox/src/agent/codex.rs) | OpenAI Codex CLI provider implementation |
| [tests.rs](file:///apps/chakravarti-cli/crates/ckrv-sandbox/src/agent/tests.rs) | 14 unit tests for agent module |

### CLI Integration

| File | Change |
|------|--------|
| [run.rs](file:///apps/chakravarti-cli/crates/ckrv-cli/src/commands/run.rs) | Added `--agent` CLI flag (default: claude) |

### Execution Engine

| File | Change |
|------|--------|
| [engine.rs](file:///apps/chakravarti-cli/crates/ckrv-ui/src/services/engine.rs) | Added `is_codex` detection, Codex CLI command construction, `OPENAI_API_KEY` env var handling |

### Docker

| File | Change |
|------|--------|
| [Dockerfile.agent](file:///apps/chakravarti-cli/docker/Dockerfile.agent) | Install both Claude Code and Codex CLIs, create config directories |

## Validation Results

### Unit Tests
```
test result: ok. 14 passed; 0 failed
```

### Build Status
```
cargo check --workspace: ✅ Success (warnings only)
```

## Usage

```bash
# Use Claude (default)
ckrv run

# Use Codex
ckrv run --agent=codex

# Use Codex with specific model
OPENAI_API_KEY=sk-... ckrv run --agent=codex --executor-model=gpt-5.2-codex
```

## Agent Detection Logic

The engine auto-detects agent type based on model name:
- **Codex**: Model starts with `gpt-`, `codex`, `openai/`, or contains `codex`
- **OpenRouter**: Model contains `/` but isn't Claude or Codex
- **Claude**: All other cases (default)
