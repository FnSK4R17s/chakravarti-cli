# Add Gemini CLI integration

**Issue**: [#31](https://github.com/FnSK4R17s/chakravarti-cli/issues/31)
**Created**: 2026-02-24
**Status**: Draft

## Problem Statement

Add support for Google's Gemini CLI as an alternative AI coding agent in Chakravarti. This expands the agent ecosystem to include Gemini, which provides strong code generation and reasoning capabilities through Google's Gemini models.

## Current State

Chakravarti currently supports three agent providers:
- Claude Code (Anthropic) - default
- OpenAI Codex 
- Kilo Code (multi-provider)

The codebase has a clean `AgentProvider` trait abstraction in `crates/ckrv-sandbox/src/agent/mod.rs` that makes adding new agents straightforward.

## Proposed Solution

Implement a new `GeminiProvider` that conforms to the existing `AgentProvider` trait. Gemini CLI is Google's coding agent that uses Gemini models.

## User Stories

### US1: Use Gemini as agent
**As a** developer,
**I want** to use Gemini CLI as the coding agent,
**So that** I can leverage Google's Gemini models for code generation.

## Technical Approach

### Options Considered

| Option | Pros | Cons |
|--------|------|------|
| Implement GeminiProvider directly | Matches existing pattern, clean integration | Requires understanding Gemini CLI interface |
| Use OpenAI-compatible API | Faster to implement | Doesn't use Gemini CLI specifically |
| Wait for official Gemini SDK | More stable | Not available yet |

### Decision

Implement `GeminiProvider` directly following the existing pattern:
1. Add `Gemini` variant to `AgentType` enum
2. Create `gemini.rs` module implementing `AgentProvider`
3. Add to `create_agent` function

Need to research:
- Gemini CLI command-line interface
- Environment variables needed (likely `GEMINI_API_KEY`)
- Config file locations

## Implementation Notes

Key files to modify:
- `crates/ckrv-sandbox/src/agent/mod.rs` - Add variant, import module
- `crates/ckrv-sandbox/src/agent/gemini.rs` - New file

Reference existing implementations:
- `claude.rs` - Good reference for prompt handling
- `codex.rs` - Another reference for agent structure

## Open Questions

- [ ] What is the exact CLI command for Gemini CLI?
- [ ] Does Gemini CLI support the same --resume, --print flags?
- [ ] What environment variables are needed?
- [ ] Any config files to mount?

## Success Criteria

| Metric | Target |
|--------|--------|
| Builds successfully | Yes |
| Tests pass | Yes |
| Can select Gemini via --agent flag | Yes |

## Next [ ] Research Gemini CLI interface
- [ ] Steps

- Implement GeminiProvider
- [ ] Add tests
- [ ] Verify build

## References

- [Google Gemini CLI](https://cloud.google.com/gemini)
- Existing agent implementations in `crates/ckrv-sandbox/src/agent/`
