# Dogfooding: Using ckrv to Build ckrv

**Created**: 2026-02-12
**Status**: In Progress

## Problem Statement

ckrv is built today using direct Claude Code sessions enhanced by `.claude/skills/`, conventions files, and workflow automations. This works well but doesn't leverage ckrv's own orchestration capabilities — the very thing it's designed to do. We should be using ckrv's multi-agent, Docker-sandboxed, worktree-based execution to build ckrv itself.

**The preferred workflow:** `/brainstorming` → `/brainstorm-to-tasks` → `ckrv run`. Simple, direct, no spec overhead.

The gap: ckrv orchestrates agents in parallel across isolated environments, but self-development is single-agent, interactive, and entirely Claude Code dependent.

## Current State

### What works well (and should be preserved)

| Capability | How it works today |
|-----------|-------------------|
| `ckrv term` → agent sessions | Spin up any configured agent in the repo |
| `.claude/skills/` workflows | Brainstorming, speckit, docs-order pipelines |
| CLAUDE.md + conventions | Guide every Claude Code session automatically |
| Vercel skills | Enforce UI quality in frontend work |
| Docs-order pipeline | `.rust` → `.skills` → `.update` → `.readme` |
| Agent configuration | 5 agent types via `~/.config/chakravarti/agents.yaml` |
| `ckrv run` core pipeline | Batch execution, worktrees, merge, resume |
| Complexity-based routing | `find_best_agent_for_level()` matches agents to task complexity |
| Docker sandboxing | Container execution via bollard |

### Pain points

- Single-agent bottleneck — one Claude Code session at a time
- Skills and conventions are Claude Code specific — Codex/Kilo get raw prompts
- No bridge from speckit output (tasks.md markdown) to ckrv run input (tasks.yaml)
- Manual verification — `cargo test`, `cargo clippy`, `npx tsc` run by hand
- Docs refresh requires interactive session through 5 sequential skills

## Proposed Solution

### Vision Alignment

From `guiding_docs/vision.md`:
- *"Fire and forget"* — kick off work, walk away, come back to review
- *"Match the right agent to the right job"* — complexity-based routing
- *"Agents execute in isolation"* — Docker sandboxes + git worktrees
- *"Not another coding agent — orchestration layer only"* — ckrv coordinates, agents implement

Dogfooding directly validates all four principles on the codebase that implements them.

### The Self-Development Workflow (Simplified: 3 Phases)

```
Phase 1               Phase 2                Phase 3
Brainstorm  →   Tasks Generation  →   Execute & Verify
(interactive)      (interactive)          (fire & forget)
ckrv term          ckrv term              ckrv run + verify
Claude native      Claude native          ALL agents
```

#### Phase 1 — Brainstorm (`ckrv term`, interactive)

**Agent:** Claude Code (native) — needs skill access, CLAUDE.md context

```
ckrv term --agent claude-native
> /brainstorming           # Create brainstorm doc linked to GitHub issue
```

The brainstorming skill creates `brainstorming/issue-{NNN}-{slug}/notes.md` with:
- Problem statement
- Current state / pain points
- Proposed solution
- Technical approach (options considered, decisions)
- User stories
- Implementation notes
- Open questions

**Output:** `brainstorming/issue-{NNN}-{slug}/notes.md`

#### Phase 2 — Tasks Generation (`ckrv term`, interactive)

**Agent:** Claude Code (native) — converts brainstorm to executable tasks

```
ckrv term --agent claude-native
> /brainstorm-to-tasks     # Generate tasks.md from brainstorm
```

The brainstorm-to-tasks skill creates `tasks.md` in the same folder with:
- Phases (logical groupings)
- Tasks with priorities (P0/P1/P2)
- Estimates (time)
- File paths affected
- Acceptance criteria per task
- Dependencies

**Output:** `brainstorming/issue-{NNN}-{slug}/tasks.md`

#### Phase 3 — Execute & Verify (`ckrv run` + `ckrv verify`, fire-and-forget)

**Mode:** Automated, Docker sandboxed, parallel agents on worktrees

```bash
ckrv run specs/{feature}/spec.yaml
```

Pipeline:
1. Loads `spec.yaml` + `tasks.yaml` from spec directory
2. AI groups pending tasks into execution batches with dependencies
3. Each batch gets a worktree + Docker container + assigned agent
4. Agents execute in parallel (respecting batch dependencies)
5. Results merge back to main branch incrementally
6. `tasks.yaml` and `plan.yaml` updated as batches complete

**Agent routing by task complexity:**

| Complexity | Agent Choice | Use Case |
|-----------|-------------|----------|
| 1-2 | Kilo (cheap models) or Codex | Boilerplate, formatting, simple additions |
| 3 | OpenRouter (mid-tier) or GLM | Standard features, test writing |
| 4-5 | Claude Code (native) | Architecture, complex refactors, cross-crate changes |

#### Phase D — Verification (`ckrv verify`, automated)

```bash
ckrv verify
```

Per-crate quality gates:
- **Rust crates:** `cargo clippy --workspace -- -D warnings` + `cargo test --workspace` + `cargo fmt --check`
- **Frontend:** `npx tsc --noEmit` + `npm run build`
- **Cross-crate:** `cargo doc --no-deps`

Failures trigger `ckrv fix` with the appropriate agent.

#### Phase E — Review & Promote

```bash
ckrv diff                    # Review all changes across worktrees
ckrv promote --push --open   # Create PR
```

Human reviews the PR. If changes needed, iterate from Phase C.

#### Phase F — Documentation Refresh (`ckrv term`, eventually automated)

```
ckrv term --agent claude-native
> /docs.rust        → /docs.frontend     (parallel)
> /docs.skills      (depends on .rust)
> /docs.update      (depends on .rust + .frontend)
> /docs.readme      (depends on .update)
```

Eventually becomes a `ckrv run` pipeline with 5 sequential tasks (see Gap 5).

## User Stories

### US1: Self-Development with Multi-Agent Execution
**As a** ckrv developer,
**I want** to run `ckrv run` on features I brainstomed for ckrv,
**So that** multiple agents implement tasks in parallel across isolated worktrees while I review the results.

### US2: Convention-Aware Agent Execution
**As a** ckrv developer,
**I want** all agents (not just Claude) to follow project conventions during execution,
**So that** code quality is consistent regardless of which agent implements a task.

### US3: Brainstorm-to-Run Pipeline
**As a** ckrv developer,
**I want** tasks generated by `/brainstorm-to-tasks` to feed directly into `ckrv run`,
**So that** I don't manually convert task formats between planning and execution.

## Technical Approach

### Gaps to Close

#### Gap 1: Brainstorm Tasks → ckrv Run Format (Sprint 1)

**Problem:** `/brainstorm-to-tasks` produces `tasks.md` (markdown). The `ckrv run` pipeline consumes `tasks.yaml` (YAML with specific structure).

**The mismatch:**

| tasks.md (from brainstorm-to-tasks) | tasks.yaml (ckrv run) |
|-----------------------------------|----------------------|
| Phases (logical groupings) | Batches (execution groups with dependencies) |
| P0/P1/P2 priorities | `complexity` 1-5 (for agent routing) |
| Time estimates (30m, 2h, etc.) | `model_tier` override (optional) |
| File paths | `file` field |
| Acceptance criteria | `description` field |
| Dependencies | `depends_on` at batch level |

**Options Considered:**

| Option | Pros | Cons |
|--------|------|------|
| A: New `ckrv tasks convert` command | Clean separation, keeps skill simple | Extra manual step |
| B: Teach `ckrv run` to parse tasks.md | Zero friction, one less step | Fragile markdown parsing |
| C: Update brainstorm-to-tasks to emit tasks.yaml | Single step produces both formats | Couples skill to ckrv internals |

**Decision:** Option A — `ckrv tasks convert` command. Keeps the brainstorming skill simple and focused on its job (converting brainstorm → tasks), while ckrv handles the bridge to its internal format.

**Files to modify:**
- `crates/ckrv-cli/src/commands/tasks.rs` (new) — convert subcommand
- `crates/ckrv-spec/src/loader.rs` — markdown parser for tasks.md format

---

#### Gap 2: Cross-Agent Context via Symlinks (Sprint 1)

**Problem:** Codex and Kilo in Docker don't read `.claude/` or `CLAUDE.md`. They get raw task prompts with no project conventions. Prompt injection is fragile and duplicates content.

**Existing Infrastructure:** The codebase already has:
- `scripts/agent-switch.sh` — cycles between `.agent/`, `.claude/`, `.opencode/` folder structures
- `.specify/scripts/bash/update-agent-context.sh` — creates agent-specific instruction files:
  - `CLAUDE.md` for Claude Code
  - `AGENTS.md` for multi-agent coordination
  - `.cursor/rules/specify-rules.mdc` for Cursor
  - `.windsurf/rules/specify-rules.md` for Windsurf
  - `.kilocode/rules/specify-rules.md` for Kilo Code

**Solution:** Instead of prompt injection, use **symlinks** so each agent reads its native instruction format from a shared source:

```
Source of truth:           Agent reads natively via symlink:
-----------------         ----------------------------------
crates/
  ├── RUST_CONVENTIONS.md  ──→ .claude/CLAUDE.md (Claude reads directly)
  ├── ckrv-ui/
      └── FRONTEND_CONVENTIONS.md
                          ──→ .cursor/rules/specify-rules.mdc (via symlink)
                          ──→ .windsurf/rules/specify-rules.md (via symlink)
                          ──→ .kilocode/rules/specify-rules.md (via symlink)
                          ──→ AGENTS.md (for Codex, OpenRouter, GLM)
```

**Why symlinks > prompt injection:**
- Agents read their native format — Cursor expects `.cursor/rules/*.mdc`, not injected text
- Single source of truth — conventions live in `crates/`, symlinks point to them
- Works in Docker — mount symlinks into containers, agents follow them natively
- No prompt bloat — no prepending megabytes of text to every task

**Implementation approaches:**

| Approach | Pros | Cons |
|----------|------|------|
| A: `ckrv init` creates symlinks | One-time setup, cleanest | Need to detect when conventions change |
| B: `.specify` script generates agent files | Already exists, maintains MANAGED sections | Duplicates content into each agent file |
| C: Symlink layer + `.specify` hybrid | Symlinks for static conventions, generated for dynamic | More complex setup |

**Decision:** Start with Approach B (existing `.specify` script) since it already works and handles MANAGED sections for project-specific data. Evaluate A for simplification later.

**What already exists (no changes needed):**
- `scripts/agent-switch.sh` — toggles between `.agent/` ↔ `.claude/` ↔ `.opencode/`
- `.specify/scripts/bash/update-agent-context.sh` — generates agent files from `plan.md`
- Template at `.specify/templates/agent-file-template.md`

**What needs enhancement:**
- Extend `.specify` script to also include `RUST_CONVENTIONS.md` content
- Create symlinks for `.cursor/`, `.windsurf/`, `.kilocode/` folders pointing to generated files
- Document which agents read which files

**Files to modify:**
- `.specify/scripts/bash/update-agent-context.sh` — add convention extraction from `RUST_CONVENTIONS.md` and `FRONTEND_CONVENTIONS.md`
- Add symlink creation for agent folders in `.specify` workflow

---

#### Gap 3: Symlink Mounting for Docker (Sprint 1 or 2)

**Problem:** Symlinks created at project root don't resolve inside Docker containers without proper mounting.

**Solution:** Mount agent instruction symlinks into Docker containers as part of `config_mounts()`:
- `.claude/CLAUDE.md` (already mounted by Claude provider)
- `.cursor/rules/specify-rules.mdc` (new Cursor provider mount)
- `.windsurf/rules/specify-rules.md` (new Windsurf provider mount)
- `.kilocode/rules/specify-rules.md` (new Kilo provider mount)
- `AGENTS.md` (already accessible via worktree mount)

**Note:** With symlinks pointing to source conventions, mounting the symlink folder is sufficient — no need to duplicate content.

**Files to modify:**
- `crates/ckrv-sandbox/src/agent/{cursor,windsurf,kilo}.rs` — add instruction file mounts
- Each agent provider's `config_mounts()` method

---

#### Gap 4: Crate-Aware Task Routing (Sprint 2)

**Problem:** The run pipeline doesn't know which crate a task targets — can't apply right conventions or verification.

**Solution:** Tasks include a `crate` and `type` field:

```yaml
tasks:
  - id: "T001"
    crate: "ckrv-ui/frontend"    # target crate
    type: "frontend"              # rust | frontend | docs | mixed
```

Used to:
- Choose Rust vs Frontend conventions for injection
- Set working directory to correct crate
- Run crate-specific verification after completion

**Files to modify:**
- `crates/ckrv-spec/src/` — task type definitions
- Speckit skill templates — include crate/type in task generation

---

#### Gap 5: Post-Batch Verification (Sprint 2)

**Problem:** No automated verification after each batch completes. Failures only caught manually.

**Solution:** Per-batch verification commands based on crate type:
- After Rust batch: `cargo check -p {crate}` + `cargo clippy -p {crate}`
- After Frontend batch: `npx tsc --noEmit`
- On failure: auto-trigger `ckrv fix` with the same agent

**Files to modify:**
- `crates/ckrv-cli/src/commands/run.rs` — post-merge verification step
- `crates/ckrv-core/src/` — verification config per crate type

---

#### Gap 6: Docs Pipeline as ckrv run Tasks (Sprint 3)

**Problem:** The docs-order pipeline (5 sequential skills) is interactive-only.

**Solution:** Create a `docs-refresh` spec template with 5 pre-defined tasks:

| Task | Complexity | Depends On | Parallel |
|------|-----------|-----------|----------|
| T001: docs.rust conventions | 3 | none | yes (with T002) |
| T002: docs.frontend conventions | 3 | none | yes (with T001) |
| T003: Generate SKILL.md from CLI | 2 | T001 | no |
| T004: Update crates/docs/*.md | 3 | T001, T002 | no |
| T005: Update README | 2 | T004 | no |

Fire-and-forget via `ckrv run specs/docs-refresh/spec.yaml`.

**Files to create:**
- `specs/templates/docs-refresh/spec.yaml`
- `specs/templates/docs-refresh/tasks.yaml`

## Implementation Notes

### Prioritized Sprint Plan

**Sprint 1: Minimum Viable Dogfooding**
1. Gap 1 (tasks.md → tasks.yaml) — unblocks speckit → run pipeline
2. Gap 2 (cross-agent symlinks) — all agents read conventions natively via symlinks

**Sprint 2: Quality & Routing**
3. Gap 4 (crate-aware routing) — right conventions for right tasks
4. Gap 5 (post-batch verification) — catch failures early

**Sprint 3: Full Automation**
5. Gap 6 (docs pipeline as tasks) — fire-and-forget docs refresh
6. Evaluate symlink simplification — can we reduce duplication further?

### Day-One Playbook (What Works Right Now)

Even before closing any gaps, here's how to dogfood today:

1. **Brainstorm** — `ckrv term --agent claude-native` → `/brainstorming`
2. **Tasks** — `/brainstorm-to-tasks` → generates tasks.md in same folder
3. **Manual convert** — Copy tasks.md structure into tasks.yaml format (temp until Gap 1)
4. **Run** — Configure agents in agents.yaml (Claude=5, OpenRouter=3, Codex=3, Kilo=2) → `ckrv run`
5. **Verify + Promote** — `ckrv verify` → `ckrv promote --push --open`
6. **Docs** — `ckrv term` → docs-order pipeline manually

### Key Architecture Decisions

- **Phases A, B, F stay interactive** — skills are Claude Code native, that's fine
- **Phase C is where orchestration value lives** — fire-and-forget, parallel, multi-agent
- **Symlinks > prompt injection** — each agent reads its native format from shared source via symlinks
- **`.specify` script is the bridge** — already generates agent files, extend to pull from `RUST_CONVENTIONS.md` and `FRONTEND_CONVENTIONS.md`
- **Docker sandboxing is the target** — local worktrees work for bootstrapping, Docker for production self-dev

## Open Questions

- [ ] Should `ckrv tasks convert` be AI-assisted (use Claude to parse markdown) or deterministic (regex/parser)?
- [ ] Symlinks vs duplication: Should `.specify` generate separate files per agent (current behavior) or create symlinks to shared conventions?
- [ ] Should the docs-refresh template be a built-in `ckrv docs` command or a regular spec?
- [ ] What's the right first feature to dogfood on? Kilo Code integration has the brainstorm ready.
- [ ] How do Cursor/Windsurf/Kilo instruction formats differ? Need to research their native rule file formats.

## Success Criteria

| Metric | Target |
|--------|--------|
| End-to-end dogfood loop | Complete one feature using the full 1→2→3 workflow |
| Multi-agent execution | At least 3 different agent types used in one `ckrv run` |
| Convention compliance | Non-Claude agents produce code following project conventions |
| Automation ratio | Phase 3 fully automated (no manual intervention) |
| Simplicity | 3-step workflow (brainstorm → tasks → execute) feels natural, not burdensome |

## Next Steps

- [ ] Try the Day-One Playbook on a real feature (Kilo Code integration?)
- [ ] Document friction points from first real dogfood attempt
- [ ] Implement Gap 1 (brainstorm tasks → ckrv run format) based on friction findings
- [ ] Implement Gap 2 (cross-agent symlinks) for all agents

## References

- `guiding_docs/vision.md` — Product vision and principles
- `scripts/agent-switch.sh` — Agent folder cycling (`.agent/` ↔ `.claude/` ↔ `.opencode/`)
- `.specify/scripts/bash/update-agent-context.sh` — Agent file generation from plan.md
- `.specify/templates/agent-file-template.md` — Template for generated agent files
- `.claude/skills/vercel-react-native-skills/AGENTS.md` — Example AGENTS.md structure
- `.claude/skills/brainstorm-to-tasks/SKILL.md` — Brainstorm to tasks conversion skill
- `.claude/skills/brainstorming/SKILL.md` — Brainstorming workflow

---

## Key Insight: Symlinks Over Prompt Injection

The original plan included "Gap 2: Convention Injection" — prepending convention text to non-Claude agent prompts. After exploring the codebase, a **much better approach emerged**:

**Why symlinks > prompt injection:**
1. **Native format compatibility** — Each agent reads its own format:
   - Claude Code: `CLAUDE.md` (markdown)
   - Cursor: `.cursor/rules/*.mdc` (markdown with metadata)
   - Windsurf: `.windsurf/rules/*.md`
   - Kilo Code: `.kilocode/rules/*.md`
2. **Single source of truth** — Conventions live in `crates/`, symlinks point to them
3. **Works in Docker** — Mount symlinks into containers, agents follow them natively
4. **No prompt bloat** — Avoid prepending megabytes of text to every task

**Existing infrastructure we can leverage:**
- `scripts/agent-switch.sh` — already manages `.agent/` ↔ `.claude/` ↔ `.opencode/` cycling
- `.specify/scripts/bash/update-agent-context.sh` — already generates agent-specific files from `plan.md`
- `.specify/templates/agent-file-template.md` — template structure with MANAGED sections

**The symlink strategy:**
```
Source of truth:              Agent reads natively via symlink:
-----------------            ----------------------------------
crates/
  ├── RUST_CONVENTIONS.md   ──→ .claude/CLAUDE.md (Claude reads directly)
  ├── ckrv-ui/
       └── FRONTEND_CONVENTIONS.md
                           ──→ .cursor/rules/specify-rules.mdc (via symlink)
                           ──→ .windsurf/rules/specify-rules.md (via symlink)
                           ──→ .kilocode/rules/specify-rules.md (via symlink)
                           ──→ AGENTS.md (for Codex, OpenRouter, GLM)
```

This aligns perfectly with ckrv's "isolation through architecture" principle — each agent stays in its native environment, while sharing conventions through proper abstraction (symlinks) rather than duplication (prompt injection).
- `crates/ckrv-cli/src/commands/run.rs` — Run pipeline implementation
- `crates/ckrv-cli/src/commands/term.rs` — Term command implementation
- `crates/ckrv-sandbox/src/agent/` — Agent provider implementations
- `crates/RUST_CONVENTIONS.md` — Rust code conventions
- `crates/ckrv-ui/FRONTEND_CONVENTIONS.md` — Frontend code conventions
- `.claude/skills/speckit-order/SKILL.md` — Speckit workflow order
- `.claude/skills/docs-order/SKILL.md` — Docs generation order
