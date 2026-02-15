---
name: docs-order
description: Reference for documentation workflow execution order and dependencies.
---

# Documentation Workflow Order

This skill provides the execution order for documentation workflows. Reference this when completing any `/docs.*` workflow to know what to run next.

## Execution Order

| Step | Workflow | What It Does |
|------|----------|--------------|
| **1** | `/docs.rust` | Detects & fixes missing Rust docs (`///`, `//!`, `// ===`, CLI `long_about`/`after_help`) |
| **2** | `/docs.frontend` | Detects & fixes missing TSX docs (`@module`, Props JSDoc, state/effect comments, sections) |
| **3** | `/docs.skills` | Generates SKILL.md and command docs from CLI attributes |
| **4** | `/docs.update` | Updates crates/docs/*.md and crate READMEs |
| **5** | `/docs.readme` | Updates main README, CONTRIBUTING, npm/README |

## Dependency Graph

```
/docs.rust ──────┬──► /docs.skills ──► /docs.update ──► /docs.readme
                 │
/docs.frontend ──┘
```

## Key Dependencies

- `/docs.skills` must run after `/docs.rust` (reads CLI `long_about`/`after_help`)
- `/docs.update` must run after `/docs.skills` (references CLI command docs)
- `/docs.readme` must run last (reads from `crates/docs/*.md`)
- `/docs.rust` and `/docs.frontend` can run in parallel

## When to Run Each

| If you changed... | Run these workflows... |
|-------------------|------------------------|
| Rust code logic | `/docs.rust` → `/docs.update` |
| CLI commands | `/docs.rust` → `/docs.skills` → `/docs.update` → `/docs.readme` |
| Frontend components | `/docs.frontend` |
| API endpoints | `/docs.frontend` → `/docs.update` |
| Vision/messaging | `/docs.readme` only |
| Everything | All in order |

## Full Refresh

```bash
# Complete documentation refresh (in order):
/docs.rust
/docs.frontend  
/docs.skills
/docs.update
/docs.readme
```
