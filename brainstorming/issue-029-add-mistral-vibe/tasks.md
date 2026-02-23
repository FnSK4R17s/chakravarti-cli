# Add Mistral Vibe - Implementation Tasks

**Issue**: [#29](https://github.com/FnSK4R17s/chakravarti-cli/issues/29)
**Brainstorm**: [notes.md](./notes.md)
**Created**: 2026-02-23
**Status**: Ready for Implementation

## Task Overview

| Phase | Tasks | Estimate |
|-------|-------|----------|
| Phase A: Scope Lock | 1 | 15m |
| Phase B: Documentation | 2 | 1.5h |
| Phase C: Code — Devstral Fallback | 1 | 20m |
| Phase D: Validation | 1 | 30m |
| Phase E: Optional Follow-ups | 2 | — |
| **Total (P0+P1)** | **5** | **~2.5h** |

## Dependencies

```
Phase A ──────────────────────────────────────────────────────►
  A1 (scope lock)
    │
    ├──► Phase B (docs) ────────────────────────────────────────►
    │      B1 (agent-guide.md)
    │      B2 (getting-started.md) — depends on B1 for link target
    │
    └──► Phase C (code) ─────────────────────────────────────────►
           C1 (fallback model list) — independent of B tasks
    │
Phase D ──────────────────────────────────────────────────────►
  D1 (smoke-run) — depends on B1 + B2 being written
```

---

## Phase A — Scope Lock

### Task A1: Confirm integration path and out-of-scope boundaries
**Priority**: P0
**Estimate**: 15m
**Files**: `brainstorming/issue-029-add-mistral-vibe/notes.md`

Verify that the chosen integration path (Kilo Code + `mistralai/devstral-latest`) is correct before writing any docs or code. No new sandbox provider is needed.

**Acceptance Criteria**:
- [ ] `agent_type: kilo_code` + `kilo.model: mistralai/devstral-latest` confirmed as canonical path
- [ ] No new `AgentType::Mistral` or `AgentType::Devstral` will be added to `ckrv-sandbox`
- [ ] Noted: revisit only if Mistral ships a native CLI at GA of Mistral Code

---

## Phase B — Documentation

### Task B1: Add "Mistral Vibe (Devstral)" section to agent-guide.md
**Priority**: P0
**Estimate**: 60m
**Files**: `crates/docs/agent-guide.md`

Insert a new `## Mistral Vibe (Devstral) Integration` section after the existing `## Kilo Code Integration` section. The Kilo Code section ends at line ~403 in the current file.

**Required content**:

1. **Why Devstral**: brief note that Devstral is Mistral's purpose-built agentic coding model (not Codestral which is for completions)
2. **Prerequisites block**:
   ```bash
   npm install -g @kilocode/cli
   kilo auth
   kilo models | grep mistral  # verify model availability
   ```
3. **Copy-paste `agents.yaml` snippet**:
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
   > Note: `mistralai/devstral-latest` is the recommended agentic model. `mistralai/codestral-latest` is available for completion-style tasks.
4. **Verification command**:
   ```bash
   ckrv task run --agent mistral-vibe -p "Create hello.txt with text 'Mistral Vibe works'"
   ```
5. **Troubleshooting table** (mirror the failure modes from notes.md §6):

   | Failure | Detection | Remediation |
   |---------|-----------|-------------|
   | `kilo` binary missing | `kilo --version` fails | `npm install -g @kilocode/cli` |
   | Kilo not authenticated | 401 at runtime | Run `kilo auth` again |
   | Invalid model ID | Provider returns model-not-found | Run `kilo models \| grep mistral`, pick exact ID |
   | Docker image missing | `ckrv-kilo:latest` not found | `just install` (rebuilds images) |
   | Auth dir missing in container | Authenticated locally, fails in container | Verify `~/.config/kilo/` exists on host |

**Acceptance Criteria**:
- [ ] Section appears in `agent-guide.md` after "Kilo Code Integration"
- [ ] `agents.yaml` snippet matches the `KiloCodeConfig` schema in `crates/ckrv-transport/src/types/agents.rs`
- [ ] Troubleshooting table covers all 5 failure modes from notes.md §6
- [ ] Model note distinguishes Devstral (agentic) from Codestral (completion)

---

### Task B2: Add Mistral Vibe quickstart to getting-started.md
**Priority**: P0
**Estimate**: 30m
**Files**: `crates/docs/getting-started.md`

Add a `### Quick start: Mistral Vibe` subsection under an agent configuration section (create one if absent). This is the minimal "happy path" — install → auth → config → run.

**Required content**:
```markdown
### Quick start: Mistral Vibe (Devstral)

1. **Install Kilo Code CLI** — `npm install -g @kilocode/cli`
2. **Authenticate** — `kilo auth` (connects your Mistral account)
3. **Add agent config** — paste into `~/.config/chakravarti/agents.yaml`:
   ```yaml
   agents:
     - id: mistral-vibe
       name: Mistral Vibe (Devstral)
       agent_type: kilo_code
       enabled: true
       kilo:
         model: "mistralai/devstral-latest"
   ```
4. **Verify** — `ckrv task run --agent mistral-vibe -p "Say hello"`

See the [Agent Guide](agent-guide.md#mistral-vibe-devstral-integration) for full details and troubleshooting.
```

**Acceptance Criteria**:
- [ ] Quickstart section present in `getting-started.md`
- [ ] No contradiction with B1 content
- [ ] Link to `agent-guide.md` anchor points to the section created in B1
- [ ] Minimum 4-step path: install → auth → config → verify

---

## Phase C — Code Change

### Task C1: Add Devstral to the Kilo Code fallback model list
**Priority**: P1
**Estimate**: 20m
**Files**: `crates/ckrv-transport/src/handlers/agents.rs`

The `get_fallback_kilo_models()` function (around line 563) currently lists only DeepSeek, Google Gemma, and Qwen models. Since Mistral/Devstral is now a documented first-class path, it must appear in the fallback list so the UI model picker shows it even when `kilo models` fails.

**Change**: append two entries to the `vec![]` in `get_fallback_kilo_models()`:

```rust
KiloCodeModel {
    id: "kilo/mistralai/devstral-latest".to_string(),
    provider: "mistralai".to_string(),
    name: "devstral-latest".to_string(),
    free: false,
},
KiloCodeModel {
    id: "kilo/mistralai/codestral-latest".to_string(),
    provider: "mistralai".to_string(),
    name: "codestral-latest".to_string(),
    free: false,
},
```

**Acceptance Criteria**:
- [ ] `get_fallback_kilo_models()` includes `mistralai/devstral-latest` and `mistralai/codestral-latest`
- [ ] `cargo test -p ckrv-transport` passes (update count assertion in `test_get_fallback_kilo_models` if present)
- [ ] `cargo clippy -p ckrv-transport` emits no new warnings

---

## Phase D — Validation

### Task D1: Smoke-run Mistral Vibe locally and record evidence
**Priority**: P1
**Estimate**: 30m
**Files**: `brainstorming/issue-029-add-mistral-vibe/tasks.md` (record output here)

Configure `mistral-vibe` in local `~/.config/chakravarti/agents.yaml` and run a real task through it.

**Steps**:
```bash
# Confirm Docker image is available
docker images | grep ckrv-kilo

# Run a minimal smoke task
ckrv task run --agent mistral-vibe -p "Create a file called hello.txt with the text 'Mistral Vibe works'"

# Confirm file was created in the worktree
```

**Acceptance Criteria**:
- [ ] Successful run output captured below (or blocker documented with specific error + root cause)
- [ ] File `hello.txt` confirmed created in worktree
- [ ] No auth, model-ID, or Docker image errors

**Evidence** *(fill in after run)*:
```
# paste ckrv output here
```

---

## Phase E — Optional Follow-ups (P2, separate issues)

### Task E1: Open follow-up issue for Mistral Vibe UI preset in AgentManager
**Priority**: P2
**Estimate**: 10m
**Files**: GitHub Issues

Add a "Mistral Vibe" quick-add preset button in `crates/ckrv-ui/frontend/src/components/AgentManager.tsx`. Out of scope for #29.

- [ ] Open GitHub issue: "Add Mistral Vibe quick-add preset to AgentManager UI"
- [ ] Reference #29 as parent

---

### Task E2: Watch for native Mistral CLI and open follow-up
**Priority**: P2
**Estimate**: Ongoing

If Mistral AI ships a `mistral` CLI tool:
- [ ] Open a new issue for native `mistral_code` sandbox provider
- [ ] New files would be: `crates/ckrv-sandbox/src/agent/mistral.rs`, new `AgentType::Mistral` variant, `docker/Dockerfile.mistral`

---

## Acceptance Criteria (Issue Closure)

- [ ] `agent-guide.md` has a "Mistral Vibe (Devstral)" section with copy-paste `agents.yaml` config
- [ ] Config snippet validated against `KiloCodeConfig` schema in `ckrv-transport`
- [ ] Auth prerequisites and troubleshooting table documented
- [ ] `getting-started.md` has a Mistral Vibe quickstart path
- [ ] `get_fallback_kilo_models()` includes `mistralai/devstral-latest`
- [ ] `cargo test -p ckrv-transport` passes
- [ ] Smoke-run evidence recorded in D1
- [ ] No new sandbox provider added (`AgentType` enum unchanged)
