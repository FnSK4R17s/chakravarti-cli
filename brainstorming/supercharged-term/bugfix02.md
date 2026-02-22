# Supercharged `ckrv term` — Bugfix Tasks (02)

**Brainstorm**: [notes.md](./notes.md)
**Created**: 2026-02-16
**Source**: Manual QA — interactive options prompt missing term-level flags

## Bugfix Overview

| # | Bug | Severity | Estimate |
|---|-----|----------|----------|
| BF-06 | Interactive options prompt missing term-level flags | High | 30m |

**Severity breakdown**: 1 High
**Total estimate**: ~30m

---

## BF-06: Interactive options prompt missing term-level flags

**Severity**: High
**File(s)**: `crates/ckrv-cli/src/commands/term.rs:1290-1356`
**Estimate**: 30m

### Problem

When a user runs `ckrv term` interactively and selects "Configure options...", the multi-select prompt only shows **agent-specific flags** (e.g., `--dangerously-skip-permissions`, `--continue`, `--verbose`). The **term-level isolation flags** (`--worktree`, `--sandbox`, `--name`) are completely absent from the interactive UI.

Current interactive flow:

```
✔ Select an agent to spawn · Claude Code (claude-default) [claude] ★
✔ Launch options · Configure options...
? Select options (Space to toggle, Enter to confirm) ›
⬚ Skip permissions - Skip all permission prompts (dangerous!)
⬚ Continue session - Resume the most recent conversation
⬚ Agent teams - Enable experimental agent teams mode
⬚ Verbose output - Enable verbose logging
⬚ JSON output - Output in JSON format
```

**Missing from the prompt:**
- Worktree isolation (`--worktree`)
- Docker sandbox (`--sandbox`)
- Name session (`--name`)

These flags are only discoverable via `ckrv term --help` or prior knowledge. Users who launch `ckrv term` interactively (which is the default workflow) have no way to discover or enable isolation modes without restarting with CLI flags.

### Root Cause

The `prompt_for_options()` function (line 1297) only iterates over `COMMON_OPTIONS` (line 85-134), which contains agent-specific CLI flags. The term-level flags (`--worktree`, `--sandbox`, `--name`) are defined in `TermArgs` (line 232-272) but are never surfaced in the interactive prompt.

The `collect_args_and_env()` function (line 740) returns only `(Vec<String>, Vec<(String, String)>)` — agent args and env vars. There is no mechanism to return term-level selections back to `execute()`.

```rust
// Current — returns only agent-specific options
fn collect_args_and_env(...) -> anyhow::Result<(Vec<String>, Vec<(String, String)>)> { ... }

// Current — only agent flags shown
fn prompt_for_options(agent_type: &AgentType) -> anyhow::Result<PromptResult> {
    // Only iterates COMMON_OPTIONS (agent flags)
    // No term-level options presented
}
```

### Fix

**Part A**: Expand `PromptResult` to carry term-level selections:

```rust
/// Result of interactive options prompt
struct PromptResult {
    args: Vec<String>,
    env_vars: Vec<(String, String)>,
    /// Whether the user selected worktree isolation mode interactively
    worktree: bool,
    /// Whether the user selected sandbox isolation mode interactively
    sandbox: bool,
    /// Session name if provided interactively
    session_name: Option<String>,
}
```

**Part B**: Update `prompt_for_options()` to present term-level options as a separate multi-select BEFORE agent-specific options:

```rust
fn prompt_for_options(agent_type: &AgentType) -> anyhow::Result<PromptResult> {
    let theme = ColorfulTheme::default();
    let mut args: Vec<String> = Vec::new();
    let mut env_vars: Vec<(String, String)> = Vec::new();
    let mut worktree = false;
    let mut sandbox = false;
    let mut session_name: Option<String> = None;

    // ... existing applicable filter + launch_choice prompt ...

    if launch_choice == 0 {
        return Ok(PromptResult { args, env_vars, worktree, sandbox, session_name });
    }

    // -- NEW: Term-level isolation options (always shown) --
    let term_options = [
        "Worktree isolation - Run agent in an isolated git worktree branch",
        "Docker sandbox - Run agent inside a Docker container",
        "Name session - Create a named session for resume later",
    ];

    let term_selections = MultiSelect::with_theme(&theme)
        .with_prompt("Isolation modes (Space to toggle, Enter to confirm)")
        .items(&term_options)
        .interact()?;

    for idx in &term_selections {
        match idx {
            0 => worktree = true,
            1 => sandbox = true,
            2 => {
                let name: String = Input::with_theme(&theme)
                    .with_prompt("Session name")
                    .interact_text()?;
                if !name.trim().is_empty() {
                    session_name = Some(name.trim().to_string());
                }
            }
            _ => {}
        }
    }

    // -- Existing: Agent-specific options --
    if !applicable.is_empty() {
        let items: Vec<String> = applicable
            .iter()
            .map(|opt| format!("{} - {}", opt.label, opt.description))
            .collect();

        let selections = MultiSelect::with_theme(&theme)
            .with_prompt("Agent options (Space to toggle, Enter to confirm)")
            .items(&items)
            .interact()?;

        for idx in selections {
            // ... existing flag/env_var matching ...
        }
    }

    // ... existing custom args prompt ...

    Ok(PromptResult { args, env_vars, worktree, sandbox, session_name })
}
```

**Part C**: Update `collect_args_and_env()` to return `PromptResult` instead of a tuple:

```rust
// Before
fn collect_args_and_env(...) -> anyhow::Result<(Vec<String>, Vec<(String, String)>)> { ... }

// After
fn collect_args_and_env(...) -> anyhow::Result<PromptResult> {
    if !args.passthrough_args.is_empty() {
        return Ok(PromptResult {
            args: args.passthrough_args.clone(),
            env_vars: Vec::new(),
            worktree: false,
            sandbox: false,
            session_name: None,
        });
    }

    if json {
        Ok(PromptResult {
            args: Vec::new(),
            env_vars: Vec::new(),
            worktree: false,
            sandbox: false,
            session_name: None,
        })
    } else {
        prompt_for_options(agent_type)
    }
}
```

**Part D**: Update `execute()` to apply interactively-selected term-level flags:

```rust
// Make args mutable
pub async fn execute(mut args: TermArgs, json: bool, ui: &UiContext) -> anyhow::Result<()> {
    // ...

    // Collect extra arguments and env vars (includes interactively-selected term options)
    let prompt_result = collect_args_and_env(&args, &agent.agent_type, json)?;
    let extra_args = prompt_result.args;

    // Apply interactively-selected term-level options to args
    if prompt_result.worktree {
        args.worktree = true;
    }
    if prompt_result.sandbox {
        args.sandbox = true;
    }
    if prompt_result.session_name.is_some() && args.name.is_none() {
        args.name = prompt_result.session_name;
    }

    // Build command based on agent type
    let (binary, mut env_vars) = build_agent_command(&agent)?;
    env_vars.extend(prompt_result.env_vars);

    // ... rest unchanged ...
}
```

**Part E**: Update `resume_session()` which also calls `collect_args_and_env()`:

```rust
// Before (line 658)
let (extra_args, _) = collect_args_and_env(_args, &agent.agent_type, json)?;

// After
let prompt_result = collect_args_and_env(_args, &agent.agent_type, json)?;
let extra_args = prompt_result.args;
```

### Interactive Flow After Fix

```
✔ Select an agent to spawn · Claude Code (claude-default) [claude] ★
✔ Launch options · Configure options...
? Isolation modes (Space to toggle, Enter to confirm) ›
⬚ Worktree isolation - Run agent in an isolated git worktree branch
⬚ Docker sandbox - Run agent inside a Docker container
⬚ Name session - Create a named session for resume later

? Agent options (Space to toggle, Enter to confirm) ›
⬚ Skip permissions - Skip all permission prompts (dangerous!)
⬚ Continue session - Resume the most recent conversation
⬚ Agent teams - Enable experimental agent teams mode
⬚ Verbose output - Enable verbose logging
⬚ JSON output - Output in JSON format
```

### Design Decisions

1. **Two separate multi-selects** rather than one combined list — isolation modes are conceptually different from agent CLI flags. Grouping them separately makes the UX clearer and avoids confusion about what each option does.

2. **"Isolation modes" prompt always shown** regardless of agent type — worktree and sandbox isolation work with all agents, unlike agent-specific flags which vary by agent.

3. **Session name uses a follow-up text input** — when the user selects "Name session", an `Input` prompt immediately asks for the name. This avoids making users type into a multi-select.

4. **CLI flags take precedence** — if the user already passed `--worktree` on the command line, the interactive prompt won't override it. The `if prompt_result.worktree { args.worktree = true; }` pattern only sets `true`, never `false`.

5. **No isolation modes prompt when "Launch directly" is selected** — the "Launch directly" path skips all configuration, same as today. Only "Configure options..." shows the new prompt.

6. **Removed `applicable.is_empty()` early return** — previously, if an agent had zero applicable options (unlikely but possible), the prompt was skipped entirely. Now the isolation modes prompt is always shown, and the agent options prompt is conditionally shown only when there are applicable options.

### Acceptance Criteria

- [ ] Running `ckrv term` → "Configure options..." shows "Isolation modes" multi-select with worktree, sandbox, and name session options
- [ ] Selecting "Worktree isolation" causes the agent to spawn in an isolated worktree
- [ ] Selecting "Docker sandbox" causes the agent to spawn in a Docker container
- [ ] Selecting "Name session" prompts for a session name and creates a named session
- [ ] Selecting multiple isolation modes (worktree + sandbox) works correctly
- [ ] "Launch directly" skips all prompts (no regression)
- [ ] CLI flags (`--worktree`, `--sandbox`, `--name`) still work when passed directly
- [ ] CLI flags take precedence over interactive selections (no conflict)
- [ ] `PromptResult` struct has `worktree`, `sandbox`, and `session_name` fields
- [ ] `collect_args_and_env()` returns `PromptResult` (not a tuple)
- [ ] `execute()` accepts `mut args: TermArgs` and applies interactive selections
- [ ] `resume_session()` updated to use new `PromptResult` return type
- [ ] Agent-specific options still appear in a separate "Agent options" prompt
- [ ] `cargo build -p ckrv-cli` succeeds
- [ ] `cargo clippy -p ckrv-cli -- -D warnings` passes (or warnings pre-existing in other files only)

---

## Verification

After this bugfix is applied:

- [ ] `cargo build -p ckrv-cli` succeeds
- [ ] `cargo clippy -p ckrv-cli -- -D warnings` passes
- [ ] `ckrv term` → "Configure options..." → shows isolation modes + agent options
- [ ] `ckrv term --worktree` still works (CLI flag path unchanged)
- [ ] `ckrv term --sandbox` still works (CLI flag path unchanged)
- [ ] `ckrv term` → select worktree interactively → agent spawns in worktree
- [ ] `ckrv term` → select sandbox interactively → agent spawns in sandbox
- [ ] `ckrv term` → select name session interactively → session state file created

## Notes

- This bugfix touches 5 code locations in `term.rs` but the changes are mechanical
- No cross-crate impact — all changes are within `ckrv-cli`
- No new dependencies needed — uses existing `dialoguer` types (`MultiSelect`, `Input`)
- Backwards compatible — `ckrv term` without "Configure options..." behaves identically to before
