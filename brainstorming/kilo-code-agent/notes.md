# Add Kilo Code Agent Integration

**Issue**: None yet (create issue when ready to implement)
**Created**: 2026-02-09
**Status**: Ready for Spec

## Problem Statement

Chakravarti-cli currently supports Claude Code and OpenAI Codex as agents. Users want access to more AI providers (Gemini, DeepSeek, Mistral, Qwen, etc.) but adding each individually requires significant effort.

Kilo Code is an open-source, multi-provider agentic CLI that supports 30+ providers through a single interface. Adding it as one agent gives users access to many models.

## Current State

**Supported agents:**
- Claude Code (Anthropic) - native CLI
- OpenAI Codex - native CLI
- Claude via OpenRouter - routes through Claude CLI
- Claude via GLM (Z.AI) - routes through Claude CLI

**Pain points:**
- Each new provider requires: Dockerfile, AgentProvider impl, CLI enum updates, docker mounts
- Users with Gemini/DeepSeek/Mistral subscriptions can't use them directly
- Open issues requesting: Gemini CLI (#31), Cursor CLI (#32), AMP CLI (#33), Qwen Code (#34), etc.

## Proposed Solution

Add Kilo Code as a single AgentProvider that unlocks 30+ AI backends through one integration.

**Kilo Code overview:**
- Open source: [github.com/Kilo-Org/kilocode](https://github.com/Kilo-Org/kilocode)
- #1 on OpenRouter, 750k+ users, 6.1T tokens/month
- CLI: `npm install -g @kilocode/cli`
- Non-interactive: `kilo run [prompt] --auto`
- Config: `~/.config/kilo/config.json`
- Auth: `kilo auth` (file-based, no env vars required)

## User Stories

### US1: Use Gemini with Chakravarti
**As a** developer with a Gemini subscription,
**I want** to configure Kilo Code with my Gemini API key,
**So that** I can use Gemini as an executor in ckrv workflows.

### US2: Fallback Between Providers
**As a** user who hits rate limits,
**I want** Kilo Code configured with multiple providers,
**So that** I can switch models without reconfiguring ckrv.

## Technical Approach

### Files to Modify

| File | Change |
|------|--------|
| `crates/ckrv-sandbox/src/agent/mod.rs` | Add `KiloCode` enum variant, update factory |
| `crates/ckrv-sandbox/src/agent/kilo.rs` | **NEW** - KiloCodeProvider implementation |
| `crates/ckrv-sandbox/src/agent/tests.rs` | Add tests for Kilo provider |
| `crates/ckrv-cli/src/services/agent_lookup.rs` | Add `KiloCode` to CLI AgentType enum |
| `crates/ckrv-sandbox/src/docker.rs` | Add Kilo config mounts (~/.config/kilo/) |
| `docker/Dockerfile.kilo` | **NEW** - Dedicated Kilo container |
| `docker/Dockerfile.agent` | Add Kilo CLI to combined image |
| `crates/docs/agent-guide.md` | Document Kilo integration |

### KiloCodeProvider Implementation

```rust
// crates/ckrv-sandbox/src/agent/kilo.rs
pub struct KiloCodeProvider;

impl AgentProvider for KiloCodeProvider {
    fn name(&self) -> &str { "Kilo Code" }
    fn agent_type(&self) -> AgentType { AgentType::KiloCode }

    fn build_command(&self, prompt: &str, workdir: &Path, config: &AgentConfig) -> Vec<String> {
        let mut cmd = vec!["kilo".into(), "run".into(), prompt.into(), "--auto".into()];
        if let Some(model) = &config.model {
            cmd.extend(["--model".into(), model.clone()]);
        }
        cmd.extend(["--cwd".into(), workdir.to_string_lossy().into()]);
        cmd
    }

    fn required_env_vars(&self) -> Vec<&str> { vec![] }  // Uses file-based auth

    fn config_mounts(&self, host_home: &str, container_home: &str) -> Vec<Mount> {
        // Mount ~/.config/kilo/
    }
}
```

### Dockerfile.kilo

```dockerfile
FROM node:22-slim
RUN apt-get update && apt-get install -y git curl ca-certificates && rm -rf /var/lib/apt/lists/*
RUN npm install -g @kilocode/cli
RUN mkdir -p /home/kilo/.config/kilo && chmod -R 777 /home/kilo
RUN mkdir -p /workspace && chmod 777 /workspace
WORKDIR /workspace
ENV HOME=/home/kilo
RUN kilo --version || true
CMD ["/bin/bash"]
```

### YAML Configuration

```yaml
agents:
  - id: kilo-agent
    name: Kilo Code
    agent_type: kilo_code
    enabled: true
    description: Multi-provider agentic coding
```

### Decision

Follow existing Claude/Codex patterns exactly:
- Same trait implementation structure
- Same Docker mount approach
- Same test patterns

Key differences:
- Uses `kilo run [prompt] --auto` (not `--print`)
- Config in `~/.config/kilo/` (XDG-compliant)
- No required env vars (file-based auth)

## Implementation Notes

### Kilo CLI Reference

| Command | Description |
|---------|-------------|
| `kilo run [message] --auto` | Non-interactive execution |
| `kilo auth` | Configure credentials |
| `kilo models [provider]` | List available models |

### Permission Config (~/.config/kilo/config.json)

```json
{
  "permission": {
    "*": "allow",
    "bash": "allow",
    "edit": "allow"
  }
}
```

### Execution Order

1. `kilo.rs` (new file, no deps)
2. `mod.rs` (add module, enum, factory)
3. `tests.rs` (add test cases)
4. `agent_lookup.rs` (CLI enum)
5. `Dockerfile.kilo` (new file)
6. `Dockerfile.agent` (add kilo)
7. `docker.rs` (add mounts)
8. `agent-guide.md` (docs)

## Open Questions

- [x] Is there a GitHub issue for this? → No, needs to be created
- [x] What CLI flags does Kilo use? → `kilo run [prompt] --auto --cwd [dir]`
- [x] Where are Kilo credentials stored? → `~/.config/kilo/config.json`
- [ ] Does Kilo support streaming output? → Need to verify `--output-format` options

## Success Criteria

| Metric | Target |
|--------|--------|
| Build passes | `cargo build --workspace` succeeds |
| Tests pass | All new Kilo tests pass |
| Docker image builds | `docker build -f Dockerfile.kilo` succeeds |
| End-to-end test | `ckrv task run --agent kilo-agent -p "echo hello"` works |

## Next Steps

- [ ] Create GitHub issue to track this work
- [ ] Implement KiloCodeProvider
- [ ] Create Dockerfile.kilo
- [ ] Add tests
- [ ] Update documentation

## References

- [Kilo Code GitHub](https://github.com/Kilo-Org/kilocode)
- [Kilo CLI Docs](https://kilo.ai/docs/cli)
- [Provider Configuration](https://github.com/Kilo-Org/kilocode/blob/main/cli/docs/PROVIDER_CONFIGURATION.md)
- [Existing Agent Guide](crates/docs/agent-guide.md)
- [Full Implementation Plan](/home/sk4r/.claude/plans/atomic-frolicking-stallman.md)
