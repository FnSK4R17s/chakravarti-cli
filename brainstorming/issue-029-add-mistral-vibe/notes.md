# Add Mistral Vibe

**Issue**: [#29](https://github.com/FnSK4R17s/chakravarti-cli/issues/29)
**Created**: 2026-02-24 — rewritten from repo source of truth per PR #58 owner instruction
**Status**: Tasks ready for implementation

> [!IMPORTANT]
> **Authoritative source**: https://github.com/mistralai/mistral-vibe (v2.2.1, fetched 2026-02-24)
> Previous brainstorm versions contained inaccuracies. This version is grounded entirely in the
> live repo: CLI flags, auth, config, agents, and install methods verified from source.

## 1) Problem Statement

Users want a first-class "Mistral Vibe" path in ckrv — running Mistral's own agentic coding CLI
(`vibe` binary) as an orchestrated agent, the same way Claude/Codex/Kilo Code are supported today.

Today there is no `vibe` integration in ckrv. Users:
- Have no `mistral_vibe` agent type available in `agents.yaml`
- Get no `ckrv-vibe:latest` Docker image for sandboxed runs
- Find no docs, config snippets, or troubleshooting for a Mistral-backed agent

## 2) What Is mistral-vibe? (Anchored to repo, 2026-02-24)

**mistral-vibe** is Mistral AI's own open-source CLI coding agent — the direct Mistral analogue to
`claude` (Anthropic) and `codex` (OpenAI).

| Property | Details |
|----------|---------|
| Repo | https://github.com/mistralai/mistral-vibe |
| Stars | ~3.2k (Feb 2026) |
| Binary | `vibe` (entry: `vibe.cli.entrypoint:main`); also `vibe-acp` (ACP endpoint) |
| Package | `mistral-vibe` on PyPI |
| Version | 2.2.1 (released 2026-02-18) |
| Language | Python ≥3.12 |
| License | Apache 2.0 |
| Default model | Devstral (Mistral's agentic coding model) |
| Auth | `MISTRAL_API_KEY` env var, `~/.vibe/.env`, or interactive `--setup` |
| Config | `./.vibe/config.toml` (project) → `~/.vibe/config.toml` (global); override: `VIBE_HOME` |
| Install (one-line) | `curl -LsSf https://mistral.ai/vibe/install.sh \| bash` |
| Install (uv) | `uv tool install mistral-vibe` |
| Install (pip) | `pip install mistral-vibe` |

### Complete CLI flags (from `vibe/cli/entrypoint.py`)

| Flag | Notes |
|------|-------|
| `-p, --prompt TEXT` | **Programmatic mode**: triggers non-interactive, auto-approve, then exit |
| `--max-turns N` | Hard turn limit (programmatic mode only) |
| `--max-price DOLLARS` | Cost ceiling; interrupts if exceeded (programmatic only) |
| `--output {text,json,streaming}` | Output format (programmatic only; default `text`) |
| `--enabled-tools TOOL` | Restrict tools (repeatable; disables all others in programmatic mode) |
| `--agent NAME` | Builtin name or `~/.vibe/agents/NAME.toml` |
| `--workdir DIR` | Change directory before running |
| `-c, --continue` | Resume most recent saved session |
| `--resume SESSION_ID` | Resume specific session (partial match) |
| `--setup` | Configure API key interactively and exit |

### Programmatic mode for ckrv (critical)

```bash
vibe -p "<task>" --output streaming --workdir <path> [--max-turns N] [--max-price X]
```

- `-p` (or `--prompt`) triggers non-interactive mode; **auto-approve is the implicit default**
- `--output streaming` → newline-delimited JSON per message (best for ckrv's stream parser)
- `--workdir` sets working directory (avoids `cd` in container)
- `--max-turns` / `--max-price` are optional safety valves

### Built-in agents (`--agent NAME`)

| Agent | Behavior |
|-------|----------|
| `default` | Requires approval per tool execution |
| `plan` | Read-only; auto-approves safe (read) tools |
| `accept-edits` | Auto-approves file edit tools only |
| `auto-approve` | Auto-approves all — also the implicit behavior of `-p` mode |

Custom agents: `~/.vibe/agents/<name>.toml` → invoke via `--agent <name>`.

### Key differences from Kilo Code

| | Kilo Code | mistral-vibe |
|---|---|---|
| Format flag | `--format json` | `--output streaming` |
| Model flag | `--model <id>` | No model flag (Devstral default) |
| Auth | `kilo auth login` / credentials dir | `MISTRAL_API_KEY` env var |
| Auto-approve | `--auto` flag | Implicit in `-p` mode |
| Cost control | — | `--max-price DOLLARS` |

## 3) Integration Path Decision

**Native mistral-vibe provider** (Option C in original analysis) — correct and only option:

- Follows identical pattern to `kilo.rs` / `claude.rs` / `codex.rs` in `ckrv-sandbox`
- `vibe -p` mode is non-interactive and streaming-output-capable → ckrv-compatible
- No alternative routing (Kilo Code + Devstral would use Kilo's toolset, not `vibe`)

## 4) Current State (Repo-Validated)

**Missing from ckrv today:**

| Location | Gap |
|----------|-----|
| `crates/ckrv-sandbox/src/agent/` | No `vibe.rs`; no `AgentType::MistralVibe` in `mod.rs` |
| `crates/ckrv-transport/src/types/agents.rs` | No `MistralVibeConfig`; no `AgentType::MistralVibe` |
| `crates/ckrv-transport/src/handlers/agents.rs` | No MistralVibe CRUD |
| `crates/ckrv-transport/src/handlers/terminal.rs` | No `ckrv-vibe:latest` image routing |
| `docker/` | No `Dockerfile.vibe` |
| `crates/docs/agent-guide.md` | No Mistral Vibe section |
| `crates/docs/getting-started.md` | No Mistral Vibe quickstart |

**Reference implementations:** `kilo.rs`, `Dockerfile.kilo`

## 5) Command & Config Contracts

### Execution command (in container)

```text
vibe -p <prompt> --output streaming --workdir <workdir> [--max-turns N] [--max-price X]
```

### agents.yaml snippet

```yaml
agents:
  - id: mistral-vibe
    name: Mistral Vibe
    agent_type: mistral_vibe
    level: 4
    is_default: false
    is_qa_agent: false
    is_test_writer: false
    enabled: true
    description: Mistral's first-party CLI coding agent (Devstral model)
    vibe:
      max_turns: 50
```

> No `model` field needed — Devstral is the default. Custom model via `~/.vibe/agents/` TOML.

### Container session contract

| Property | Value |
|----------|-------|
| Image | `ckrv-vibe:latest` |
| HOME | `/home/vibe` |
| Auth | `MISTRAL_API_KEY` env var injected at runtime |
| Env | `TERM=xterm-256color`, `COLORTERM=truecolor` |
| Mount | None (no credentials dir — env var only) |

## 6) Auth & Prerequisites

```bash
# Install (one-line, recommended)
curl -LsSf https://mistral.ai/vibe/install.sh | bash
# OR via uv
uv tool install mistral-vibe
# OR pip (requires Python >=3.12)
pip install mistral-vibe

# Get API key: https://console.mistral.ai → API Keys
export MISTRAL_API_KEY="sk-..."

# Verify
vibe --version
vibe -p "Say hello" --max-turns 1
```

Auth priority: `MISTRAL_API_KEY` env var > `~/.vibe/.env` > interactive `--setup`.

### Failure modes

| Failure | Detection | Remediation |
|---------|-----------|-------------|
| `vibe` binary missing | `vibe --version` fails | `uv tool install mistral-vibe` |
| API key missing | 401 error at runtime | Set `MISTRAL_API_KEY` env var |
| Python < 3.12 | Install fails | Use `uv` (manages Python version) |
| Docker image missing | `ckrv-vibe:latest` not found | `just install` (rebuilds images) |
| API key not in container | Auth error inside container | Verify `MISTRAL_API_KEY` in ckrv env config |

## 7) Scope for Issue #29

### In scope

1. `crates/ckrv-sandbox/src/agent/vibe.rs` — `MistralVibeProvider` (modelled on `kilo.rs`)
2. `AgentType::MistralVibe` in `ckrv-sandbox/src/agent/mod.rs`
3. `MistralVibeConfig` + `AgentType::MistralVibe` in `ckrv-transport/src/types/agents.rs`
4. `AgentType::MistralVibe` CRUD in `ckrv-transport/src/handlers/agents.rs`
5. `ckrv-vibe:latest` image routing in `ckrv-transport/src/handlers/terminal.rs`
6. `docker/Dockerfile.vibe` (Python 3.12, uv, mistral-vibe)
7. Mistral Vibe section in `crates/docs/agent-guide.md`
8. Mistral Vibe quickstart in `crates/docs/getting-started.md`

### Out of scope

- Kilo Code + Devstral routing (wrong tool)
- `MISTRAL_API_KEY` storage/management in ckrv (env var passthrough only)
- UI preset in `AgentManager.tsx` (separate follow-up issue)
- Custom `~/.vibe/agents/` config management (outside ckrv)
- `vibe-acp` (Agent Client Protocol) integration (future)

## 8) Success Criteria

- [ ] `vibe.rs` exists; invokes `vibe -p ... --output streaming --workdir ...`
- [ ] `AgentType::MistralVibe` in both `ckrv-sandbox` and `ckrv-transport`
- [ ] `docker/Dockerfile.vibe` builds `ckrv-vibe:latest` successfully
- [ ] `agent-guide.md` has Mistral Vibe section with `agent_type: mistral_vibe` config
- [ ] `getting-started.md` has 4-step Mistral Vibe quickstart
- [ ] `MISTRAL_API_KEY` env var passthrough documented and verified in container
- [ ] Smoke-run: `ckrv task run --agent mistral-vibe -p "..."` completes
- [ ] `cargo test --workspace` passes
- [ ] Kilo Code path unaffected

## 9) Open Questions

- [ ] Does `--output streaming` JSON schema match ckrv's existing stream parser, or is a new parser needed?
- [ ] Should `--max-price` be exposed as an optional `vibe.max_price` field in `agents.yaml`?
- [ ] Does `vibe` need `--no-update-check` or similar flag for container use? (check `vibe --help`)
- [ ] Is `uv tool install mistral-vibe` the right Dockerfile install method, or `pip install`?
- [ ] Should `MISTRAL_API_KEY` be an explicit named field in ckrv's env config or host env passthrough only?

## 10) References

- https://github.com/mistralai/mistral-vibe (authoritative source, v2.2.1)
- https://github.com/FnSK4R17s/chakravarti-cli/issues/29
- `crates/ckrv-sandbox/src/agent/kilo.rs` (reference provider implementation)
- `crates/ckrv-sandbox/src/agent/mod.rs`
- `crates/ckrv-transport/src/types/agents.rs`
- `crates/ckrv-transport/src/handlers/agents.rs`
- `crates/ckrv-transport/src/handlers/terminal.rs`
- `docker/Dockerfile.kilo` (reference Dockerfile)
- `crates/docs/agent-guide.md`
- `crates/docs/getting-started.md`
