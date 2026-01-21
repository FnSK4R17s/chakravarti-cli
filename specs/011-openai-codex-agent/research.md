# Research: OpenAI Codex CLI Agent Integration

**Feature**: 011-openai-codex-agent
**Date**: 2026-01-21

## Research Questions

### 1. Codex CLI Interface and Command Syntax

**Decision**: Codex CLI uses similar pattern to Claude Code CLI with `codex` command

**Findings**:
- Codex CLI is invoked with `codex` command
- Shows version info: `OpenAI Codex (v0.87.0)`
- Supports model selection: `gpt-5.2-codex medium` (shown in user's terminal output)
- Has `/model` command for model changes
- Has `/approvals` for permission settings
- Has `/init` for creating AGENTS.md
- Interactive mode with stdin/stdout

**Command mapping**:
| Claude Code | Codex CLI | Notes |
|-------------|-----------|-------|
| `claude --print "prompt"` | `codex --print "prompt"` | Assumed similar flag |
| `--dangerously-skip-permissions` | `--auto-approve` or similar | Needs verification |
| `--output-format stream-json` | `--output-format json` | Needs verification |
| `--verbose` | `--verbose` | Assumed standard |

**Alternatives Considered**:
- Direct OpenAI API calls: Rejected (want CLI tool parity with Claude)
- Custom wrapper script: Rejected (adds complexity, prefer native CLI)

### 2. Codex CLI Installation in Docker

**Decision**: Install via npm in Dockerfile alongside Claude Code

**Rationale**:
- Codex CLI is distributed as npm package (`@openai/codex`)
- Same installation pattern as Claude Code
- Both can coexist in same Docker image

**Installation command**:
```bash
npm install -g @openai/codex
```

**Alternatives Considered**:
- Separate Docker images per agent: Rejected (increases complexity, storage)
- Runtime installation: Rejected (slow, network dependency during execution)

### 3. Authentication Pattern

**Decision**: Environment variable `OPENAI_API_KEY` (standard OpenAI pattern)

**Rationale**:
- Consistent with OpenAI ecosystem
- Matches existing Claude pattern (ANTHROPIC_API_KEY)
- Already supported by Codex CLI natively

**Environment Variables**:
| Variable | Agent | Purpose |
|----------|-------|---------|
| `OPENAI_API_KEY` | Codex | Authentication |
| `OPENAI_MODEL` | Codex | Model selection (optional) |
| `ANTHROPIC_API_KEY` | Claude | Authentication |
| `ANTHROPIC_AUTH_TOKEN` | Claude | Alternative auth |

### 4. Agent Provider Abstraction

**Decision**: Rust trait `AgentProvider` with implementations per agent

**Rationale**:
- Rust idiomatic pattern for polymorphism
- Testable via mock implementations
- Extensible for future agents (Gemini, local LLMs)

**Interface Design**:
```rust
pub trait AgentProvider: Send + Sync {
    fn name(&self) -> &str;
    fn build_command(&self, prompt: &str, workdir: &Path) -> Vec<String>;
    fn required_env_vars(&self) -> Vec<&str>;
    fn config_mounts(&self) -> Vec<Mount>;
    fn parse_output(&self, output: &str) -> Result<AgentOutput>;
}
```

**Alternatives Considered**:
- Enum with match: Rejected (not extensible without code changes)
- Configuration-driven: Rejected (too complex for 2 agents)

### 5. Agent Selection Mechanism

**Decision**: CLI flag `--agent=codex|claude` with config file fallback

**Rationale**:
- Explicit per-execution control
- Configuration file for persistent default
- Environment variable for CI/CD scenarios

**Priority order**:
1. CLI flag: `--agent=codex`
2. Environment: `CKRV_DEFAULT_AGENT=codex`
3. Config file: `.chakravarti/config.yaml` → `agent: codex`
4. Hardcoded default: `claude`

## Open Questions (Resolved)

| Question | Resolution |
|----------|------------|
| Does Codex support non-interactive mode? | Yes, `--print` flag based on `codex` help |
| Output format for streaming? | Standard JSON assumed, needs runtime verification |
| Permission model? | Uses `/approvals` command, need `--auto-approve` equivalent |

## Next Steps

1. Create `AgentProvider` trait in `ckrv-sandbox/src/agent/mod.rs`
2. Implement `ClaudeProvider` extracting existing logic
3. Implement `CodexProvider` with parallel structure
4. Modify Docker build to install both CLIs
5. Add `--agent` flag to CLI and propagate through execution stack
