---
last_commit: c1bb442
last_updated: 2026-01-21
---

# API Contract: Documentation Frontmatter

## Schema

All documentation files in `crates/docs/` and `crates/<crate>/docs/` must include YAML frontmatter.

### Required Fields

```yaml
---
last_commit: string  # 7-character git short hash
last_updated: string # ISO date YYYY-MM-DD
---
```

### Optional Fields

```yaml
---
related_files:       # List of source files this doc covers
  - string           # Relative path from crate root
---
```

## Validation

### last_commit

- Pattern: `^[a-f0-9]{7}$`
- Must be a valid git commit in the repository
- Used for staleness detection

### last_updated

- Pattern: `^\d{4}-\d{2}-\d{2}$`
- Must be a valid date
- Should be the date the doc was last meaningfully updated

### related_files

- Each path must be relative to crate root
- Used to detect when docs need updating after code changes
- Optional but recommended for API documentation

## Examples

### Minimal

```yaml
---
last_commit: c1bb442
last_updated: 2026-01-21
---
```

### Full

```yaml
---
last_commit: c1bb442
last_updated: 2026-01-21
related_files:
  - src/lib.rs
  - src/agent/mod.rs
  - src/agent/claude.rs
---
```

## Error Cases

| Error | Cause | Resolution |
|-------|-------|------------|
| `Missing frontmatter` | No `---` delimiters | Add YAML frontmatter block |
| `Invalid commit hash` | Wrong length or characters | Use `git rev-parse --short HEAD` |
| `Invalid date format` | Not YYYY-MM-DD | Fix date format |
| `Unknown related file` | File doesn't exist | Update or remove path |
