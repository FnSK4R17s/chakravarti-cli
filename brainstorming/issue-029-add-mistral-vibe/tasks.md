# Add Mistral Vibe - Implementation Tasks

**Issue**: [#29](https://github.com/FnSK4R17s/chakravarti-cli/issues/29)
**Brainstorm**: [notes.md](./notes.md)
**Created**: 2026-02-24 — rewritten from repo source of truth per PR #58 owner instruction
**Status**: Ready for Implementation

> [!IMPORTANT]
> All CLI flags, auth mechanisms, and config contracts in this file are anchored to
> https://github.com/mistralai/mistral-vibe (v2.2.1, fetched 2026-02-24).
> The primary programmatic flag is `-p / --prompt`; `--output`, `--max-turns`, and
> `--max-price` are programmatic-mode-only. There is NO `--auto` flag — auto-approve
> is the implicit default when using `-p` mode.

## Task Overview

| Phase | Tasks | Priority |
|-------|-------|----------|
| Phase A: Sandbox Provider | 3 | P0 |
| Phase B: Docker Image | 1 | P0 |
| Phase C: Transport Layer | 2 | P0 |
| Phase D: Documentation | 2 | P0 |
| Phase E: Validation | 1 | P1 |
| Phase F: Optional Follow-ups | 2 | P2 |
| **Total (P0)** | **8** | — |

## Dependencies

```
Phase A (sandbox provider)
  A1 vibe.rs provider
  A2 AgentType::MistralVibe in mod.rs          ─┐
  A3 mod vibe; declaration                       │
                                                 │
Phase B (Dockerfile.vibe) ─────────────────────── ├──► Phase C ──► Phase E
                                                 │
Phase C (transport layer)                        │
  C1 types/agents.rs (depends A2)              ─┤
  C2 handlers/agents.rs + terminal.rs           │
     (depends C1, B)                           ─┘

Phase D (docs) — parallel with A/B/C
  D1 agent-guide.md
  D2 getting-started.md (depends D1)

Phase E: smoke-run — depends all phases complete
```

---

## Phase A — Sandbox Provider

### Task A1: Create `crates/ckrv-sandbox/src/agent/vibe.rs`
**Priority**: P0
**Files**: `crates/ckrv-sandbox/src/agent/vibe.rs`

Implement `MistralVibeProvider`, modelled on `kilo.rs`. Use `-p` (short form) or `--prompt`
(long form) for programmatic mode — both are valid; long form preferred for readability.

**Command construction**:

```rust
let mut cmd = vec![
    "vibe".to_string(),
    "--prompt".to_string(),   // or "-p"; long form for clarity
    prompt.to_string(),
    "--output".to_string(),
    "streaming".to_string(),
    "--workdir".to_string(),
    workdir.to_string(),
];
if let Some(max_turns) = config.max_turns {
    cmd.push("--max-turns".to_string());
    cmd.push(max_turns.to_string());
}
if let Some(max_price) = config.max_price {
    cmd.push("--max-price".to_string());
    cmd.push(max_price.to_string());
}
```

**Key differences from `kilo.rs`**:
- `--output streaming` (not `--format json`)
- No `--model` flag (Devstral is default; custom via `--agent NAME` or `~/.vibe/agents/`)
- No `--auto` flag — auto-approve is implicit in `--prompt` mode
- Auth: inject `MISTRAL_API_KEY` as env var; no credentials dir mount

**Acceptance Criteria**:
- [ ] `MistralVibeProvider` implements the `AgentProvider` trait
- [ ] Command uses `vibe --prompt <prompt> --output streaming --workdir <path>`
- [ ] `max_turns` passed as `--max-turns` when set
- [ ] `max_price` passed as `--max-price` when set
- [ ] `MISTRAL_API_KEY` injected as env var into container execution
- [ ] Unit test for command construction exists
- [ ] `cargo clippy -p ckrv-sandbox` emits no new warnings

### Task A2: Add `AgentType::MistralVibe` to `crates/ckrv-sandbox/src/agent/mod.rs`
**Priority**: P0
**Files**: `crates/ckrv-sandbox/src/agent/mod.rs`

Add `MistralVibe` variant to `AgentType` enum and dispatch to `vibe.rs`.

```rust
pub enum AgentType {
    Claude,
    Codex,
    KiloCode,
    MistralVibe,  // new
}
// ...
AgentType::MistralVibe => Box::new(MistralVibeProvider::new(config)),
```

**Acceptance Criteria**:
- [ ] `AgentType::MistralVibe` variant exists
- [ ] All match arms remain exhaustive (no compiler warnings)
- [ ] `mod vibe;` declaration added

### Task A3: Verify module compiles cleanly
**Priority**: P0

- [ ] `cargo build -p ckrv-sandbox` succeeds with `vibe.rs` present
- [ ] No dead-code warnings for the new module

---

## Phase B — Docker Image

### Task B1: Create `docker/Dockerfile.vibe`
**Priority**: P0
**Files**: `docker/Dockerfile.vibe`, `Justfile` (add `docker-build-vibe` target)

Reference `Dockerfile.kilo` for structure. Requires Python ≥3.12.

**Dockerfile**:
```dockerfile
FROM python:3.12-slim

# Install uv and mistral-vibe
RUN pip install uv && uv tool install mistral-vibe

# Ensure vibe binary is on PATH
ENV PATH="/root/.local/bin:$PATH"

# Non-root user matching container HOME=/home/vibe
RUN useradd -m vibe
USER vibe
WORKDIR /home/vibe

# MISTRAL_API_KEY injected at runtime via --env (never baked in)
```

**Justfile target**:
```just
docker-build-vibe:
    docker build -f docker/Dockerfile.vibe -t ckrv-vibe:latest .
```

**Acceptance Criteria**:
- [ ] `docker build -f docker/Dockerfile.vibe -t ckrv-vibe:latest .` succeeds
- [ ] `docker run --rm -e MISTRAL_API_KEY=test ckrv-vibe:latest vibe --version` prints version
- [ ] `vibe` binary is on PATH inside the container
- [ ] Image build wired into `just install` (or documented separately with a note)

---

## Phase C — Transport Layer

### Task C1: Add `MistralVibeConfig` to `crates/ckrv-transport/src/types/agents.rs`
**Priority**: P0
**Files**: `crates/ckrv-transport/src/types/agents.rs`

```rust
pub enum AgentType {
    Claude,
    Codex,
    KiloCode,
    MistralVibe,  // new — serializes to "mistral_vibe"
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MistralVibeConfig {
    pub max_turns: Option<u32>,
    pub max_price: Option<f64>,  // maps to --max-price DOLLARS
}
```

Add `vibe: Option<MistralVibeConfig>` field to the `Agent` config struct.

**Acceptance Criteria**:
- [ ] `AgentType::MistralVibe` serializes to `"mistral_vibe"` (matches `agents.yaml` key)
- [ ] `MistralVibeConfig` is `Serialize`/`Deserialize` compatible
- [ ] `cargo test -p ckrv-transport` passes

### Task C2: Wire `mistral_vibe` in `handlers/agents.rs` and `handlers/terminal.rs`
**Priority**: P0
**Files**: `crates/ckrv-transport/src/handlers/agents.rs`, `crates/ckrv-transport/src/handlers/terminal.rs`

**`agents.rs`**: Handle `AgentType::MistralVibe` in CRUD. No model discovery needed (single default
model); return empty or minimal model list.

**`terminal.rs`**: Add match arm for `AgentType::MistralVibe`:
- Container image: `ckrv-vibe:latest`
- Container HOME: `/home/vibe`
- Auth: inject `MISTRAL_API_KEY` from host env (no credentials dir mount)
- Env: `TERM=xterm-256color`, `COLORTERM=truecolor`

**Acceptance Criteria**:
- [ ] Terminal session for `mistral_vibe` uses `ckrv-vibe:latest`
- [ ] `MISTRAL_API_KEY` passed through to container as env var
- [ ] No auth directory mount (unlike Kilo Code's `~/.config/kilo` mount)
- [ ] `cargo clippy -p ckrv-transport` emits no new warnings
- [ ] `cargo test -p ckrv-transport` passes

---

## Phase D — Documentation

### Task D1: Add "Mistral Vibe" section to `crates/docs/agent-guide.md`
**Priority**: P0
**Files**: `crates/docs/agent-guide.md`

Insert `## Mistral Vibe Integration` section after "Kilo Code Integration".

**Required content**:
1. What it is: Mistral's first-party CLI coding agent (`vibe` binary, Devstral model)
2. Prerequisites and install:
   ```bash
   # One-line install (Linux/macOS, recommended)
   curl -LsSf https://mistral.ai/vibe/install.sh | bash
   # OR: uv tool install mistral-vibe
   # OR: pip install mistral-vibe  (requires Python >=3.12)

   # API key: https://console.mistral.ai → API Keys
   export MISTRAL_API_KEY="sk-..."

   # Verify
   vibe --version
   vibe -p "Say hello" --max-turns 1
   ```
3. Copy-paste `agents.yaml` snippet (with `agent_type: mistral_vibe`)
4. Verification command: `ckrv task run --agent mistral-vibe -p "..."`
5. Auth note: `MISTRAL_API_KEY` is injected into the container at runtime
6. Troubleshooting table (5 failure modes from notes.md §6)

**Acceptance Criteria**:
- [ ] Section present in `agent-guide.md` after Kilo Code section
- [ ] `agents.yaml` snippet uses `agent_type: mistral_vibe`
- [ ] Install shows curl one-liner as primary method
- [ ] Troubleshooting covers all 5 failure modes
- [ ] No reference to Kilo Code as the Mistral path

### Task D2: Add Mistral Vibe quickstart to `crates/docs/getting-started.md`
**Priority**: P0
**Files**: `crates/docs/getting-started.md`

Add `### Quick start: Mistral Vibe` subsection with 4-step happy path:

```markdown
### Quick start: Mistral Vibe

1. **Install** — `curl -LsSf https://mistral.ai/vibe/install.sh | bash`
   (or `uv tool install mistral-vibe` if you have uv; requires Python ≥3.12)
2. **Get a Mistral API key** — https://console.mistral.ai → API Keys
3. **Add agent config** — paste into `~/.config/chakravarti/agents.yaml`:
   ```yaml
   agents:
     - id: mistral-vibe
       name: Mistral Vibe
       agent_type: mistral_vibe
       enabled: true
       vibe:
         max_turns: 50
   ```
4. **Set API key and verify**:
   ```bash
   export MISTRAL_API_KEY="sk-..."
   ckrv task run --agent mistral-vibe -p "Say hello"
   ```

See [Agent Guide](agent-guide.md#mistral-vibe-integration) for full details and troubleshooting.
```

**Acceptance Criteria**:
- [ ] Quickstart section present in `getting-started.md`
- [ ] 4-step path: install → API key → config → verify
- [ ] Curl one-liner shown as primary install method
- [ ] No contradiction with D1 content
- [ ] Link to `agent-guide.md#mistral-vibe-integration` works

---

## Phase E — Validation

### Task E1: Smoke-run Mistral Vibe locally and record evidence
**Priority**: P1
**Files**: This file (record output below)

```bash
# Confirm vibe binary works
vibe --version
export MISTRAL_API_KEY="sk-..."

# Confirm Docker image is available
docker images | grep ckrv-vibe

# Run minimal smoke task
ckrv task run --agent mistral-vibe -p "Create a file called hello.txt with the text 'Mistral Vibe works'"

# Confirm file created in worktree
```

**Acceptance Criteria**:
- [ ] Successful run output captured below (or blocker documented with specific error + root cause)
- [ ] `hello.txt` confirmed created in worktree
- [ ] No auth, model-ID, or Docker image errors

**Evidence** *(fill in after run)*:
```
# paste ckrv output here
```

---

## Phase F — Optional Follow-ups (P2, separate issues)

### Task F1: Open follow-up issue for Mistral Vibe UI preset in AgentManager
**Priority**: P2

Add a "Mistral Vibe" quick-add preset in `crates/ckrv-ui/frontend/src/components/AgentManager.tsx`.
- [ ] Open GitHub issue: "Add Mistral Vibe quick-add preset to AgentManager UI"
- [ ] Reference #29 as parent

### Task F2: Audit docs for Kilo Code + Devstral workaround mentions
**Priority**: P2

Ensure no existing docs suggest "use Kilo Code with `mistralai/devstral-latest`" as a Mistral path.
- [ ] Audit `crates/docs/` for Devstral under Kilo Code mentions
- [ ] Remove or redirect to Mistral Vibe section

---

## Acceptance Criteria (Issue Closure)

- [ ] `crates/ckrv-sandbox/src/agent/vibe.rs` exists; invokes `vibe --prompt ... --output streaming --workdir ...`
- [ ] `AgentType::MistralVibe` in both `ckrv-sandbox` and `ckrv-transport`
- [ ] `docker/Dockerfile.vibe` builds a working `ckrv-vibe:latest` image
- [ ] `agent-guide.md` has Mistral Vibe section with `agent_type: mistral_vibe` config
- [ ] `getting-started.md` has 4-step Mistral Vibe quickstart
- [ ] `MISTRAL_API_KEY` env var passthrough documented and verified in container
- [ ] Smoke-run evidence recorded in E1
- [ ] `cargo test --workspace` passes
- [ ] Kilo Code path unaffected
