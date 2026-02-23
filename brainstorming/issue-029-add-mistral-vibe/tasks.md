# Add Mistral Vibe - Implementation Tasks

**Issue**: [#29](https://github.com/FnSK4R17s/chakravarti-cli/issues/29)  
**Brainstorm**: [notes.md](./notes.md)  
**Created**: 2026-02-23  
**Status**: Execution-ready task plan

## 0) Delivery Summary

Goal: ship a technically explicit **Mistral Vibe via Kilo Code** path without introducing a new native provider.

Primary outcomes:
1. concrete docs + command/config contracts
2. auth/mount troubleshooting coverage
3. validation checklist + smoke evidence

---

## 1) Work Breakdown Structure

## Phase A — Contract Finalization (P0)

### A1. Freeze implementation scope (Kilo-based path)
**Owner**: maintainer/implementer  
**Estimate**: 20m

**Actions**
- Record decision in issue/spec handoff: no new `mistral_*` provider for #29.
- Reference architecture preference: orchestration-first, reuse existing backends.

**Done when**
- [ ] Scope statement captured in issue thread or spec draft
- [ ] Explicitly notes: `agent_type: kilo_code` is canonical path

---

### A2. Define command + config contracts
**Owner**: implementer  
**Estimate**: 40m

**Actions**
- Document final command contract from `crates/ckrv-sandbox/src/agent/kilo.rs`:
  - `kilo run <prompt> --auto [--format json] [--model <id>] --cwd <path>`
- Document YAML schema expectations from `crates/ckrv-transport/src/types/agents.rs`.

**Done when**
- [ ] Contract section present in docs/brainstorm
- [ ] Includes required/optional flags and failure semantics

---

## Phase B — Documentation & Discoverability (P0/P1)

### B1. Agent guide: add “Mistral Vibe (Kilo)” section
**Target file**: `crates/docs/agent-guide.md`  
**Priority**: P0  
**Estimate**: 60m

**Required content**
- Prereqs (`@kilocode/cli`, `kilo auth`, `kilo models`)
- Copy-paste agent config snippet:
  - `agent_type: kilo_code`
  - `kilo.model: kilo/mistralai/...`
- Verification command:
  - `ckrv task run --agent mistral-vibe -p "Create hello.txt"`
- Troubleshooting subsection:
  - missing binary
  - missing `~/.config/kilo`
  - invalid model id

**Done when**
- [ ] Section merged with correct file references
- [ ] Snippet matches runtime schema

---

### B2. Getting started: add quick Mistral onboarding path
**Target file**: `crates/docs/getting-started.md`  
**Priority**: P1  
**Estimate**: 30m

**Required content**
- 5-minute setup flow pointing to agent guide for details
- One minimal successful command path

**Done when**
- [ ] New subsection exists
- [ ] Flow works from clean machine assumptions

---

### B3. CLI discoverability touchpoint
**Target file**: `crates/docs/cli-commands.md` (if provider examples are accepted there)  
**Priority**: P1  
**Estimate**: 20m

**Actions**
- Add one `--agent mistral-vibe` example for task/run execution OR
- If not added, explicitly document reason in PR notes (scope/consistency policy)

**Done when**
- [ ] Either example added or rationale recorded

---

## Phase C — Validation Hardening (P1)

### C1. Improve test-agent diagnostics for Kilo auth (optional but recommended)
**Target file**: `crates/ckrv-transport/src/handlers/agents.rs`  
**Priority**: P1  
**Estimate**: 45m

**Current behavior**
- `kilo_code` test checks only `kilo --version`.

**Desired behavior (if implemented)**
- Keep version check.
- Add soft warning/hint when `${HOME}/.config/kilo` missing.
- Preserve non-breaking API shape (`TestAgentResponse`).

**Done when**
- [ ] Failure/success messages are more actionable
- [ ] No API contract breakage

---

### C2. Ensure terminal path docs include config mount contract
**Target refs**: `crates/ckrv-transport/src/handlers/terminal.rs`, docs sections  
**Priority**: P1  
**Estimate**: 20m

**Actions**
- Confirm docs mention actual runtime mapping:
  - `${HOME}/.config/kilo` -> `/home/kilo/.config/kilo`
- Confirm mention of image selection `ckrv-kilo:latest`

**Done when**
- [ ] Mount and image names in docs match code

---

## Phase D — Verification & Exit (P0)

### D1. Manual smoke test matrix execution
**Priority**: P0  
**Estimate**: 60m

Run and record outcomes for:
1. `kilo --version` unavailable (negative path)
2. `kilo models` output available/fallback
3. `ckrv task run --agent mistral-vibe -p "Create hello.txt"` success path
4. Optional terminal session start with `kilo_code` agent

**Done when**
- [ ] Results captured in PR description or check log
- [ ] At least one successful end-to-end Mistral-backed run documented

---

### D2. Acceptance checklist signoff
**Priority**: P0  
**Estimate**: 20m

- [ ] Mistral path is explicit in docs (not implied)
- [ ] Config and command contracts are concrete and copy-pasteable
- [ ] Failure modes have remediation text
- [ ] No native provider introduced for this issue
- [ ] Branch is clean and pushed

---

## 2) Dependency Graph

```text
A1 -> A2 -> B1 -> B2 -> (B3 optional)
A2 -> C2
(B1,B2,C2[,C1]) -> D1 -> D2
```

---

## 3) Test Matrix (Execution Checklist)

| ID | Layer | Test | Expected |
|---|---|---|---|
| T1 | Config | Valid `kilo_code` agent in `agents.yaml` loads via list agents | Agent returned with `kilo.model` |
| T2 | Validation | `test_agent` with Kilo binary present | success=true + version message |
| T3 | Validation | `test_agent` with Kilo binary missing | success=false + actionable error |
| T4 | Discovery | `kilo models` parse path | models list with provider/name/free fields |
| T5 | Discovery | `kilo models` failure path | fallback list returned |
| T6 | Runtime | Non-interactive task run with Mistral model | command exits 0, expected output generated |
| T7 | Runtime | Missing Kilo auth dir | run fails with auth/provider error; docs remediation works |
| T8 | Terminal | Start session with `kilo_code` | uses `ckrv-kilo:latest`, HOME `/home/kilo` |

---

## 4) Risks and Mitigations

1. **Model ID churn in Kilo catalog**
   - Mitigation: docs instruct `kilo models` discovery before pinning
2. **Auth mount absent in containerized/dev envs**
   - Mitigation: explicit mount troubleshooting + preflight check hint
3. **Terminology mismatch (“Mistral Vibe” vs technical type)**
   - Mitigation: always map phrase to concrete `agent_type: kilo_code`
4. **Over-scoping into native provider work**
   - Mitigation: hold line on scope gate A1

---

## 5) PR/Commit Requirements

Before merge:
- [ ] `git diff` shows only intended issue-29 artifacts (plus any doc/code updates explicitly required by this plan)
- [ ] commit message references issue #29
- [ ] branch `openclaw/issue-29` pushed to origin
- [ ] PR description includes smoke test evidence and acceptance checklist

---

## 6) Suggested Commit Plan

1. `docs(issue-29): define mistral vibe contract in brainstorm notes/tasks`
2. `docs(issue-29): add mistral vibe setup to agent guide/getting-started` (if executed in same branch)
3. `feat(issue-29): improve kilo test diagnostics` (optional)

This breakdown keeps review atomic and reversible.
