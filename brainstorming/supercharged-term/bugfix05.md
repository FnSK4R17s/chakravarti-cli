# Supercharged `ckrv term` — Bugfix Tasks (05)

**Brainstorm**: [notes.md](./notes.md)
**Created**: 2026-02-16
**Source**: Manual QA — sessions not persisted, `--resume` requires explicit name

## Bugfix Overview

| # | Bug | Severity | Estimate |
|---|-----|----------|----------|
| BF-13 | Sessions not persisted unless `--name` is explicitly given | High | 25m |
| BF-14 | `--resume` requires a value — should list sessions if omitted | Medium | 15m |
| BF-15 | Session name not printed after agent exits | Medium | 5m |

**Severity breakdown**: 1 High, 2 Medium
**Total estimate**: ~45m

---

## BF-13: Sessions not persisted unless `--name` is explicitly given

**Severity**: High
**File(s)**: `crates/ckrv-cli/src/commands/term.rs:362,376-386,782-790`
**Estimate**: 25m

### Problem

Session state is only created when `--name` is explicitly passed (line 376-386):

```rust
// Line 376-386 — only saves if user provided --name
if let Some(session_name) = &args.name {
    create_session_state(
        session_name,
        &agent.id,
        worktree_info.as_ref(),
        &cwd,
        args.sandbox,
        args.worktree,
    )?;
}
```

When `--name` is NOT provided, `generate_session_id()` creates a `term-<uuid>` ID (line 782-790), but this is only used for worktree branch naming — no session state file is persisted. So `--list-sessions` shows nothing, and `--resume` is unusable.

### Fix

**Part A**: Auto-generate a memorable, Reddit-style session name when `--name` is not provided.

Add a `generate_session_name()` function that creates terminal-safe names like `brave-panda-4821`, `swift-falcon-0137`, `calm-tiger-9502`:

```rust
/// Generate a memorable, terminal-safe session name.
/// Format: adjective-animal-NNNN (e.g., "brave-panda-4821", "swift-falcon-0137")
/// The 4-digit suffix prevents collisions across concurrent or rapid sessions.
fn generate_session_name() -> String {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};

    const ADJECTIVES: &[&str] = &[
        "bold", "brave", "calm", "cool", "crisp", "deft", "fair", "fast",
        "fine", "firm", "fond", "free", "glad", "gold", "good", "holy",
        "keen", "kind", "lean", "live", "neat", "nice", "pure", "rare",
        "rich", "safe", "sage", "slim", "soft", "sure", "tall", "tidy",
        "true", "vast", "warm", "wide", "wild", "wise", "zany", "epic",
        "swift",
    ];

    // Animals native to the Indian subcontinent 🇮🇳
    const ANIMALS: &[&str] = &[
        "ape", "bat", "bear", "bison", "boar", "bull", "civet", "cobra",
        "crane", "crow", "deer", "dove", "eagle", "elephant", "fox",
        "frog", "gaur", "gecko", "goat", "hawk", "hare", "heron", "ibis",
        "jackal", "kite", "koel", "langur", "lion", "moth", "mongoose",
        "myna", "newt", "otter", "owl", "panda", "peacock", "rat",
        "rhino", "robin", "shrew", "stork", "tiger", "viper", "wolf",
    ];

    let mut hasher = DefaultHasher::new();
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos()
        .hash(&mut hasher);
    let hash = hasher.finish();

    let adj = ADJECTIVES[(hash as usize) % ADJECTIVES.len()];
    let animal = ANIMALS[((hash >> 16) as usize) % ANIMALS.len()];
    let suffix = (hash >> 32) % 10000; // 4-digit numeric suffix

    // A cow can only be holy 🐄
    let (adj, animal) = if adj == "holy" { ("holy", "cow") } else { (adj, animal) };

    format!("{adj}-{animal}-{suffix:04}")
}
```

**Part B**: Always assign a session name and always persist session state.

Update `execute()` to auto-generate a name when none is provided:

```rust
// Generate session name — either user-provided or auto-generated
if args.name.is_none() {
    args.name = Some(generate_session_name());
}
let session_name = args.name.as_ref().unwrap(); // Safe — we just ensured it's Some

// Generate session ID (used for worktree branch naming)
let session_id = generate_session_id(Some(session_name));

// ... worktree creation, working_dir setup ...

// Always create session state (no longer gated by `if let Some(session_name)`)
create_session_state(
    session_name,
    &agent.id,
    worktree_info.as_ref(),
    &cwd,
    args.sandbox,
    args.worktree,
)?;
```

**Part C**: Remove the `if let Some(session_name)` gate on session persistence (line 376-386). Session state should ALWAYS be saved.

Similarly, update the post-agent status update (line 422-425):

```rust
// Before: only updates if named
if let Some(session_name) = &args.name {
    update_session_status(session_name, SessionStatus::Stopped)?;
}

// After: always updates (args.name is always Some now)
let session_name = args.name.as_ref().unwrap();
update_session_status(session_name, SessionStatus::Stopped)?;
```

### Acceptance Criteria

- [x] Every `ckrv term` session auto-generates a name when `--name` is omitted
- [x] Auto-generated names are Reddit-style: `adjective-animal-NNNN` (e.g., `brave-panda-4821`)
- [x] Names are terminal-safe: lowercase, hyphen-separated, no special chars
- [x] Session state is always persisted to `.chakravarti/sessions/<name>.yaml`
- [x] `ckrv term --list-sessions` shows all sessions (including auto-named ones)
- [x] `--name` still works for explicit naming (overrides auto-generation)
- [x] Session status is updated to `stopped` when agent exits

---

## BF-14: `--resume` requires a value — should list sessions if omitted

**Severity**: Medium
**File(s)**: `crates/ckrv-cli/src/commands/term.rs:253-255`
**Estimate**: 15m

### Problem

```bash
$ ckrv term --resume
error: a value is required for '--resume <RESUME>' but none was supplied
```

`--resume` is defined with `Option<String>` which requires a value (line 253-255):

```rust
/// Resume a named session
#[arg(long)]
resume: Option<String>,
```

When `--resume` is passed without a value, clap errors because it expects a string argument.

### Fix

Change `--resume` to accept an optional value. If no value is given, show the session list and prompt the user to select one:

```rust
/// Resume a session. Optionally pass a session name, or omit to select interactively.
#[arg(long, num_args = 0..=1, default_missing_value = "")]
resume: Option<String>,
```

Then in `execute()`, handle the empty-string case:

```rust
if let Some(session_name) = &args.resume {
    if session_name.is_empty() {
        // No name given — show interactive session picker
        let sessions = get_all_sessions()?;
        if sessions.is_empty() {
            ui.info("No Sessions", "No sessions available to resume.");
            return Ok(());
        }

        let items: Vec<String> = sessions
            .iter()
            .map(|(name, state)| {
                let age = format_age(state.created_at);
                format!("{} [{}] {} — {}", name, state.status, state.agent_id, age)
            })
            .collect();

        let selection = Select::with_theme(&ColorfulTheme::default())
            .with_prompt("Select a session to resume")
            .items(&items)
            .interact()?;

        let selected_name = &sessions[selection].0;
        return resume_session(&args, selected_name, json, ui, &cwd).await;
    }

    return resume_session(&args, session_name, json, ui, &cwd).await;
}
```

This requires a helper to load all sessions:

```rust
fn get_all_sessions() -> anyhow::Result<Vec<(String, SessionState)>> {
    let sessions_dir = get_sessions_dir()?;
    if !sessions_dir.exists() {
        return Ok(Vec::new());
    }

    let mut sessions = Vec::new();
    for entry in std::fs::read_dir(&sessions_dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.extension().is_some_and(|e| e == "yaml") {
            if let Ok(content) = std::fs::read_to_string(&path) {
                if let Ok(state) = serde_yaml::from_str::<SessionState>(&content) {
                    if let Some(name) = path.file_stem().and_then(|s| s.to_str()) {
                        sessions.push((name.to_string(), state));
                    }
                }
            }
        }
    }
    sessions.sort_by(|a, b| b.1.created_at.cmp(&a.1.created_at));
    Ok(sessions)
}
```

> **Note**: This `get_all_sessions()` helper can also be used to refactor `list_sessions()` which has the same logic duplicated.

### Acceptance Criteria

- [x] `ckrv term --resume` (no value) shows interactive session picker
- [x] `ckrv term --resume brave-panda` still works with explicit name
- [x] Session picker shows name, status, agent, and age
- [x] "No sessions" message shown when no sessions exist
- [x] `list_sessions()` refactored to use shared `get_all_sessions()` helper

---

## BF-15: Session name not printed after agent exits

**Severity**: Medium
**File(s)**: `crates/ckrv-cli/src/commands/term.rs:417-425`
**Estimate**: 5m

### Problem

After the agent exits, ckrv shows "No Changes" or the post-session diff menu — but never tells the user the session name. If they want to resume later, they'd need to guess the name or run `--list-sessions`.

### Fix

Print the session name and a hint about `--resume` after the agent exits:

```rust
// After agent exits, print session info
if !json {
    let session_name = args.name.as_ref().unwrap();
    println!();
    ui.info(
        "Session",
        &format!(
            "\"{}\" — resume with: ckrv term --resume {}",
            session_name, session_name
        ),
    );
}

// Handle post-session for worktree mode
if let Some(ref wt) = worktree_info {
    handle_post_session(wt, &cwd, ui, &agent.name)?;
}
```

### Expected Output

```
▌ ✔ Container Started
▌ Container ID: 90a1f24e13a2...

 ▐▛███▜▌   Claude Code v2.1.29
▝▜█████▛▘  Opus 4.5 · Claude Pro
  ▘▘ ▝▝    /workspace

▌ ℹ Session
▌ "brave-panda-4821" — resume with: ckrv term --resume brave-panda-4821

▌ ✔ No Changes
▌ Agent made no changes to the worktree.
```

### Acceptance Criteria

- [x] Session name is printed after agent exits (before post-session handling)
- [x] Resume command hint is shown alongside the session name
- [x] Not shown when `--json` is passed

---

## Verification

After all bugfixes are applied:

- [x] `ckrv term` → auto-generates session name → session persisted
- [x] `ckrv term --list-sessions` → shows auto-named sessions
- [x] `ckrv term --resume` → shows interactive session picker
- [x] `ckrv term --resume brave-panda-4821` → resumes specific session
- [x] `ckrv term --name my-session` → uses explicit name (override)
- [x] Session name printed after agent exits with resume hint
- [x] `ckrv term --cleanup brave-panda-4821` → removes the session
- [x] `cargo build -p ckrv-cli` succeeds

## Notes

- Auto-generated names use a time-based hash, not true randomness — collision chance is negligible for human-interactive sessions
- The word lists are intentionally small (~40-50 words each) to keep names short and memorable
- Names are all lowercase with hyphens — safe for filenames, URLs, and terminal usage
- `get_all_sessions()` helper deduplicates logic currently in `list_sessions()`
