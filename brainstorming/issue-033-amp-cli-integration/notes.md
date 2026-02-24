# Add AMP CLI integration

**Issue**: [#33](https://github.com/FnSK4R17s/chakravarti-cli/issues/33)
**Created**: 2026-02-24
**Status**: Draft | In Progress | Ready for Spec | Archived

## Problem Statement

Need to integrate with Amp CLI (https://ampcode.com/) as an additional agent provider. Amp is a frontier coding agent with competitive pricing and unique features (Oracle, Librarian, etc.). Adding support will give users more choice in AI coding assistants.

## Current State

The sandbox currently supports:
- Claude Code CLI (Anthropic)
- OpenAI Codex CLI
- Kilo Code CLI (multi-provider)

Each agent implements the `AgentProvider` trait in `crates/ckrv-sandbox/src/agent/`.

## Proposed Solution

Create a new `AmpProvider` struct that implements `AgentProvider`, similar to existing providers. Add `Amp` to the `AgentType` enum with string parsing support.

## User Stories

### US1: Use Amp as Agent Provider
**As a** user of Chakravarti CLI,
**I want** to select Amp as my AI agent provider,
**So that** I can leverage Amp's features (Oracle, Librarian, shareable walkthroughs) for spec-driven development.

### US2: Configure Amp via agents.yaml
**As a** user,
**I want** to configure Amp with custom settings in agents.yaml,
**So that** I can use my existing Amp subscription and access specific models.

## Technical Approach

### Options Considered

| Option | Pros | Cons |
|--------|------|------|
| Option A: Full AmpProvider implementation | Complete feature support, follows existing pattern | More code to maintain |
| Option B: Use Amp as model backend for existing provider | Reuses code | May not capture Amp-specific features |
| Option C: External tool wrapper | Quick to implement | Limited integration, loses sandbox benefits |

### Decision

Option A: Full AmpProvider implementation - follows the existing `AgentProvider` pattern, allows Amp-specific features to be exposed later.

### Implementation Plan

1. Add `Amp` variant to `AgentType` enum in `mod.rs`
2. Add string parsing in `AgentType::from_str()`
3. Add `display_name()` entry
4. Create `amp.rs` module with `AmpProvider` struct
5. Implement `AgentProvider` trait for `AmpProvider`
6. Add `AmpProvider` to `create_agent()` match statement
7. Add tests for parsing and basic functionality

## Implementation Notes

- Amp CLI installation: `curl -fsSL https://ampcode.com/install.sh | bash`
- Amp uses `amp` command (not `ampcode`)
- Check Amp's CLI flags: `amp --help`
- Amp has unique features: Oracle (code analysis), Librarian (context management), walkthroughs
- Config file location: likely `~/.config/amp/` or similar

## Open Questions

- [ ] What are the required environment variables for Amp? (API key?)
- [ ] What CLI flags does `amp` support for non-interactive execution?
- [ ] Where does Amp store config files in the home directory?
- [ ] What is the output format for programmatic parsing?

## Success Criteria

| Metric | Target |
|--------|--------|
| Amp available as agent type | `ckrv run --agent amp` works |
| Configuration via agents.yaml | `amp` type can be configured |
| Integration tests pass | `cargo test` passes |
| Documentation updated | Agent guide mentions Amp |

## Next Steps

- [ ] Research Amp CLI flags and configuration
- [ ] Create AmpProvider skeleton
- [ ] Implement AgentProvider trait
- [ ] Add tests
- [ ] Update documentation

## References

- https://ampcode.com/ - Amp product homepage
- https://ampcode.com/manual - Amp Owner's Manual
- https://ampcode.com/models - Available models
- Existing agent implementations: `crates/ckrv-sandbox/src/agent/{claude,codex,kilo}.rs`
