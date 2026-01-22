# Research: Test and QA Commands

**Feature**: 014-test-qa-commands  
**Date**: 2026-01-21  
**Status**: Complete

## Research Questions

### Q1: How are existing CLI commands structured?

**Decision**: Follow the established pattern in `crates/ckrv-cli/src/commands/`

**Rationale**: Each command is a separate file with:
- A `clap::Args` struct for arguments
- An `execute()` async function as entry point
- Output structs with `Serialize` for JSON mode
- Support for `--json` global flag

**Reference**: [verify.rs](file:///apps/chakravarti-cli/crates/ckrv-cli/src/commands/verify.rs) - 420 lines, handles lint/type/test checks

### Q2: How is project type detection handled?

**Decision**: Reuse existing `detect_project_type()` from verify.rs

**Rationale**: The verify command already has robust detection for:
- Rust (Cargo.toml)
- Python (pyproject.toml, requirements.txt, setup.py)
- TypeScript (tsconfig.json)
- JavaScript (package.json)
- Go (go.mod)

**Enhancement needed**: Add Makefile fallback check before Unknown

### Q3: How are role-based agents accessed?

**Decision**: Agents are configured in `~/.config/chakravarti/agents.yaml` with role fields

**Rationale**: `AgentConfig` struct in [agents.rs](file:///apps/chakravarti-cli/crates/ckrv-ui/src/api/agents.rs) has:
- `is_qa_agent: bool` - For QA review agent
- `is_test_writer: bool` - For test writing agent

**Implementation**: Add helper functions to find agents by role:
```rust
fn find_test_writer_agent(agents: &[AgentConfig]) -> Option<&AgentConfig> {
    agents.iter().find(|a| a.is_test_writer && a.enabled)
}

fn find_qa_agent(agents: &[AgentConfig]) -> Option<&AgentConfig> {
    agents.iter().find(|a| a.is_qa_agent && a.enabled)
}
```

### Q4: How is sandbox execution triggered for agents?

**Decision**: Use existing engine.rs patterns from ckrv-ui

**Rationale**: [engine.rs](file:///apps/chakravarti-cli/crates/ckrv-ui/src/services/engine.rs) handles:
- Docker container creation
- Environment variable injection
- Agent-specific configuration (Claude, OpenRouter, GLM, Codex)
- Credential mounting

**Pattern**: Extract reusable sandbox execution logic into a shared module that both UI engine and CLI commands can use.

### Q5: How should diff vs main be computed?

**Decision**: Use git2 crate or shell out to `git diff main...HEAD`

**Rationale**: Need to:
1. Get list of changed files
2. Get actual diff content for QA review
3. Support `--base` flag for alternative comparison branch

**Implementation**: 
```rust
fn get_changed_files(base: &str) -> Result<Vec<PathBuf>> {
    let output = Command::new("git")
        .args(["diff", "--name-only", &format!("{}...HEAD", base)])
        .output()?;
    // Parse file list
}
```

### Q6: What existing tests exist for CLI commands?

**Finding**: No dedicated test files in `crates/ckrv-cli/src/commands/`

**Decision**: Create test module with unit tests for new commands

**Approach**:
- Mock agent configuration loading
- Mock sandbox execution
- Test Markdown output generation
- Integration tests require Docker

## Architecture Decision

### Shared Services

Extract test framework detection and sandbox invocation into shared services:

```
crates/ckrv-cli/src/
├── commands/
│   ├── test.rs      # New: ckrv test command
│   ├── qa.rs        # New: ckrv qa command
│   └── verify.rs    # Existing
├── services/
│   ├── mod.rs
│   ├── agent_lookup.rs   # Find agents by role
│   ├── test_runner.rs    # Framework detection + execution
│   └── sandbox.rs        # Docker sandbox (shared with UI)
```

### Report Generation

Both test and QA commands output Markdown:

```rust
struct MarkdownReport {
    title: String,
    sections: Vec<ReportSection>,
}

impl MarkdownReport {
    fn render(&self) -> String { /* ... */ }
}
```

## Summary

| Question | Decision |
|----------|----------|
| Command structure | Follow verify.rs pattern |
| Project detection | Reuse + add Makefile fallback |
| Agent lookup | Query by role fields |
| Sandbox execution | Extract shared module |
| Diff computation | Git CLI via subprocess |
| Test strategy | Unit tests + manual integration |
