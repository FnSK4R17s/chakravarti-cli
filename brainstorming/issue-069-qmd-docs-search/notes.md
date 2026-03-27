# Explore QMD for Better Docs Search

**Issue**: [#69](https://github.com/FnSK4R17s/chakravarti-cli/issues/69)
**Created**: 2026-03-02
**Status**: Tasks Generated

## Problem Statement

Chakravarti CLI has ~50+ well-structured markdown docs (architecture, conventions, per-crate guides, command references, specs, brainstorms) but no way to semantically search them. Agents working on the codebase rely on file reads and grep, which misses conceptual matches.

Example: an agent asking "how do I add a new agent" would need to know to look at `crates/docs/agent-guide.md` — grep for "add agent" might not find it.

## Current State

**Docs inventory:**

| Location | Count | Content |
|----------|-------|---------|
| `crates/docs/` | 6 | Architecture, agent guide, CLI commands, getting started |
| `crates/*/docs/README.md` | 13 | Per-crate documentation |
| `crates/ckrv-cli/docs/commands/` | 21 | Auto-generated command docs |
| Conventions files | 2 | `RUST_CONVENTIONS.md`, `FRONTEND_CONVENTIONS.md` |
| `.agent/skills/*/SKILL.md` | 1 | Auto-generated agent skills |
| `brainstorming/` | ~44 | Feature exploration notes |
| `specs/` | ~170 | Feature specifications |

**Current doc tooling:**
- `just docs` → `cargo doc --no-deps --open` (rustdoc only)
- `just skill` → generates SKILL.md from CLI metadata
- `command_docs_gen` binary → auto-generates per-command markdown
- 100% module doc coverage per health report

**Pain points:**
- No unified search across all markdown docs
- Agents can't discover docs by concept, only by filename/keyword
- No MCP interface for doc retrieval
- `just docs` only opens rustdoc, doesn't surface markdown docs

## Proposed Solution

Integrate [qmd](https://github.com/tobi/qmd) — a local semantic search engine for markdown — to make all project docs queryable by both humans and AI agents.

qmd provides:
- **BM25 full-text search** (SQLite FTS5) for fast keyword matching
- **Vector semantic search** using local GGUF embeddings
- **LLM re-ranking** with query expansion for highest-quality results
- **MCP server** so agents can query docs directly
- **Collection management** with context metadata

## User Stories

### US1: Agent Doc Discovery
**As a** Claude Code agent working on the chakravarti-cli codebase,
**I want** to semantically search project documentation via MCP tools,
**So that** I can find relevant architecture docs, conventions, and guides without knowing exact file paths.

### US2: Developer Quick Search
**As a** developer contributing to chakravarti-cli,
**I want** to run `just docs-search "worktree isolation"` from the terminal,
**So that** I can find relevant docs without manually browsing the directory tree.

## Technical Approach

### Options Considered

| Option | Pros | Cons |
|--------|------|------|
| **qmd** (selected) | Semantic + keyword search, MCP server, local-only, collection metadata | Node >= 22 required, ~2GB model download on first run |
| Custom grep-based search | Zero dependencies, instant | No semantic search, keyword-only |
| mdbook with search | Nice HTML site, built-in search | Keyword-only, heavy setup, doesn't help agents |

### Decision

**qmd** — it's the only option that provides semantic search AND an MCP server interface. The Node.js requirement is already met (v24.11.0 for frontend), and the `npx` invocation pattern matches our existing `.mcp.json` setup for chrome-devtools.

## Implementation Notes

### Installation: `npx` (no permanent install)
- `npx -y @tobilu/qmd` — matches existing chrome-devtools MCP pattern
- No root `package.json` needed (project is Rust-first)
- Optional: `npm install -g @tobilu/qmd` for heavy users

### 5 Collections

| Collection | Path | Mask | Content |
|------------|------|------|---------|
| `ckrv-docs` | `./crates/` | `**/*.md` | Architecture, conventions, per-crate guides |
| `ckrv-commands` | `./crates/ckrv-cli/docs/commands/` | `**/*.md` | Auto-generated CLI command docs |
| `ckrv-specs` | `./specs/` | `**/*.md` | Feature specifications |
| `ckrv-brainstorming` | `./brainstorming/` | `**/*.md` | Exploration notes |
| `ckrv-root` | `./` | `*.md` (non-recursive) | README, CLAUDE.md, top-level docs |

Each collection gets descriptive context metadata so qmd understands the hierarchy.

### Files Modified (4 files, 0 new config files)

1. **`.mcp.json`** — Add `qmd` MCP server entry alongside chrome-devtools
2. **`.claude/settings.json`** — Pre-approve 6 read-only qmd MCP tool permissions
3. **`.claude/settings.local.json`** — Add `"qmd"` to `enabledMcpjsonServers`
4. **`justfile`** — Add `docs-*` recipes after existing DOCUMENTATION section

### Justfile Recipes

| Recipe | Purpose |
|--------|---------|
| `docs-index` | One-time setup: create collections, add context metadata, generate embeddings |
| `docs-search <query>` | Fast BM25 keyword search |
| `docs-vsearch <query>` | Semantic vector search |
| `docs-query <query>` | Full hybrid + LLM re-ranking (best quality, slowest) |
| `docs-reindex` | Refresh index after doc changes |
| `docs-status` | Show index status |

### MCP Tools Exposed to Agents

All read-only, safe to pre-approve:
- `qmd_search` — keyword search
- `qmd_vector_search` — semantic search
- `qmd_deep_search` — hybrid with re-ranking
- `qmd_get` — fetch specific document
- `qmd_multi_get` — fetch multiple documents by pattern
- `qmd_status` — check index status

### Indexing Strategy

Manual indexing — `just docs-index` once, `just docs-reindex` after doc changes. No file watchers or build hooks because:
- Embedding generation is slow (first run downloads ~2GB of models)
- Docs change infrequently
- qmd's SQLite index persists at `~/.cache/qmd/` across sessions

## Open Questions

- [ ] Should `just skill` chain to `docs-reindex` automatically?
- [ ] Do we need all 5 collections, or would 2-3 (docs + specs + root) suffice to start?
- [ ] Worth adding a `docs-mcp` recipe for HTTP daemon mode (useful for multi-agent setups)?

## Success Criteria

| Metric | Target |
|--------|--------|
| `just docs-search "orchestrator"` finds relevant docs | `crates/ckrv-core/docs/README.md`, `architecture.md` |
| `just docs-vsearch "how do I add a new agent"` | Returns `agent-guide.md` in top 3 |
| Claude Code agent can use `mcp__qmd__qmd_search` | Tools appear and are auto-approved |
| No new config files in repo | qmd state stays in `~/.cache/qmd/` |

## Vision Alignment

From `vision.md`:
> "Repos benefit from being agent-ready. Poorly documented codebases produce inconsistent results."

qmd makes existing docs **discoverable** to agents without changing the docs themselves. It's infrastructure that improves agent effectiveness — aligned with ckrv's role as an orchestration layer.

## Next Steps

- [ ] Implement the 4-file change (`.mcp.json`, settings, justfile)
- [ ] Run `just docs-index` and validate search results
- [ ] Test MCP integration in a fresh Claude Code session
- [ ] Document in `crates/docs/getting-started.md` (optional setup step)

## References

- [qmd GitHub](https://github.com/tobi/qmd)
- [Issue #69](https://github.com/FnSK4R17s/chakravarti-cli/issues/69)
- [Plan file](/home/sk4r/.claude/plans/nifty-gathering-wall.md) (session plan)
