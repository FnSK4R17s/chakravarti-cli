# Add Qwen Code Integration

**Issue**: [#34](https://github.com/FnSK4R17s/chakravarti-cli/issues/34)
**Created**: 2026-02-23
**Status**: Draft | In Progress | Ready for Spec | Archived

## Problem Statement

Chakravarti-cli currently supports Claude Code, OpenAI Codex, and Kilo Code as agent providers. Adding Qwen Code integration would expand the orchestration capabilities to include Alibaba's Qwen3-Coder models, which are powerful open-source coding models with strong agentic capabilities.

This aligns with the vision of treating agents as interchangeable workers - users should be able to route tasks to Qwen Code as another provider option.

## Current State

The `ckrv-sandbox` crate has an agent abstraction with:
- `AgentType` enum with Claude, Codex, KiloCode variants
- `AgentProvider` trait defining the interface
- Provider implementations: `ClaudeProvider`, `CodexProvider`, `KiloCodeProvider`

Qwen Code is not yet integrated.

## Proposed Solution

Add Qwen Code as a new agent provider by:
1. Adding `QwenCode` variant to `AgentType` enum
2. Creating `QwenCodeProvider` implementing `AgentProvider`
3. Adding to the `create_agent` factory function
4. Supporting both OAuth and OpenAI-compatible API authentication

## User Stories

### US1: Qwen Code Provider Support
**As a** user with Qwen API access,
**I want** to use Qwen Code as an agent provider in Chakravarti,
**So that** I can route coding tasks to Qwen3-Coder models through the orchestration engine.

### US2: Flexible Authentication
**As a** user,
**I want** to authenticate with Qwen Code via OAuth or OpenAI-compatible API,
**So that** I can use whichever authentication method suits my setup.

### US3: Headless Execution
**As a** user running automated workflows,
**I want** Qwen Code to work in headless/non-interactive mode,
**So that** it integrates with Chakravarti's fire-and-forget paradigm.

## Technical Approach

### Qwen Code CLI Overview

- **Package**: `@qwen-code/qwen-code` on npm
- **Command**: `qwen` (installed globally)
- **Runtime**: Node.js 20+
- **Authentication**:
  - Qwen OAuth (recommended, 2,000 requests/day free)
  - OpenAI-compatible API via environment variables
- **Modes**: Interactive and headless

### Environment Variables Required

```bash
# Option 1: Qwen OAuth (automatic after /auth)
# Option 2: OpenAI-compatible
export OPENAI_API_KEY="your-qwen-api-key"
export OPENAI_BASE_URL="https://api.qwen-lm.com/v1"  # or custom
export OPENAI_MODEL="qwen3-coder-plus"
```

### Options Considered

| Option | Pros | Cons | Recommendation |
|--------|------|------|----------------|
| Native CLI integration | Works with existing AgentProvider pattern, follows codebase conventions | Requires understanding Qwen's CLI flags | ✅ Recommended |
| SDK integration | More programmatic control | Adds Node.js dependency to container, more complex | Not recommended |
| HTTP API direct | No CLI dependency | Duplicates work Qwen CLI already does | Not recommended |

### Implementation Plan

1. **Add AgentType variant**: `QwenCode` to enum in `mod.rs`
2. **Create provider**: `qwen.rs` implementing `AgentProvider`
3. **Command construction**: Use `qwen` CLI with appropriate flags for headless mode
4. **Environment variables**: Support `OPENAI_API_KEY`, `OPENAI_BASE_URL`, `OPENAI_MODEL`
5. **Config mounts**: Handle `.qwen/settings.json` if present

### Qwen CLI Flags (Research Needed)

Need to verify:
- Headless/non-interactive mode flags
- Print/output format options
- Working directory specification
- Model override options

## Implementation Notes

### File Changes Required

- `crates/ckrv-sandbox/src/agent/mod.rs` - Add QwenCode variant and register provider
- `crates/ckrv-sandbox/src/agent/qwen.rs` - New file for QwenCodeProvider

### Dependencies

- No new Rust dependencies (Qwen Code runs in container via npm)
- Node.js 20+ already required for Qwen Code

### Docker Considerations

- Need to ensure Node.js is available in the sandbox image
- npm packages installed at runtime or pre-baked into image

## Open Questions

- [ ] What are the exact CLI flags for headless execution with Qwen Code?
- [ ] Does Qwen Code support `--print` or similar output mode like Claude?
- [ ] How does Qwen Code handle working directory specification?
- [ ] Is there a `--dangerously-skip-permissions` equivalent?

## Success Criteria

| Metric | Target |
|--------|--------|
| Qwen Code executes in sandbox | Yes |
| Supports both OAuth and API key auth | Yes |
| Works with existing orchestrator | Yes |
| Passes existing agent provider tests | Yes |

## Next Steps

- [ ] Research Qwen Code CLI flags for headless execution
- [ ] Create QwenCodeProvider implementation
- [ ] Add tests following existing provider test patterns
- [ ] Document configuration in agents.yaml

## References

- [Qwen Code GitHub](https://github.com/QwenLM/qwen-code)
- [Qwen Code Documentation](https://qwenlm.github.io/qwen-code-docs/en/users/overview)
- Existing agent implementations: `crates/ckrv-sandbox/src/agent/{claude,codex,kilo}.rs`
