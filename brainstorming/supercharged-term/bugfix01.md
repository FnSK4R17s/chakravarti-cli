# Supercharged `ckrv term` — Bugfix Tasks (01)

**Brainstorm**: [notes.md](./notes.md)
**Created**: 2026-02-16
**Source**: Post-implementation review in notes.md

## Bugfix Overview

| # | Bug | Severity | Estimate |
|---|-----|----------|----------|
| BF-01 | `create_worktree()` is needlessly `async` | Low | 10m |
| BF-02 | `resume_session()` crashes on sandbox-only sessions | High | 30m |
| BF-03 | `execute_in_sandbox()` mount path collision | Critical | 20m |
| BF-04 | Hardcoded commit message — should be interactive | Medium | 25m |
| BF-05 | 62 clippy warnings in `term.rs` | Medium | 30m |

**Severity breakdown**: 1 Critical, 1 High, 2 Medium, 1 Low
**Total estimate**: ~2h

---

## BF-01: `create_worktree()` is needlessly `async`

**Severity**: Low
**File(s)**: `crates/ckrv-cli/src/commands/term.rs:747`
**Estimate**: 10m

### Problem

`create_worktree()` is declared `async` but performs only synchronous `git2` operations via `DefaultWorktreeManager`. No `.await` is ever called inside the function body.

```rust
// Current — async with no awaits
async fn create_worktree(
    session_id: &str,
    cwd: &PathBuf,
    ui: &UiContext,
    json: bool,
) -> anyhow::Result<Worktree> {
    let manager = DefaultWorktreeManager::new(cwd)?;
    let job_id = session_id.strip_prefix("term-").unwrap_or(session_id);
    let worktree = manager.create(job_id, "1")?;
    // ... no .await calls anywhere
    Ok(worktree)
}
```

### Fix

Remove `async` from the function signature. Update all call sites from `.await?` to just `?`.

```rust
// Fixed — sync function
fn create_worktree(
    session_id: &str,
    cwd: &Path,         // Also fix &PathBuf → &Path (clippy)
    ui: &UiContext,
    json: bool,
) -> anyhow::Result<Worktree> {
    let manager = DefaultWorktreeManager::new(cwd)?;
    let job_id = session_id.strip_prefix("term-").unwrap_or(session_id);
    let worktree = manager.create(job_id, "1")?;
    // ...
    Ok(worktree)
}
```

Update call sites:

```rust
// Before
let worktree_info = if args.worktree {
    Some(create_worktree(&session_id, &cwd, ui, json).await?)
} else { None };

// After
let worktree_info = if args.worktree {
    Some(create_worktree(&session_id, &cwd, ui, json)?)
} else { None };
```

### Acceptance Criteria

- [ ] `create_worktree()` is a sync `fn`, not `async fn`
- [ ] All call sites use `?` instead of `.await?`
- [ ] Clippy warning `unused async for function with no await statements` is resolved
- [ ] `&PathBuf` parameter changed to `&Path`
- [ ] `ckrv term --worktree` still creates worktrees correctly

---

## BF-02: `resume_session()` crashes on sandbox-only sessions

**Severity**: High
**File(s)**: `crates/ckrv-cli/src/commands/term.rs:620-627`
**Estimate**: 30m

### Problem

When resuming a session created with `--sandbox` alone (no `--worktree`), `resume_session()` unconditionally requires `worktree_path` to exist. Since sandbox-only sessions have `worktree_path: None`, the `.ok_or_else()` always errors.

```rust
// Current — crashes when worktree_path is None
let worktree_path = state
    .worktree_path
    .as_ref()
    .map(|p| PathBuf::from(p))
    .filter(|p| p.exists())
    .ok_or_else(|| {
        anyhow::anyhow!("Worktree for session '{}' no longer exists", session_name)
    })?;
```

This means:
- `ckrv term --sandbox --name my-session` → creates session ✅
- `ckrv term --resume my-session` → **crashes** ❌

### Fix

**Part A**: Handle `worktree_path: None` gracefully — fall back to `cwd`:

```rust
// Fixed — handles sandbox-only sessions
let working_dir = match &state.worktree_path {
    Some(path) => {
        let p = PathBuf::from(path);
        if !p.exists() {
            return Err(anyhow::anyhow!(
                "Worktree for session '{}' no longer exists at {}",
                session_name, path
            ));
        }
        p
    }
    None => cwd.clone(), // Sandbox-only: use cwd
};
```

**Part B**: Store mode flags in `SessionState` so resume doesn't require re-specifying them:

```rust
// Add to SessionState struct
#[derive(Debug, Clone, Serialize, Deserialize)]
struct SessionState {
    name: String,
    agent_id: String,
    sandbox: bool,          // NEW — was session created with --sandbox?
    worktree: bool,         // NEW — was session created with --worktree?
    container_id: Option<String>,
    worktree_path: Option<String>,
    worktree_branch: Option<String>,
    created_at: DateTime<Utc>,
    status: SessionStatus,
}
```

**Part C**: In `resume_session()`, use stored mode instead of requiring re-specification:

```rust
// Resume reads mode from session state
let use_sandbox = state.sandbox; // From stored state, not from args
```

### Acceptance Criteria

- [ ] `ckrv term --sandbox --name test` creates a session successfully
- [ ] `ckrv term --resume test` resumes without crash (uses cwd, not worktree)
- [ ] `ckrv term --worktree --name test2` → `--resume test2` still works
- [ ] `ckrv term --sandbox --worktree --name test3` → `--resume test3` works
- [ ] `SessionState` YAML includes `sandbox` and `worktree` booleans
- [ ] Existing session YAML files are backwards-compatible (default `false` for missing fields)

---

## BF-03: `execute_in_sandbox()` mount path collision

**Severity**: Critical
**File(s)**: `crates/ckrv-cli/src/commands/term.rs:944-952`
**Estimate**: 20m

### Problem

The Docker `create_session()` call uses the host path as both the mount source AND mount target:

```rust
// Current — host path used as container path (WRONG)
let workdir_str = working_dir.to_string_lossy().to_string();
let container_id = docker.create_session(
    &workdir_str,    // container workdir → e.g., /apps/chakravarti-cli
    &workdir_str,    // mount source (host) → /apps/chakravarti-cli
    &workdir_str,    // mount target (container) → /apps/chakravarti-cli ← BROKEN
    env_map.clone(),
    extra_mounts,
).await?;
```

The container filesystem doesn't have `/apps/chakravarti-cli`. The Docker image uses `/workspace` as its canonical workdir (see `docker/Dockerfile.claude` line 6: `WORKDIR /workspace`).

**Impact**: The agent will fail to start inside the container because the working directory path doesn't exist in the container filesystem.

### Fix

Use `/workspace` as the container mount target, matching all other Docker usages in the codebase (`run.rs`, UI terminal modal):

```rust
// Fixed — use /workspace as container target
let host_path = working_dir.to_string_lossy().to_string();
let container_workdir = "/workspace";

let container_id = docker.create_session(
    container_workdir,   // container workdir → /workspace
    &host_path,          // mount source (host) → /apps/chakravarti-cli
    container_workdir,   // mount target (container) → /workspace
    env_map.clone(),
    extra_mounts,
).await?;
```

Also update the `docker exec -it` command to set workdir:

```rust
// Ensure docker exec runs in the right directory
let status = Command::new("docker")
    .args(["exec", "-it", "-w", container_workdir, &container_id])
    .args(&agent_cmd)
    .status();
```

### Acceptance Criteria

- [ ] `create_session()` mounts host path → `/workspace` in container
- [ ] Container workdir is set to `/workspace`
- [ ] `docker exec` uses `-w /workspace` to set working directory
- [ ] Agent can read/write files in the mounted workspace
- [ ] Changes made inside container are visible on host filesystem
- [ ] Pattern matches `run.rs` and UI terminal Docker usage

---

## BF-04: Hardcoded commit message — interactive prompt

**Severity**: Medium
**File(s)**: `crates/ckrv-cli/src/commands/term.rs:1160`
**Estimate**: 25m

### Problem

When user selects "Merge into current branch" from the post-session prompt, the commit message is hardcoded and generic:

```rust
let commit_msg = format!("feat(term): changes from terminal session");
```

This makes `git log` useless for understanding what a session produced. Every merge commit looks identical.

### Fix

Add an interactive commit message prompt in `merge_worktree()`, called after `git add .` but before `git commit`:

```rust
/// Prompt the user for a commit message
fn prompt_commit_message(worktree: &Worktree, agent_name: &str) -> anyhow::Result<String> {
    let auto_msg = format!(
        "feat(term): {} session changes via {}",
        worktree.job_id, agent_name
    );

    let items = [
        "Write custom message...",
        &format!("Use auto-generated: \"{}\"", auto_msg),
    ];

    let choice = Select::with_theme(&ColorfulTheme::default())
        .with_prompt("Commit message")
        .items(&items)
        .default(0)
        .interact()?;

    match choice {
        0 => {
            let msg: String = Input::with_theme(&ColorfulTheme::default())
                .with_prompt("Enter commit message")
                .interact_text()?;
            Ok(msg)
        }
        _ => Ok(auto_msg),
    }
}
```

Update `merge_worktree()` to accept `agent_name` and call the prompt:

```rust
fn merge_worktree(
    worktree: &Worktree,
    cwd: &Path,
    ui: &UiContext,
    agent_name: &str,    // NEW parameter
) -> anyhow::Result<()> {
    // ... git add ...

    if has_staged {
        let commit_msg = prompt_commit_message(worktree, agent_name)?;
        // ... git commit -m &commit_msg ...
    }

    // ... merge ...
}
```

Update `handle_post_session()` to pass the agent name through.

### Acceptance Criteria

- [ ] User is prompted for commit message when choosing "Merge"
- [ ] "Write custom message" opens a text input prompt
- [ ] "Use auto-generated" uses a message with session ID and agent name
- [ ] Auto-generated message format: `feat(term): <session-id> session changes via <agent-name>`
- [ ] `merge_worktree()` signature includes agent name parameter
- [ ] `handle_post_session()` receives and passes agent name

---

## BF-05: Clippy warnings in `term.rs`

**Severity**: Medium
**File(s)**: `crates/ckrv-cli/src/commands/term.rs` (62 warnings)
**Estimate**: 30m

### Problem

62 clippy warnings across `term.rs`, categorized:

| Warning Type | Count | Fix |
|-------------|-------|-----|
| `format!` variable interpolation | 23 | `format!("{variable}")` |
| `&PathBuf` → `&Path` | 4 | Change function signatures |
| `SessionStatus::Foo` → `Self::Foo` | 4 | Use `Self` in impl blocks |
| `&Option<T>` → `Option<&T>` | 3 | Change function signatures |
| Missing doc backticks | 3 | Add backticks in comments |
| Unused variables | 4 | Prefix with `_` or use |
| `async fn` with no await | 2 | Remove `async` (see BF-01) |
| `SESSION_MAX_AGE_SECS` unused | 1 | Use it or remove it |
| `more than 3 bools in struct` | 1 | Known tradeoff — suppress with reason |
| Misc (closures, `unwrap`, etc.) | 17 | Apply individual fixes |

### Fix

Apply fixes systematically, top-to-bottom through the file:

1. **Format strings** (23 fixes): Find-replace `format!("{}", var)` → `format!("{var}")`
2. **Function signatures** (4+3 fixes): Change `&PathBuf` to `&Path`, `&Option<T>` to `Option<&T>`
3. **Self in impl** (4 fixes): Change `SessionStatus::Active` to `Self::Active` etc.
4. **Unused vars** (4 fixes): Prefix with `_` where not used
5. **Remove `SESSION_MAX_AGE_SECS`** if not used, or implement the stale session warning
6. **Bool struct warning**: Add `#[allow(clippy::struct_excessive_bools)]` with a comment explaining this is a CLI args struct

Run after each batch:
```bash
cargo clippy -p ckrv-cli 2>&1 | grep "term\.rs" | grep "warning:" | wc -l
```

### Acceptance Criteria

- [ ] `cargo clippy -p ckrv-cli 2>&1 | grep "term\.rs" | grep "warning:" | wc -l` returns 0 (or only the suppressed bool warning)
- [ ] No `#[allow(...)]` added without a comment explaining why
- [ ] All function signatures use idiomatic Rust types (`&Path`, `Option<&T>`)
- [ ] All format strings use inline variable syntax
- [ ] Build still succeeds: `cargo build -p ckrv-cli`

---

## Verification

After all bugfixes are applied:

- [ ] `cargo build -p ckrv-cli` succeeds with no errors
- [ ] `cargo clippy -p ckrv-cli -- -D warnings` passes (or warnings are pre-existing in other files only)
- [ ] `cargo test -p ckrv-cli` passes
- [ ] `ckrv term --worktree` creates worktree and post-session prompt works
- [ ] `ckrv term --sandbox` creates container with correct `/workspace` mount
- [ ] `ckrv term --sandbox --worktree` combines both modes
- [ ] `ckrv term --name test --worktree` → `ckrv term --resume test` works
- [ ] `ckrv term --name test2 --sandbox` → `ckrv term --resume test2` works (no crash)
- [ ] Merge post-session action prompts for commit message

## Notes

- **BF-03 is the highest priority** — it causes runtime failure for all `--sandbox` usage
- **BF-02 + BF-04** should be done together since both touch `SessionState` and the resume/merge flow
- **BF-05** can be done last as a sweep pass — recommended to run after BF-01 through BF-04 are applied
- **BF-01** is trivial but blocks BF-05 (since it eliminates 2 of the 62 warnings)
