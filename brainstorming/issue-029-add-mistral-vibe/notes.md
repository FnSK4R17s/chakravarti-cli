# Add Mistral Vibe

**Issue**: [#29](https://github.com/FnSK4R17s/chakravarti-cli/issues/29)
**Created**: 2026-02-23
**Status**: Tasks Generated

## 1) Problem Statement

Users want a first-class "Mistral Vibe" path in ckrv — i.e., running Mistral's agentic coding model (Devstral) as an orchestrated agent. The issue links to the Mistral Code announcement, which introduced **Devstral** as Mistral's purpose-built agentic model for multi-step coding tasks.

Today there is no explicit Mistral path. Users:
- Do not know whether to use Kilo Code, OpenRouter, or wait for a native Mistral CLI
- Cannot discover which Mistral model is best for agentic coding tasks (answer: Devstral)
- Get no docs, no copy-paste config, and no troubleshooting guidance for a Mistral-backed agent

## 2) What Is Mistral Code? (Validated 2026-02-23)

Mistral Code is an enterprise AI coding assistant announced by Mistral AI. Key facts:

| Property | Details |
|----------|---------|
| Product type | Enterprise IDE assistant |
| IDE support | VS Code extension, JetBrains plugin (private beta) |
| CLI | **None** — no Mistral Code CLI exists at launch |
| Deployment | Cloud SaaS, reserved capacity, air-gapped on-prem |
| Agentic model | **Devstral** — purpose-built for multi-step agentic coding tasks |
| Completion model | Codestral — fill-in-the-middle, autocomplete |
| Search model | Codestral Embed — semantic code search |
| Chat model | Mistral Medium — conversational assistance |
| Languages | 80+ programming languages |
| Target market | Enterprise (compliance, on-prem, fine-tuning) |

**Key insight**: Because there is no Mistral Code CLI, ckrv cannot invoke it like Claude Code or Codex. The integration path must go through an intermediary that already has Mistral support.

## 3) Integration Path Analysis

Three options for routing ckrv tasks to Devstral:

| Option | Mechanism | Pros | Cons |
|--------|-----------|------|------|
| **A: Kilo Code (recommended)** | `kilo_code` provider with `devstral` model | Already wired, file-based auth, Docker image exists | Requires Kilo CLI install + `kilo auth` |
| **B: OpenRouter** | `claude_openrouter` provider with Mistral model | Simple API key, no extra CLI | Routes through Claude Code CLI, not truly "Mistral native" feel |
| **C: Native Mistral provider** | New `ckrv-sandbox` provider calling Mistral API | Cleanest contract | Requires new sandbox provider, Docker image, Mistral API key auth — over-engineered for #29 |

### Decision: Option A — Kilo Code with Devstral

Rationale aligning with vision:
- `ckrv` is the orchestration layer; agents are interchangeable workers
- Kilo Code already supports Mistral providers including Devstral
- No new sandbox code needed — purely docs + config contract
- Consistent with how GLM Coding Plan (Z.AI) was integrated: via Claude Code env vars, not a new binary

## 4) Repo-Validated Current State

Validated against codebase as of 2026-02-23:

### Sandbox execution layer
- `crates/ckrv-sandbox/src/agent/mod.rs` — `AgentType::{Claude, Codex, KiloCode}`
- `crates/ckrv-sandbox/src/agent/kilo.rs` — command: `kilo run <prompt> --auto [--format json] [--model <id>] --cwd <path>`
- Credential mount: `~/.config/kilo` → container `${HOME}/.config/kilo`

### Transport / API layer
- `crates/ckrv-transport/src/types/agents.rs` — `AgentType::KiloCode` with `KiloCodeConfig { model: String }`
- `crates/ckrv-transport/src/handlers/agents.rs` — `agents.yaml` read/write; `kilo models` discovery with fallback list
- `crates/ckrv-transport/src/handlers/terminal.rs` — `ckrv-kilo:latest` image, `HOME=/home/kilo`, auth mount

### UI
- `crates/ckrv-ui/frontend/src/components/AgentManager.tsx` — `kilo_code` agent type exists, model picker present

### Docs (gaps)
- `crates/docs/agent-guide.md` — Kilo Code section present, but **no Mistral/Devstral model example**
- `crates/docs/getting-started.md` — no Mistral quickstart
- `crates/docs/cli-commands.md` — no Mistral example

## 5) Command & Config Contracts

### agents.yaml — Mistral Vibe preset

```yaml
agents:
  - id: mistral-vibe
    name: Mistral Vibe (Devstral)
    agent_type: kilo_code
    level: 4
    is_default: false
    is_qa_agent: false
    is_test_writer: false
    enabled: true
    description: Devstral via Kilo Code — Mistral's agentic coding model
    kilo:
      model: "mistralai/devstral-latest"
```

> **Model note**: Use `kilo models` to list exact available Mistral model IDs. `mistralai/devstral-latest` is the recommended agentic model; `mistralai/codestral-latest` is available for completions-style tasks.

### Sandbox execution command

```text
kilo run <prompt> --auto [--format json] [--model mistralai/devstral-latest] --cwd <workdir>
```

Rules:
- `--auto` always present (non-interactive)
- `--format json` present when streaming=true
- `--model` always present when `kilo.model` configured
- `--cwd` always present

### Terminal session contract

For `agent_type == kilo_code`:
- Container image: `ckrv-kilo:latest`
- Container HOME: `/home/kilo`
- Auth mount: `${HOST_HOME}/.config/kilo` → `/home/kilo/.config/kilo`
- Env: `TERM=xterm-256color`, `COLORTERM=truecolor`

## 6) Auth & Prerequisites

```bash
# Install Kilo Code CLI
npm install -g @kilocode/cli

# Authenticate (connects Mistral or other providers)
kilo auth

# List available Mistral models
kilo models | grep mistral
```

Auth artifacts stored at: `~/.config/kilo/config.json`

### Failure modes

| Failure | Detection | Remediation |
|---------|-----------|-------------|
| `kilo` binary missing | `kilo --version` fails | `npm install -g @kilocode/cli` |
| Kilo not authenticated | 401 at runtime despite binary present | Run `kilo auth` again |
| Invalid model ID | Provider returns model-not-found | Run `kilo models`, pick exact ID |
| Docker image missing | `ckrv-kilo:latest` not found | `just install` (rebuilds images) |
| Auth dir missing in container | Authenticated locally, fails in container | Verify `~/.config/kilo/` exists on host |

## 7) Scope for Issue #29

### In scope
1. Documentation: Mistral Vibe section in `agent-guide.md` with Devstral model focus
2. Documentation: Quick start path in `getting-started.md`
3. Copy-paste `agents.yaml` snippet validated against runtime schema
4. Auth/mount troubleshooting coverage
5. Verification command example

### Out of scope (explicit)
- New `AgentType::Mistral` or `AgentType::Devstral` in `ckrv-sandbox` — unnecessary complexity
- Direct Mistral API wiring bypassing Kilo — no native CLI to invoke
- UI preset button in AgentManager.tsx — nice-to-have, separate follow-up issue
- Mistral Code IDE extension integration — not a CLI tool, wrong layer for ckrv

## 8) Future Consideration: Native Mistral CLI

If Mistral AI ships a `mistral` CLI tool analogous to `claude` or `codex`, the path would be:
1. New `crates/ckrv-sandbox/src/agent/mistral.rs` provider
2. New `AgentType::Mistral` variant
3. New `docker/Dockerfile.mistral`
4. New `mistral_code` agent_type in transport layer

This should be tracked as a separate issue when/if Mistral CLI becomes available.

## 9) Success Criteria

Issue #29 is complete when:
- [ ] `agent-guide.md` has a "Mistral Vibe (Devstral)" section with copy-paste config
- [ ] Config snippet uses `kilo_code` agent_type + `mistralai/devstral-latest` model
- [ ] Auth prerequisites (`kilo auth`) and troubleshooting documented
- [ ] Quick start path added to `getting-started.md`
- [ ] At least one smoke-run demonstrated locally (output captured in task notes)
- [ ] No new sandbox provider introduced

## 10) Open Questions

- [ ] Should `kilo models` fallback list in `agents.rs` include Devstral explicitly?
- [ ] Which Mistral model ID is stable long-term: `devstral-latest` vs specific version pin?
- [ ] Should the UI's AgentManager show "Mistral Vibe" as a quick-add preset? (follow-up?)
- [ ] Will Mistral ship a native CLI alongside general availability of Mistral Code?

## 11) References

- https://github.com/FnSK4R17s/chakravarti-cli/issues/29
- https://mistral.ai/news/mistral-code
- `crates/ckrv-sandbox/src/agent/kilo.rs`
- `crates/ckrv-sandbox/src/agent/mod.rs`
- `crates/ckrv-transport/src/types/agents.rs`
- `crates/ckrv-transport/src/handlers/agents.rs`
- `crates/ckrv-transport/src/handlers/terminal.rs`
- `crates/docs/agent-guide.md`
- `crates/docs/getting-started.md`
