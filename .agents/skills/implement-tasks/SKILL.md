---
name: implement-tasks
description: Pre-flight checklist and guardrails for implementing tasks and bugfixes from brainstorming docs. Use before writing any code to ensure you've read conventions, checked cross-crate impact, and aligned with guiding docs.
license: MIT
compatibility: Claude Code, Cursor, Kilo Code, any CLI-capable agent
metadata:
  author: FnSK4R17s
  version: "1.0"
---

# Implement Tasks & Bugfixes

A pre-flight checklist for agents implementing tasks from `brainstorming/<feature>/tasks.md` or `brainstorming/<feature>/bugfixNN.md`. Following this skill prevents broken builds, convention violations, and cross-crate regressions.

## Before You Write Any Code

**Do ALL of these steps first. Do not skip any.**

### 1. Read the guiding documents

```bash
# Always start here — understand the project vision and principles
cat guiding_docs/vision.md
```

Key things to internalize:
- **Target user**: Solo founders, senior ICs with 2+ AI subscriptions
- **Core principles**: Spec-first, fire-and-forget, isolation-is-safety, git-native
- **Non-goals**: Not another coding agent — orchestration layer only

### 2. Read the relevant conventions

**For Rust code** (anything in `crates/`):
```bash
cat crates/RUST_CONVENTIONS.md
```

Key requirements:
- Every `pub` item needs `///` doc comments
- Section separators: `// ============================================================`
- Imports: std → external → workspace → crate
- `long_about` and `after_help` on CLI commands
- **Clippy compliance**: Run `cargo clippy -p <crate> -- -D warnings` before finishing
- No bare `unwrap()` — use `.context()` or `ok_or_else()`

**For frontend code** (anything in `crates/ckrv-ui/frontend/`):
```bash
cat crates/ckrv-ui/FRONTEND_CONVENTIONS.md
```

Key requirements:
- `@module` header with `@description`, `@context`, `@dependencies`
- JSDoc on all Props interfaces
- State variable comments
- Components over 500 lines must be split
- OKLCH colors via CSS theme, no inline hex

### 3. Read the crate-specific docs

Before modifying any crate, read its docs:

```bash
# Find and read the crate's docs
cat crates/<crate-name>/docs/README.md

# Also check the crate's lib.rs for module-level docs
head -30 crates/<crate-name>/src/lib.rs
```

### 4. Understand the task

Read the full task or bugfix description. Pay attention to:
- **File(s)** listed — these are your primary targets
- **Depends on** — ensure prerequisite tasks are complete
- **Acceptance criteria** — these are your definition of done
- **Code snippets** — if the task includes proposed code, study it but don't blindly copy

## While Implementing

### 5. Check cross-crate impact

Before changing any public API (`pub fn`, `pub struct`, `pub enum`, `pub trait`):

```bash
# Find all callers of the function/type you're changing
grep -rn "function_name\|TypeName" crates/ --include="*.rs"

# Check workspace dependency graph
cargo tree -p <crate-name> --invert
```

**Rules**:
- If you change a public signature in `ckrv-git`, check `ckrv-cli` and `ckrv-core`
- If you change a public signature in `ckrv-sandbox`, check `ckrv-cli` and `ckrv-core`
- If you change types in `ckrv-core`, check **everything** — it's the hub crate
- If you add a new dependency to `Cargo.toml`, verify it doesn't duplicate functionality

### 6. Follow the existing patterns

Before writing new logic, find similar existing code:

```bash
# Find how the codebase already does something similar
grep -rn "similar_pattern" crates/ --include="*.rs" | head -10
```

| If you're doing... | Look at... |
|-------------------|-----------|
| Creating a worktree | `crates/ckrv-cli/src/commands/run.rs` lines 1096-1377 |
| Docker session lifecycle | `crates/ckrv-sandbox/src/docker.rs` |
| Interactive prompts | `crates/ckrv-cli/src/commands/term.rs` |
| Agent configuration | `crates/ckrv-cli/src/services/agent_lookup.rs` |
| CLI command structure | Any existing command in `crates/ckrv-cli/src/commands/` |
| API endpoints | `crates/ckrv-ui/src/api/` |
| Frontend components | `crates/ckrv-ui/frontend/src/components/` |

### 7. Keep files under size limits

| Lines | Status | Action |
|-------|--------|--------|
| < 300 | ✅ Good | Proceed |
| 300-500 | ⚠️ OK | Watch for growth |
| 500-800 | 🟠 Warning | Plan to split on next change |
| > 800 | 🔴 Alert | Must split into submodules |

If a file is already near the limit, extract the new code into a helper module instead of making it bigger.

## After Implementing

### 8. Build check

```bash
# Build the specific crate first
cargo build -p <crate-name>

# Then build workspace to catch cross-crate breakage
cargo build --workspace
```

### 9. Clippy check

```bash
# Check the crate you modified
cargo clippy -p <crate-name> -- -D warnings

# Common fixes:
# - &PathBuf → &Path
# - &Option<T> → Option<&T>
# - Self::Variant instead of EnumName::Variant
# - format!("{variable}") instead of format!("{}", variable)
# - Remove async from functions with no .await
```

### 10. Test check

```bash
# Run crate-specific tests
cargo test -p <crate-name>

# Run workspace tests if you changed a shared crate
cargo test --workspace
```

### 11. Update the task status

Mark the task as complete in the task/bugfix file:

```markdown
# Before
- [ ] Criterion 1

# After
- [x] Criterion 1
```

### 12. Mark task dependencies complete

If other tasks depend on yours, verify they haven't been invalidated by your changes.

## Common Pitfalls

| Pitfall | Prevention |
|---------|-----------|
| Changing a public API without updating callers | Step 5: grep for all usages first |
| Ignoring clippy warnings | Step 9: always run clippy after changes |
| Hardcoding paths or values | Use constants, config, or env vars |
| Adding code to an already-large file | Step 7: check line count, extract if needed |
| Not reading existing patterns first | Step 6: find similar code before writing new |
| Breaking the build in other crates | Step 8: `cargo build --workspace` |
| Skipping convention requirements | Steps 2-3: read conventions BEFORE coding |
| Missing doc comments on pub items | Conventions require `///` on every `pub` item |

## Quick Reference: Crate Responsibility Map

```
ckrv-cli        → CLI commands, user interaction, agent spawning
ckrv-core       → Orchestration, job management, domain types
ckrv-git        → Worktrees, branches, diffs (git2 bindings)
ckrv-sandbox    → Docker containers, agent providers, execution
ckrv-spec       → Spec parsing and validation
ckrv-metrics    → Cost/timing tracking
ckrv-verify     → Test execution/parsing (stub)
ckrv-ui         → Web dashboard (axum backend + React frontend)
ckrv-transport  → Shared API types for web + desktop
```

## Never Do

- ❌ `git commit` or `git push` — only the user commits
- ❌ Modify files in `specs/` unless explicitly asked
- ❌ Add `#[allow(...)]` without a comment explaining why
- ❌ Use `unwrap()` on user input or external data
- ❌ Skip reading conventions "because it's a small change"
- ❌ Inline hardcoded colors in frontend (use OKLCH theme variables)
- ❌ Create files outside the project directory
