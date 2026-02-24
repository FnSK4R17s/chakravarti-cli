# Add Qwen Code Integration

**Issue**: [#34](https://github.com/FnSK4R17s/chakravarti-cli/issues/34)
**Created**: 2026-02-24
**Status**: Draft | In Progress | Ready for Spec | Archived

## Problem Statement

Chakravarti currently supports Claude Code, OpenAI Codex, and Kilo Code as AI agent providers. The request is to add support for Qwen Code, an open-source AI agent from Alibaba Cloud that provides an OpenAI-compatible API and CLI interface. This would allow users to leverage Qwen's coding capabilities within the Chakravarti orchestration engine.

## Current State

- Chakravarti supports three agent providers: Claude Code, OpenAI Codex, and Kilo Code
- Agent abstraction via `AgentProvider` trait in `crates/ckrv-sandbox/src/agent/mod.rs`
- Each provider implements: `name()`, `agent_type()`, `build_command()`, `required_env_vars()`, `config_mounts()`, `parse_output()`
- `AgentType` enum currently has: `Claude`, `Codex`, `KiloCode`

## Proposed Solution

Add Qwen Code as a new agent provider following the existing pattern:
1. Create `crates/ckrv-sandbox/src/agent/qwen.rs` implementing `AgentProvider` trait
2. Add `Qwen` variant to `AgentType` enum
3. Add to `AgentType::from_str()` parser
4. Add to `create_agent()` function
5. Support both CLI and OpenAI-compatible API modes

## User Stories

### US1: Use Qwen Code CLI in Chakravarti
**As a** user,
**I want** to use Qwen Code as my AI agent provider,
**So that** I can leverage Qwen's coding capabilities within Chakravarti's orchestration.

### US2: Use Qwen via OpenAI-compatible API
**As a** user,
**I want** to connect to Qwen models via OpenAI-compatible API endpoints,
**So that** I can use Qwen3-Coder models from various providers (Ollama, OpenRouter, ModelStudio).

## Technical Approach

### Options Considered

| Option | Pros | Cons |
|--------|------|------|
| Qwen Code CLI only | Native experience, OAuth support | Requires Node.js 20+, not always installed |
| OpenAI-compatible API only | Works anywhere with API key | Requires API setup, no free tier in CLI |
| Both CLI and API (Recommended) | Maximum flexibility, supports all use cases | More implementation complexity |

### Decision

Support both Qwen Code CLI and OpenAI-compatible API modes:
- **CLI mode**: Use `qwen` command (requires Node.js 20+)
- **API mode**: Use OpenAI-compatible endpoints with Qwen models

Implementation will follow the existing provider pattern with:
- Environment variable support: `OPENAI_API_KEY`, `QWEN_AUTH_TOKEN`, `OPENAI_BASE_URL`
- Config mounts for `~/.qwen/` directory
- Parse output in standard format

## Implementation Notes

- Qwen Code requires Node.js 20+ (document in prerequisites)
- Qwen provides OAuth (free tier) or API key authentication
- OpenAI-compatible API supports: OpenAI, Anthropic, Gemini-compatible endpoints
- For API mode: Use `qwen/qwen3-coder` model names
- Qwen Code CLI has built-in terminal UI - need `--yes` flag for non-interactive
- Headless mode: Use `qwen --yes --approval-mode=auto` for automated execution
- Alternative: Use the OpenAI-compatible API directly without CLI

### Key Files to Modify

1. `crates/ckrv-sandbox/src/agent/mod.rs` - Add Qwen variant, update `from_str()`, `create_agent()`
2. `crates/ckrv-sandbox/src/agent/qwen.rs` - New file implementing `AgentProvider`

### Dependencies

- Node.js 20+ for CLI mode (for API mode, just needs network access)
- Qwen Code CLI: `npm install -g @qwen-code/qwen-code`

## Open Questions

- [ ] Should we prioritize CLI or API mode implementation first?
- [ ] How to handle the interactive OAuth flow in containerized environments?
- [ ] Should we add Qwen-specific config options (session management, etc.)?

## Success Criteria

| Metric | Target |
|--------|--------|
| AgentType::from_str("qwen") works | Returns Qwen variant |
| build_command produces valid qwen command | CLI executes successfully |
| required_env_vars includes OPENAI_API_KEY | For API mode |
| Integration test passes | Agent executes in sandbox |

## Next Steps

- [ ] Create `qwen.rs` provider implementation
- [ ] Add Qwen to AgentType enum and parser
- [ ] Update create_agent() function
- [ ] Add integration test
- [ ] Document in agent guide

## References

- [Qwen Code GitHub](https://github.com/QwenLM/qwen-code)
- [Qwen Code Docs](https://qwenlm.github.io/qwen-code-docs/)
- [Qwen Code VSCode Extension](https://marketplace.visualstudio.com/items?itemName=qwenlm.qwen-code-vscode-ide-companion)
- [OpenAI-compatible API setup](https://qwenlm.github.io/qwen-code-docs/en/users/installation/)
