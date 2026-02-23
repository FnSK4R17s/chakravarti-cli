# Add Mistral Vibe

**Issue**: [#29](https://github.com/FnSK4R17s/chakravarti-cli/issues/29)  
**Created**: 2026-02-23  
**Status**: Ready for Spec (implementation-scoped)

## 1) Problem Statement (Concrete)

Users want a first-class “Mistral Vibe” path. The codebase already supports Mistral-capable execution through **Kilo Code** (`agent_type: kilo_code`), but that path is not packaged as an explicit, low-friction preset with clear contracts, validation, and docs.

Today this causes:
- ambiguous setup (users do not know whether to use Kilo vs OpenRouter)
- avoidable runtime failures (missing `~/.config/kilo` auth mount, invalid model IDs)
- inconsistent UX language across docs/UI

## 2) Repo-Validated Current State

Validated against repository structure and implementation:

### Existing agent execution backends
- `crates/ckrv-sandbox/src/agent/mod.rs`
  - `AgentType::{Claude, Codex, KiloCode}` at sandbox execution layer
- `crates/ckrv-sandbox/src/agent/kilo.rs`
  - Command builder for Kilo: `kilo run <prompt> --auto [--format json] [--model <id>] --cwd <path>`
  - Credential mount source: `~/.config/kilo` -> container `${HOME}/.config/kilo`

### Agent config + UI/API layer
- `crates/ckrv-transport/src/types/agents.rs`
  - API `AgentType` includes `KiloCode` (`snake_case: kilo_code`)
  - `KiloCodeConfig { model: String }`
- `crates/ckrv-transport/src/handlers/agents.rs`
  - Reads/writes `~/.config/chakravarti/agents.yaml`
  - Kilo model discovery via `kilo models` with fallback list
- `crates/ckrv-ui/frontend/src/components/AgentManager.tsx`
  - Agent type includes `kilo_code`
  - Kilo model picker path already exists

### Terminal session behavior
- `crates/ckrv-transport/src/handlers/terminal.rs`
  - For `kilo_code` sessions:
    - image: `ckrv-kilo:latest`
    - `HOME=/home/kilo`
    - bind mount: `${HOST_HOME}/.config/kilo:/home/kilo/.config/kilo` when present
    - TERM vars set (`TERM=xterm-256color`, `COLORTERM=truecolor`)

## 3) Scope Decision

## In-scope for issue #29 (recommended)
Deliver **Mistral Vibe as a Kilo-based preset + docs + validation improvements**.

This means:
1. no new sandbox provider type in `ckrv-sandbox`
2. no new Docker image lineage
3. explicit preset contract and discoverability for Mistral in existing agent flows

## Out of scope (defer)
- Native `mistral_*` provider in sandbox
- New direct Mistral auth protocol implementation in backend

## 4) Command & Config Contracts

## 4.1 Agents YAML contract (authoritative runtime file)
File: `~/.config/chakravarti/agents.yaml`

Mistral Vibe preset (Kilo path):

```yaml
agents:
  - id: mistral-vibe
    name: Mistral Vibe (Kilo)
    agent_type: kilo_code
    level: 4
    is_default: false
    is_qa_agent: false
    is_test_writer: false
    enabled: true
    description: Mistral via Kilo Code (multi-provider)
    kilo:
      model: "kilo/mistralai/mistral-medium-2508"
```

Contract requirements:
- `agent_type` MUST be `kilo_code`
- `kilo.model` MUST be non-empty and SHOULD be a value returned by `kilo models`
- `enabled=true` required for selection in role lookup paths

## 4.2 Execution command contract (sandbox provider)
File: `crates/ckrv-sandbox/src/agent/kilo.rs`

Expected command shape:

```text
kilo run <prompt> --auto [--format json] [--model <provider/model>] --cwd <workdir>
```

Rules:
- `--auto` always present for non-interactive execution
- `--format json` present when `streaming=true`
- `--model` present when model configured in agent
- `--cwd` always present

## 4.3 Terminal contract (interactive session)
File: `crates/ckrv-transport/src/handlers/terminal.rs`

For `agent_type == kilo_code`:
- container image MUST be `ckrv-kilo:latest`
- container HOME MUST be `/home/kilo`
- credentials mount SHOULD be attempted from `${HOME}/.config/kilo`
- terminal env MUST include `TERM=xterm-256color` and `COLORTERM=truecolor`

## 5) Auth & Mounts (explicit)

## Host prerequisites
1. Kilo CLI installed (`kilo --version` succeeds)
2. User authenticated (`kilo auth` completed)
3. Auth artifact exists under `~/.config/kilo/`

## Runtime mount behavior
- Non-interactive sandbox execution (provider mount contract):
  - host: `${HOME}/.config/kilo`
  - container: `${container_home}/.config/kilo`
- Interactive terminal execution:
  - host: `${HOME}/.config/kilo`
  - container: `/home/kilo/.config/kilo`

## Failure if missing
If mount source missing or empty, command may run but Kilo backend provider auth fails at runtime (401/unauthorized/provider auth error depending on model backend).

## 6) File-by-File Implementation Plan

## 6.1 `crates/docs/agent-guide.md`
Add a dedicated section: **Mistral Vibe (via Kilo Code)**
- copy-paste `agents.yaml` snippet
- prerequisite block (`npm i -g @kilocode/cli`, `kilo auth`, `kilo models`)
- verification command(s):
  - `ckrv task run --agent mistral-vibe -p "Create hello.txt"`
- troubleshooting for auth mount and bad model IDs

## 6.2 `crates/docs/getting-started.md`
Add short “Quick start: Mistral Vibe” subsection
- shortest path from fresh repo + installed CLI to successful run
- link to agent guide section for details

## 6.3 `crates/docs/cli-commands.md` (if needed for discoverability)
- Add one note/example showing `--agent mistral-vibe` with task/run flows
- Keep unchanged if this file intentionally avoids provider-specific examples

## 6.4 `brainstorming/issue-029-add-mistral-vibe/tasks.md`
Replace generic tasks with implementation-grade tasks, tests, and exit criteria (done in this branch).

## 6.5 Optional UX follow-up (same issue or follow-up)
`crates/ckrv-ui/frontend/src/components/AgentManager.tsx`
- add a one-click preset option in create modal:
  - label: “Mistral Vibe (Kilo)”
  - initializes `agent_type='kilo_code'` + first Mistral model candidate from model list
- if omitted in this issue, document as explicit follow-up

## 7) Failure Modes and Required Handling

1. **Kilo binary missing**
   - detection: test-agent endpoint (`kilo --version`) fails
   - expected message: “Kilo Code CLI not found …”
2. **Kilo auth missing / bad token**
   - detection: runtime command fails despite CLI presence
   - expected remediation: run `kilo auth`, verify mount path exists
3. **Invalid model ID in `kilo.model`**
   - detection: provider returns model-not-found/unsupported
   - remediation: use `kilo models` and select exact ID
4. **Docker image unavailable (`ckrv-kilo:latest`)**
   - detection: terminal start fails container creation/start
   - remediation: run install target that builds images (`just install`)
5. **HOME bind not present in container session**
   - detection: unauthenticated execution even though host is logged in
   - remediation: verify bind in terminal handler path and host config directory

## 8) Test Matrix (Spec-level)

| Area | Test | Input | Expected |
|---|---|---|---|
| Config load/save | `agents.yaml` roundtrip for `kilo_code` | valid `kilo.model` | persisted + listed via API |
| Agent test endpoint | Kilo CLI presence | `agent_type=kilo_code` | success message with version |
| Agent test endpoint | Missing Kilo CLI | simulate no binary | failure message actionable |
| Model discovery | `kilo models` success | CLI output lines | parsed provider/name/free flags |
| Model discovery fallback | command failure | non-zero exit | curated fallback list returned |
| Terminal session | start for Kilo | `agent_type=kilo_code` | image `ckrv-kilo:latest`, HOME `/home/kilo` |
| Terminal session | missing auth dir | no `~/.config/kilo` | session starts, auth-dependent commands fail clearly |
| End-to-end task run | `ckrv task run --agent mistral-vibe` | simple prompt | successful output + files changed |

## 9) Acceptance Criteria (Issue Closure)

Issue #29 is complete when all are true:
- [ ] Docs provide a clear Mistral Vibe path using `kilo_code`
- [ ] Copy-paste `agents.yaml` preset included and verified
- [ ] Runtime command contract documented (`kilo run ... --model ... --cwd ...`)
- [ ] Auth/mount prerequisites and troubleshooting documented
- [ ] At least one smoke run demonstrated in local validation notes
- [ ] No new native provider type added (unless scope change approved)

## 10) Non-goals / Guardrails

- Do not add `AgentType::Mistral` in sandbox in this issue
- Do not implement direct Mistral API wiring bypassing Kilo
- Keep changes DX/documentation-focused with small validation hardening

## 11) Open Questions (for spec handoff)

- Should UI include explicit preset button in this issue or follow-up?
- Which Mistral model ID should be “recommended default” in docs (stable SKU vs latest)?
- Should `test_agent_handler` for `kilo_code` validate auth presence in addition to binary presence?

## 12) References

- `crates/ckrv-sandbox/src/agent/kilo.rs`
- `crates/ckrv-sandbox/src/agent/mod.rs`
- `crates/ckrv-transport/src/types/agents.rs`
- `crates/ckrv-transport/src/handlers/agents.rs`
- `crates/ckrv-transport/src/handlers/terminal.rs`
- `crates/docs/agent-guide.md`
- `crates/docs/getting-started.md`
- https://github.com/FnSK4R17s/chakravarti-cli/issues/29
- https://mistral.ai/news/mistral-code
