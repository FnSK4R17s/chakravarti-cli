# Add Qwen Code Agent Integration

**Issue**: [#34](https://github.com/FnSK4R17s/chakravarti-cli/issues/34)
**Created**: 2026-02-23
**Status**: Draft

## Problem Statement

Alibaba's Qwen3-Coder is among the highest-performing open-weight coding models available today, yet chakravarti-cli has no native integration for it. Users with DashScope (Alibaba Cloud) subscriptions cannot direct Qwen Code at their specs without routing through a third-party intermediary. This breaks the vision principle of matching the right model to the right job—Qwen3-Coder is a legitimate L4/L5-calibre executor that users are actively requesting (#34 was filed alongside Gemini CLI, Cursor CLI, Amp, and Opencode).

Two workarounds exist today—Kilo Code and OpenRouter both expose Qwen models—but neither gives users a clean, direct integration with DashScope credentials and the full Qwen Code CLI feature set.

## Current State

**Supported agents:**
| Agent | Auth | Integration |
|-------|------|-------------|
| Claude Code | `~/.claude.json` | Native CLI + Docker |
| OpenAI Codex | `~/.codex/`, `OPENAI_API_KEY` | Native CLI + Docker |
| Kilo Code | `~/.config/kilo/` file-based | Native CLI + Docker |
| Claude + OpenRouter | `ANTHROPIC_AUTH_TOKEN` env | Env-var wrapper |
| Claude + GLM | `ZAI_API_KEY` env | Env-var wrapper |

**Current Qwen access paths (workarounds):**
- Kilo Code with model `kilo/qwen/qwen3-coder:free` — already in the fallback model list (`handlers/agents.rs:578`)
- OpenRouter via `claude_openrouter` agent type with an OpenRouter model that routes to Qwen

**Pain points:**
- Users pay for DashScope directly but must route through Kilo or OpenRouter, adding latency and cost
- No `agent_type: qwen_code` in `agents.yaml` — harder to discover and configure
- Qwen Code CLI has its own non-interactive mode, streaming format, and model flags that are not exercised when accessed through Kilo
- Every other top-tier native coding CLI (Claude Code, Codex) has its own `AgentProvider`; Qwen Code deserves parity

## Proposed Solution

Add `QwenCodeProvider` as a first-class `AgentProvider` implementation following the exact pattern established by `KiloCodeProvider`. This gives users:

1. `agent_type: qwen_code` in `agents.yaml`
2. A dedicated `Dockerfile.qwen` for isolated container execution
3. Direct `DASHSCOPE_API_KEY` auth (no Kilo or OpenRouter account required)
4. Native Qwen Code CLI flags (model selection, non-interactive mode, streaming)
5. Documentation in `agent-guide.md`

This does **not** replace Kilo Code's Qwen access—users who already use Kilo can keep doing so. This adds a direct path for users who have DashScope subscriptions and want native integration.

## User Stories

### US1: Direct DashScope Execution
**As a** developer with a DashScope API key,
**I want** to configure `agent_type: qwen_code` in my `agents.yaml`,
**So that** I can use Qwen3-Coder directly without routing through Kilo Code or OpenRouter.

### US2: Model-Level Task Assignment
**As a** user orchestrating parallel tasks,
**I want** to assign routine tasks to `qwen_code` with `qwen-coder-turbo` and complex tasks to Claude,
**So that** I minimize cost while maintaining quality on the tasks that matter.

### US3: Isolated Container Execution
**As a** user running `ckrv run`,
**I want** Qwen Code to execute inside a Docker sandbox on its own git worktree,
**So that** Qwen Code agent output is isolated from my main branch in the same way Claude and Codex are.

### US4: UI Agent Manager Discovery
**As a** user browsing the Agent Manager in `ckrv ui`,
**I want** to see Qwen Code as a first-class agent option with its own configuration card,
**So that** I can configure and enable it without reading the CLI docs.

## Technical Approach

### Options Considered

| Option | Pros | Cons |
|--------|------|------|
| **A: Native QwenCodeProvider** | Direct DashScope auth, full CLI flag control, parity with Claude/Codex, no Kilo dependency | Requires maintaining a separate Dockerfile; CLI API must be verified |
| **B: Document Kilo route** | Zero new code, works today via `kilo/qwen/qwen3-coder:free` | No `agent_type: qwen_code`, users discover this via docs only; Kilo dependency |
| **C: OpenRouter wrapper** | Also zero new code, already supported via `claude_openrouter` | Routes through Claude CLI, not a Qwen-native path; OpenRouter subscription required |

### Decision

**Option A** — native `QwenCodeProvider`. The project vision explicitly calls for orchestrating all AI subscriptions together. A DashScope subscriber shouldn't need a Kilo or OpenRouter account to use their Qwen access in ckrv. The `AgentProvider` trait was designed exactly for this, the Kilo implementation is a complete reference, and the work is scoped to ~6h following the established pattern.

Options B and C are documented as interim workarounds for users who want Qwen access today.

### Files to Modify

| File | Change |
|------|--------|
| `crates/ckrv-sandbox/src/agent/qwen.rs` | **NEW** — `QwenCodeProvider` implementation |
| `crates/ckrv-sandbox/src/agent/mod.rs` | Add `QwenCode` variant, update `from_str`, `create_agent` |
| `crates/ckrv-sandbox/src/agent/tests.rs` | Add Qwen-specific test cases |
| `crates/ckrv-cli/src/services/agent_lookup.rs` | Add `QwenCode` to CLI `AgentType` enum |
| `docker/Dockerfile.qwen` | **NEW** — dedicated Qwen Code container |
| `docker/Dockerfile.agent` | Add Qwen Code CLI to combined image |
| `crates/docs/agent-guide.md` | Document Qwen Code integration |

### QwenCodeProvider Sketch

```rust
// crates/ckrv-sandbox/src/agent/qwen.rs
#[derive(Debug, Default)]
pub struct QwenCodeProvider;

impl AgentProvider for QwenCodeProvider {
    fn name(&self) -> &str { "Qwen Code" }
    fn agent_type(&self) -> AgentType { AgentType::QwenCode }

    fn build_command(&self, prompt: &str, workdir: &Path, config: &AgentConfig) -> Vec<String> {
        let mut cmd = vec!["qwen".to_string()];
        // Non-interactive flag — verify exact flag against Qwen Code CLI docs
        cmd.push("--no-interactive".to_string());
        // Prompt
        cmd.push("--message".to_string());
        cmd.push(prompt.to_string());
        // Working directory
        cmd.push("--directory".to_string());
        cmd.push(workdir.to_string_lossy().to_string());
        // Model override (e.g. "qwen3-coder-480b-a22b")
        if let Some(ref model) = config.model {
            cmd.push("--model".to_string());
            cmd.push(model.clone());
        }
        cmd
    }

    fn required_env_vars(&self) -> Vec<&str> {
        // Qwen Code authenticates via DashScope API key
        vec!["DASHSCOPE_API_KEY"]
    }

    fn config_mounts(&self, host_home: &str, container_home: &str) -> Vec<Mount> {
        let mut mounts = Vec::new();
        let qwen_dir = format!("{}/.qwen", host_home);
        if std::path::Path::new(&qwen_dir).exists() {
            mounts.push(Mount {
                target: Some(format!("{}/.qwen", container_home)),
                source: Some(qwen_dir),
                typ: Some(bollard::models::MountTypeEnum::BIND),
                read_only: Some(false),
                ..Default::default()
            });
        }
        mounts
    }

    fn parse_output(&self, stdout: &str, stderr: &str, exit_code: i32) -> Result<AgentOutput> {
        Ok(AgentOutput {
            success: exit_code == 0,
            stdout: stdout.to_string(),
            stderr: stderr.to_string(),
            exit_code,
        })
    }
}
```

### YAML Configuration

```yaml
agents:
  - id: qwen-agent
    name: Qwen Code
    agent_type: qwen_code
    enabled: true
    description: Alibaba Qwen3-Coder via DashScope
```

Environment variables required in the host shell:

```bash
export DASHSCOPE_API_KEY=sk-...
```

### Dockerfile.qwen Sketch

```dockerfile
FROM node:22-slim
RUN apt-get update && apt-get install -y git curl ca-certificates \
    && rm -rf /var/lib/apt/lists/*
RUN npm install -g @qwen-code/cli
RUN useradd -m -s /bin/bash -d /home/qwen qwen \
    && mkdir -p /home/qwen/.qwen \
    && chown -R qwen:qwen /home/qwen
RUN mkdir -p /workspace && chown qwen:qwen /workspace
WORKDIR /workspace
ENV HOME=/home/qwen
RUN qwen --version || true
USER qwen
CMD ["/bin/bash"]
```

> **Note**: The `USER qwen` directive is mandatory. Claude Code and Codex both require a non-root user; assume Qwen Code has the same restriction until confirmed otherwise.

## Implementation Notes

### CLI Flag Verification Required

The exact Qwen Code CLI flags for non-interactive execution must be verified against the official repository (`@qwen-code/cli`) before writing `build_command`. The sketch above assumes Claude Code-like flags (`--no-interactive`, `--message`, `--directory`). If Qwen Code was forked from Gemini CLI, flags may differ. See open questions.

### DashScope vs QwenCode Config File

Qwen Code may support both env-var auth (`DASHSCOPE_API_KEY`) and a local config file (`~/.qwen/settings.json`). The provider should:
1. Declare `DASHSCOPE_API_KEY` in `required_env_vars()` (primary path)
2. Mount `~/.qwen/` if it exists (config file path)

This mirrors how Codex handles both `~/.codex/` and `OPENAI_API_KEY`.

### Model Identifiers

Known DashScope model IDs for Qwen Code (verify against DashScope docs):

| Model ID | Notes |
|----------|-------|
| `qwen3-coder-480b-a22b` | Flagship — 480B params |
| `qwen3-coder-480b-a22b-instruct` | Instruct variant |
| `qwen-coder-turbo` | Fast, cost-optimized |
| `qwen-coder-plus` | Balanced |

### Execution Order

1. `qwen.rs` — new file, no deps within ckrv-sandbox
2. `mod.rs` — add module, enum variant, factory arm
3. `tests.rs` — add test functions
4. `agent_lookup.rs` — add CLI enum variant
5. `Dockerfile.qwen` — new file
6. `Dockerfile.agent` — add qwen-code install block
7. `agent-guide.md` — documentation

### Cross-Crate Impact

No changes needed to `ckrv-core`, `ckrv-transport`, or `ckrv-ui` for the CLI path. If UI Agent Manager support is desired (showing Qwen Code as a named card), `ckrv-transport/src/handlers/agents.rs` would need a `QwenCodeModel` type and `/agents/qwen-models` route — scoped to a follow-up issue.

## Open Questions

- [ ] What are the exact non-interactive CLI flags for `@qwen-code/cli`? Candidates: `--no-interactive`, `--print`, `--headless`. Check the [qwen-code GitHub repo](https://github.com/QwenLM/qwen-code).
- [ ] What is the correct npm package name? `@qwen-code/cli` or `@qwen-code/qwen-code`?
- [ ] Does Qwen Code require a non-root Docker user? (Assumed yes, matching Claude/Codex pattern.)
- [ ] Is `DASHSCOPE_API_KEY` the only required env var, or does it also accept `QWEN_API_KEY`?
- [ ] Does Qwen Code support a `--cwd` / `--directory` workdir flag, or does it use `$PWD`?
- [ ] Does Qwen Code emit streaming JSON output (like Kilo's `--format json`) that could feed the `ckrv ui` terminal?
- [ ] Should `ckrv ui` Agent Manager get a Qwen Code configuration card in this issue or a follow-up?

## Success Criteria

| Metric | Target |
|--------|--------|
| Build | `cargo build --workspace` succeeds |
| Unit tests | All new Qwen provider tests pass (`cargo test -p ckrv-sandbox`) |
| YAML deserialization | `agent_type: qwen_code` deserializes correctly |
| Docker image | `docker build -f docker/Dockerfile.qwen .` succeeds |
| End-to-end | `ckrv task run --agent qwen-agent -p "echo hello"` executes in container |
| Documentation | Qwen Code appears in `agent-guide.md` support table |

## Next Steps

- [ ] Verify Qwen Code CLI flags against `@qwen-code/cli` npm package or GitHub repo
- [ ] Confirm npm package name and binary name
- [ ] Implement `QwenCodeProvider` in `crates/ckrv-sandbox/src/agent/qwen.rs`
- [ ] Add `QwenCode` variant to sandbox and CLI `AgentType` enums
- [ ] Create `docker/Dockerfile.qwen`
- [ ] Add tests
- [ ] Update `agent-guide.md`
- [ ] Follow-up: consider `/agents/qwen-models` route in `ckrv-transport` for UI model picker

## References

- [Qwen Code GitHub](https://github.com/QwenLM/qwen-code)
- [DashScope API Docs](https://help.aliyun.com/zh/dashscope/)
- [Kilo Code integration brainstorm](../kilo-code-agent/notes.md) — reference pattern
- [Agent Guide](../../crates/docs/agent-guide.md)
- [Agent Provider trait](../../crates/ckrv-sandbox/src/agent/mod.rs)
- [Kilo provider implementation](../../crates/ckrv-sandbox/src/agent/kilo.rs)
