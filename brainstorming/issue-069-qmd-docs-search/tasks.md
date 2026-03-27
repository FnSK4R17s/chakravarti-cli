# Explore QMD for Better Docs Search - Tasks

**Issue**: [#69](https://github.com/FnSK4R17s/chakravarti-cli/issues/69)
**Brainstorm**: [notes.md](./notes.md)
**Created**: 2026-03-02

## Task Overview

| Phase | Tasks | Estimate |
|-------|-------|----------|
| Phase 1: MCP & Permissions | 3 | 15m |
| Phase 2: Justfile Recipes | 2 | 30m |
| Phase 3: Index & Validate | 3 | 20m |
| **Total** | **8** | **~1h** |

> **Note**: First run of `just docs-index` will download ~2GB of GGUF models. This is a one-time cost not included in estimates.

---

## Dependencies

```
Phase 1 ──────────────────────────────────────────────►
  Task 1.1 ──┬─► Task 1.2
              └─► Task 1.3

Phase 2 ──────────────────────────────────────────────►
  Task 2.1 ──► Task 2.2

Phase 3 (requires Phase 1 + 2) ──────────────────────►
  Task 3.1 ──► Task 3.2 ──► Task 3.3
```

---

## Phase 1: MCP & Permissions

### Task 1.1: Add qmd MCP server to `.mcp.json`
**Priority**: P0
**Estimate**: 5m
**Files**: `.mcp.json`

Add a `qmd` entry alongside the existing `chrome-devtools` server. Use the same `npx -y` pattern.

```json
"qmd": {
  "type": "stdio",
  "command": "npx",
  "args": ["-y", "@tobilu/qmd", "mcp"],
  "env": {}
}
```

**Acceptance Criteria**:
- [ ] `.mcp.json` has `qmd` server entry
- [ ] Uses `npx -y` pattern matching chrome-devtools
- [ ] JSON is valid

---

### Task 1.2: Pre-approve qmd MCP tool permissions
**Priority**: P0
**Estimate**: 5m
**Files**: `.claude/settings.json`

Add all 6 qmd MCP tools to the `allow` array. All are read-only and safe to auto-approve.

Tools to add:
- `mcp__qmd__qmd_search`
- `mcp__qmd__qmd_vector_search`
- `mcp__qmd__qmd_deep_search`
- `mcp__qmd__qmd_get`
- `mcp__qmd__qmd_multi_get`
- `mcp__qmd__qmd_status`

**Acceptance Criteria**:
- [ ] All 6 tool permissions added to `.claude/settings.json`
- [ ] JSON is valid

---

### Task 1.3: Enable qmd in local settings
**Priority**: P0
**Estimate**: 5m
**Files**: `.claude/settings.local.json`

Add `"qmd"` to the `enabledMcpjsonServers` array.

**Acceptance Criteria**:
- [ ] `enabledMcpjsonServers` contains `"qmd"`
- [ ] JSON is valid

---

## Phase 2: Justfile Recipes

### Task 2.1: Add `docs-index` recipe
**Priority**: P1
**Estimate**: 20m
**Files**: `justfile`

Insert after the existing `docs` recipe (line ~257). This is the one-time setup recipe that creates 5 collections with context metadata and generates embeddings.

**Collections to create:**

| Collection | Path | Mask |
|------------|------|------|
| `ckrv-docs` | `./crates/` | `**/*.md` |
| `ckrv-commands` | `./crates/ckrv-cli/docs/commands/` | `**/*.md` |
| `ckrv-specs` | `./specs/` | `**/*.md` |
| `ckrv-brainstorming` | `./brainstorming/` | `**/*.md` |
| `ckrv-root` | `./` | `*.md` |

**Context metadata for each:**
- `/` → "Chakravarti CLI: spec-driven AI agent orchestration engine. Rust workspace with 13 crates."
- `ckrv-docs` → "Core development documentation: architecture, conventions, per-crate guides"
- `ckrv-commands` → "Auto-generated CLI command reference with flags, options, and exit codes"
- `ckrv-specs` → "Feature specifications: plans, tasks, contracts, data models, checklists"
- `ckrv-brainstorming` → "Feature exploration notes linked to GitHub issues"
- `ckrv-root` → "Project overview, development guidelines, documentation system guide"

Use `#!/usr/bin/env bash` with `set -euo pipefail`. End with `npx -y @tobilu/qmd embed`.

**Acceptance Criteria**:
- [ ] `just docs-index` creates all 5 collections
- [ ] Context metadata set for each collection
- [ ] Embeddings generated at the end
- [ ] Uses `npx -y @tobilu/qmd` consistently

---

### Task 2.2: Add search and utility recipes
**Priority**: P1
**Estimate**: 10m
**Files**: `justfile`

Add these recipes after `docs-index`:

| Recipe | Command |
|--------|---------|
| `docs-search query` | `npx -y @tobilu/qmd search "{{query}}" --md` |
| `docs-vsearch query` | `npx -y @tobilu/qmd vsearch "{{query}}" --md` |
| `docs-query query` | `npx -y @tobilu/qmd query "{{query}}" --md` |
| `docs-reindex` | `npx -y @tobilu/qmd update && npx -y @tobilu/qmd embed` |
| `docs-status` | `npx -y @tobilu/qmd status` |

All use `--md` output format for human and agent readability.

**Acceptance Criteria**:
- [ ] All 5 recipes added to justfile
- [ ] Each recipe has a descriptive comment
- [ ] `just --list` shows all new recipes under DOCUMENTATION section
- [ ] Recipes use `{{query}}` parameter syntax correctly

---

## Phase 3: Index & Validate

### Task 3.1: Run initial indexing
**Priority**: P1
**Estimate**: 5m (excludes model download)
**Files**: none (runtime only)

Run `just docs-index` from project root. First run will:
1. Download qmd via npx
2. Download ~2GB of GGUF models
3. Create 5 collections
4. Generate embeddings for all docs

**Acceptance Criteria**:
- [ ] `just docs-index` completes without errors
- [ ] `just docs-status` shows all 5 collections with document counts

---

### Task 3.2: Validate search quality
**Priority**: P1
**Estimate**: 10m
**Files**: none (runtime only)

Test all three search tiers:

| Test | Command | Expected top result |
|------|---------|---------------------|
| Keyword | `just docs-search "orchestrator"` | `crates/ckrv-core/docs/README.md` |
| Semantic | `just docs-vsearch "how do I add a new agent"` | `crates/docs/agent-guide.md` |
| Hybrid | `just docs-query "worktree isolation and safety"` | `crates/ckrv-git/docs/README.md` |

**Acceptance Criteria**:
- [ ] All three search tiers return relevant results
- [ ] `agent-guide.md` appears in top 3 for the semantic query
- [ ] Output is readable markdown

---

### Task 3.3: Validate MCP integration
**Priority**: P1
**Estimate**: 5m
**Files**: none (runtime only)

Start a fresh Claude Code session in the project and verify:
1. qmd MCP tools appear in the tool list
2. Tools are auto-approved (no permission prompts)
3. Agent can use `mcp__qmd__qmd_search` to find docs

**Acceptance Criteria**:
- [ ] qmd MCP server starts without errors
- [ ] All 6 tools are accessible
- [ ] No permission prompts for qmd tools
- [ ] Search returns results within Claude Code session

---

## Post-Implementation

After all tasks complete:
- [ ] Update brainstorm status to "Archived"
- [ ] Optionally add a "Doc Search" section to `crates/docs/getting-started.md`
