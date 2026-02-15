# Supercharged `ckrv term`: Sandbox + Worktree Mode

**Created**: 2026-02-15
**Status**: Draft

## Problem Statement

Today, `ckrv term` is a thin launcher — it picks an agent, sets env vars, and calls `std::process::Command::new(&binary)`. The agent runs **directly on your main worktree**, with no isolation whatsoever. This means:

1. **No safety net.** A rogue agent (or a careless prompt) can mutate your working directory, break your build, or trash uncommitted work.
2. **No `ckrv run`-style isolation.** The `run` command already creates worktrees + Docker containers per batch — `term` gets none of that, despite being the most interactive command.
3. **No "undo".** If the agent messes things up, you're left doing `git checkout .` or `git stash` and hoping you haven't lost something.
4. **No parallel term sessions.** Because everything shares the main worktree, you can't have two agents working in separate interactive sessions on different tasks without conflicts.

The irony: `ckrv run` (fire-and-forget) has full isolation, but `ckrv term` (interactive, human-in-the-loop) has zero.

## Current State

### `ckrv term` today (in `term.rs`)

```
User → agent selection → option prompts → build_agent_command()
     → std::process::Command::new(binary)
     → cmd.env(key, value)                   // env vars
     → cmd.args(extra_args)                  // passthrough
     → cmd.status()                          // blocking, on main worktree
```

**What it does NOT do:**
- Create a git worktree
- Spin up a Docker container
- Mount credentials via `config_mounts()`
- Provide any rollback mechanism
- Track session state

### Existing infrastructure we can reuse

| Component | Crate | What it does |
|-----------|-------|-------------|
| `DefaultWorktreeManager` | `ckrv-git` | Creates/cleans worktrees under `.chakravarti/worktrees/`, manages branches like `worktree/<branch>/ckrv-<id>` |
| `DockerClient.create_session()` | `ckrv-sandbox` | Creates a long-lived Docker container with `tail -f /dev/null`, mounts workspace + credentials |
| `DockerClient.exec_in_session()` | `ckrv-sandbox` | Runs commands inside a session container with PTY support |
| `DockerClient.stop_session()` | `ckrv-sandbox` | Tears down session container |
| `AgentProvider.config_mounts()` | `ckrv-sandbox` | Returns agent-specific credential mounts (`.claude.json`, `.codex/`, etc.) |
| `AgentProvider.required_env_vars()` | `ckrv-sandbox` | Returns needed env vars per agent |
| `AgentProvider.build_command()` | `ckrv-sandbox` | Constructs the CLI command for an agent |
| `load_agents_config()` | `ckrv-cli` | Loads `~/.config/chakravarti/agents.yaml` |
| `build_agent_command()` | `ckrv-cli (term.rs)` | Resolves binary path + env vars per agent type |

**Key insight:** `create_session()` + `exec_in_session()` already exists for the UI's interactive terminal modal. We just need to expose the same pattern from the CLI.

## Proposed Solution

### Vision Alignment

From `guiding_docs/vision.md`:
- *"Isolation Is Safety"* — Docker sandboxes + git worktrees. **`term` is the only command that violates this principle.**
- *"Git-Native"* — Worktrees are git-native isolation. Agent changes live on a branch, reviewable via `ckrv diff`.
- *"Fire and forget"* — Even interactive sessions should be safe to "let run" without worry.
- *"Not another coding agent — orchestration layer"* — `term` should orchestrate the agent into a safe environment, not just launch a naked process.

### The New `ckrv term` Modes

```
ckrv term                              # Default: direct spawn (current behavior)
ckrv term --sandbox                    # Docker sandbox, main worktree mounted
ckrv term --worktree                   # Isolated git worktree, no Docker
ckrv term --sandbox --worktree         # Docker + worktree (max isolation)
ckrv term --sandbox --worktree --name  # Named session for resume
```

#### Mode Matrix

| Mode | Worktree | Docker | Changes visible on | Rollback |
|------|----------|--------|-------------------|----------|
| `ckrv term` (default) | ❌ | ❌ | Main branch directly | `git checkout .` |
| `--worktree` | ✅ | ❌ | Isolated branch | `git worktree remove` |
| `--sandbox` | ❌ | ✅ | Main branch (mounted) | Stop container |
| `--sandbox --worktree` | ✅ | ✅ | Isolated branch in container | Stop container + remove worktree |

### Detailed Design

#### 1. `--worktree` mode

Creates a temporary worktree, `cd`s the agent into it, and spawns. When the agent exits, offers to merge, diff, or discard.

```rust
// In term.rs execute()
if args.worktree {
    let manager = DefaultWorktreeManager::new(&cwd)?;
    let session_id = format!("term-{}", uuid::Uuid::new_v4().to_string()[..8]);
    let worktree = manager.create(&session_id, "1")?;

    ui.success("Worktree", &format!("Created at {}", worktree.path.display()));
    ui.info("Branch", &worktree.branch);

    // Spawn agent in worktree directory
    cmd.current_dir(&worktree.path);
    let status = cmd.status()?;

    // Post-session: prompt for action
    match post_session_prompt(&cwd, &worktree)? {
        PostAction::Merge => merge_worktree(&cwd, &worktree)?,
        PostAction::Diff  => show_diff(&worktree)?,
        PostAction::Keep  => ui.info("Kept", &format!("Worktree at {}", worktree.path.display())),
        PostAction::Discard => manager.cleanup(&worktree)?,
    }
}
```

**Post-session prompt flow:**

```
Agent exited. What would you like to do?

  ❯ View diff (ckrv diff --worktree)
    Merge into current branch
    Keep worktree for later
    Discard all changes
```

#### 2. `--sandbox` mode

Uses the existing `DockerClient.create_session()` → `exec_in_session()` → `stop_session()` pipeline.

```rust
if args.sandbox {
    let docker = DockerClient::new()?;
    let agent_provider = create_agent(agent_sandbox_type);
    let host_home = std::env::var("HOME").unwrap_or_default();
    let container_home = "/home/claude";

    // Get agent-specific mounts (credentials)
    let agent_mounts = agent_provider.config_mounts(&host_home, container_home);
    let extra_mounts: Vec<BindMount> = agent_mounts.iter().map(|m| {
        BindMount::new(
            m.source.clone().unwrap_or_default(),
            m.target.clone().unwrap_or_default(),
        )
    }).collect();

    // Create session container
    let mount_source = workspace_path.to_string_lossy().to_string();
    let mount_target = "/workspace";
    let container_id = docker.create_session(
        mount_target,
        &mount_source,
        mount_target,
        env_map,
        extra_mounts,
    ).await?;

    ui.success("Sandbox", &format!("Container {}", &container_id[..12]));

    // Exec agent inside container (interactive, PTY)
    let agent_cmd = agent_provider.build_command(
        "", // no prompt for interactive mode
        std::path::Path::new(mount_target),
        &agent_config,
    );
    let result = docker.exec_in_session(&container_id, agent_cmd, env_map).await?;

    // Cleanup
    docker.stop_session(&container_id).await?;
    ui.success("Sandbox", "Container stopped and removed");
}
```

**What this buys you:**
- Agent can't escape the container — filesystem, network, and process isolation
- Credentials mounted read-only via `config_mounts()`
- Container removed on exit — no orphaned processes

#### 3. `--sandbox --worktree` mode (maximum isolation)

Combines both: creates a worktree, then mounts that worktree's path into a Docker container.

```rust
if args.sandbox && args.worktree {
    // 1. Create worktree
    let worktree = manager.create(&session_id, "1")?;

    // 2. Create Docker session mounting worktree (not main repo)
    let mount_source = worktree.path.to_string_lossy().to_string();
    let container_id = docker.create_session(
        "/workspace",
        &mount_source,
        "/workspace",
        env_map,
        extra_mounts,
    ).await?;

    // 3. Exec agent in container
    // ... same as sandbox mode ...

    // 4. Cleanup: stop container, then prompt for worktree action
    docker.stop_session(&container_id).await?;
    post_session_prompt(&cwd, &worktree)?;
}
```

**Why this is the ultimate mode:**
- Agent changes land on an isolated branch (worktree)
- Agent process is containerized (sandbox)
- Main branch is completely untouchable
- You review via `ckrv diff`, merge via `ckrv promote` or the post-session prompt

#### 4. Named sessions with `--name`

```bash
ckrv term --sandbox --worktree --name "fix-auth-bug"
```

Creates a named worktree (`term-fix-auth-bug`) and tagged container (`ckrv-term-fix-auth-bug`). Benefits:

- **Resume:** `ckrv term --sandbox --resume fix-auth-bug` reconnects to the existing container/worktree
- **List:** `ckrv term --list-sessions` shows active term sessions
- **Cleanup:** `ckrv term --cleanup fix-auth-bug` removes container and optionally worktree

Session state stored in `.chakravarti/sessions/`:
```yaml
# .chakravarti/sessions/fix-auth-bug.yaml
name: fix-auth-bug
agent: claude-native
container_id: "abc123def456"
worktree_path: "/repo/.chakravarti/worktrees/term-fix-auth-bug_1"
worktree_branch: "worktree/main/ckrv-term-fix-auth-bug"
created_at: "2026-02-15T20:54:00Z"
status: active
```

## User Stories

### US1: Safe Interactive Exploration
**As a** ckrv developer,
**I want** to run `ckrv term --worktree` to explore changes in an isolated branch,
**So that** my main working directory stays clean even if the agent goes off-script.

### US2: Sandboxed Agent Sessions
**As a** ckrv developer working with untrusted or experimental agents,
**I want** to run `ckrv term --sandbox` to contain the agent in Docker,
**So that** I have process + filesystem isolation without managing containers manually.

### US3: Maximum Isolation for Complex Work
**As a** ckrv developer working on a risky refactor,
**I want** to run `ckrv term --sandbox --worktree` for full Docker + worktree isolation,
**So that** even if the agent crashes or produces bad code, my main branch and host system are untouched.

### US4: Resumable Named Sessions
**As a** ckrv developer working on a multi-session task,
**I want** to name my sessions and resume them later,
**So that** I can step away and come back to exactly where I left off.

### US5: Parallel Term Sessions
**As a** ckrv developer,
**I want** to run multiple `ckrv term --worktree` sessions simultaneously on different tasks,
**So that** I can have one agent brainstorming while another implements, without conflicts.

## Technical Approach

### Options Considered

| Option | Pros | Cons |
|--------|------|------|
| A: Add flags to `term.rs` directly | Simple, all in one file | `term.rs` grows large, mixing concerns |
| B: Extract into `TermSession` struct | Clean separation, testable | More files, slight over-engineering for v1 |
| C: New `ckrv sandbox` command | Clear mental model | Fragments UX, two commands to learn |

### Decision

**Option A for v1, refactor to B when it grows.** The flags add naturally to the existing `TermArgs` struct, and the `execute()` function already has clear sections. If/when `term.rs` exceeds 700 lines, extract a `TermSession` struct.

### Implementation Plan

#### New CLI Arguments in `TermArgs`

```rust
pub struct TermArgs {
    // ... existing fields ...

    /// Run agent in a Docker sandbox
    #[arg(long)]
    sandbox: bool,

    /// Run agent in an isolated git worktree
    #[arg(long)]
    worktree: bool,

    /// Name the session for resume capability
    #[arg(long)]
    name: Option<String>,

    /// Resume a named session
    #[arg(long, conflicts_with = "agent")]
    resume: Option<String>,

    /// List active term sessions
    #[arg(long)]
    list_sessions: bool,

    /// Clean up a named session
    #[arg(long)]
    cleanup: Option<String>,
}
```

#### Files to Modify

| File | Changes |
|------|---------|
| `crates/ckrv-cli/src/commands/term.rs` | Add `--sandbox`, `--worktree`, `--name`, `--resume` flags; add worktree creation/cleanup logic; add Docker session lifecycle; add post-session prompt |
| `crates/ckrv-sandbox/src/agent/mod.rs` | No changes needed — `AgentType` mapping already works between CLI and sandbox crate via `AgentType::from_str()` |
| `crates/ckrv-sandbox/src/docker.rs` | No changes needed — `create_session()`, `exec_in_session()`, `stop_session()` already exist |
| `crates/ckrv-git/src/worktree.rs` | No changes needed — `DefaultWorktreeManager` already handles creation/cleanup |
| `crates/ckrv-cli/src/commands/term.rs` | New `PostAction` enum and `post_session_prompt()` function |

#### Dependency Graph (what calls what)

```
term.rs::execute()
  ├── load_agents_config()                    # existing (agent_lookup.rs)
  ├── build_agent_command()                   # existing (term.rs)
  │
  ├── [if --worktree]
  │   ├── DefaultWorktreeManager::new()       # existing (ckrv-git)
  │   ├── manager.create(session_id, "1")     # existing (ckrv-git)
  │   ├── cmd.current_dir(worktree.path)      # new usage
  │   └── post_session_prompt()               # NEW
  │       ├── merge_worktree()                # NEW (reuses git CLI)
  │       └── manager.cleanup()               # existing (ckrv-git)
  │
  ├── [if --sandbox]
  │   ├── DockerClient::new()                 # existing (ckrv-sandbox)
  │   ├── create_agent(agent_type)            # existing (ckrv-sandbox)
  │   ├── agent.config_mounts()               # existing (ckrv-sandbox)
  │   ├── docker.create_session()             # existing (ckrv-sandbox)
  │   ├── docker.exec_in_session()            # existing (ckrv-sandbox)
  │   └── docker.stop_session()               # existing (ckrv-sandbox)
  │
  └── [if --sandbox --worktree]
      ├── Creates worktree FIRST
      ├── Mounts worktree path into Docker
      └── On exit: stops container, then prompts for worktree action
```

### Key Technical Challenges

#### 1. PTY Forwarding with Docker

`exec_in_session()` already has `tty: Some(true)` in its `CreateExecOptions`. But the current implementation collects output into a String and returns it — it doesn't forward the PTY to the host terminal. 

**For interactive `ckrv term --sandbox`**, we need the container's PTY connected to the user's real terminal. Options:

| Approach | Effort | Quality |
|----------|--------|---------|
| A: Use `docker exec -it` via `std::process::Command` | Low | Works immediately, well-tested |
| B: Extend bollard exec to pipe stdin/stdout | High | More control, but complex |
| C: Use the Tauri PTY approach from the UI | Medium | Works for web terminal, not CLI |

**Decision:** Option A for v1. Shell out to `docker exec -it <container_id> <agent_cmd>`. The bollard API is better for batch execution; for interactive term sessions, `docker exec -it` is battle-tested.

```rust
// Instead of docker.exec_in_session(), use native docker exec
let mut exec_cmd = Command::new("docker");
exec_cmd.args(["exec", "-it", &container_id]);
exec_cmd.args(&agent_cmd);
let status = exec_cmd.status()?;
```

#### 2. Agent Type Mapping (CLI ↔ Sandbox)

`term.rs` uses `crate::services::agent_lookup::AgentType` (5 variants: Claude, ClaudeOpenRouter, ClaudeGlm, Codex, KiloCode).

`ckrv-sandbox` uses `ckrv_sandbox::agent::AgentType` (3 variants: Claude, Codex, KiloCode).

For `--sandbox` mode, we need to map CLI agent types to sandbox agent types. ClaudeOpenRouter and ClaudeGlm map to Claude (same binary, different env vars).

```rust
fn to_sandbox_agent_type(cli_type: &crate::services::agent_lookup::AgentType) -> ckrv_sandbox::AgentType {
    match cli_type {
        AgentType::Claude | AgentType::ClaudeOpenRouter | AgentType::ClaudeGlm => ckrv_sandbox::AgentType::Claude,
        AgentType::Codex => ckrv_sandbox::AgentType::Codex,
        AgentType::KiloCode => ckrv_sandbox::AgentType::KiloCode,
    }
}
```

#### 3. Merging Env Vars

`build_agent_command()` returns env vars for OpenRouter/GLM (base URL, API key, model overrides). These need to be passed into the Docker session via the `env` HashMap. This is just plumbing — collect them from `build_agent_command()` and pass to `create_session()`.

#### 4. Signal Handling for Cleanup

If the user presses Ctrl+C during a sandbox session, we need to clean up the container and optionally the worktree. Use `ctrlc` crate or `tokio::signal` to register a cleanup handler.

```rust
let container_id_clone = container_id.clone();
tokio::spawn(async move {
    tokio::signal::ctrl_c().await.ok();
    let docker = DockerClient::new().ok();
    if let Some(d) = docker {
        d.stop_session(&container_id_clone).await.ok();
    }
});
```

## Implementation Notes

### What's 100% reusable (zero new code in these crates)

- **`ckrv-git`**: `DefaultWorktreeManager::new()`, `.create()`, `.cleanup()`, `.list()` — all work as-is for term sessions, just need a `term-` prefixed `job_id` instead of `batch-`
- **`ckrv-sandbox`**: `DockerClient::new()`, `.create_session()`, `.stop_session()` — pre-existing session lifecycle
- **`ckrv-sandbox::agent`**: `create_agent()`, `AgentProvider::config_mounts()`, `.required_env_vars()` — credential mounting already solved
- **`ckrv-cli::services::agent_lookup`**: `load_agents_config()`, agent type resolution

### New code lives entirely in `term.rs`

- `PostAction` enum + `post_session_prompt()` (~50 lines)
- `merge_worktree()` helper (~30 lines)
- `show_diff()` helper (~15 lines)
- Worktree lifecycle in `execute()` (~40 lines)
- Docker session lifecycle in `execute()` (~60 lines)
- Session state save/load for `--name` (~80 lines)
- **Total: ~275 lines of new code**, all in `term.rs`

### Backwards Compatibility

- `ckrv term` without flags behaves **exactly** as today — no breaking changes
- All new behavior is opt-in via `--sandbox` and `--worktree` flags
- Session state in `.chakravarti/sessions/` doesn't affect any existing state

## Open Questions

- [ ] Should `--sandbox` be the default eventually? The vision says "isolation through architecture" — but forcing Docker for every `ckrv term` would be a friction increase.
- [ ] Should the post-session prompt default to «View diff» or «Merge»? Power users might prefer auto-merge; new users want to review first.
- [ ] How should `--resume` work when the container has been stopped? Restart the container, or create a new one with the same worktree?
- [ ] Should `ckrv term --worktree` use the `AgentProvider.build_command()` from the sandbox crate, or continue using the CLI-side `build_agent_command()`? They do similar things but with different abstractions.
- [ ] Should session state files be gitignored? (Probably yes, they contain container IDs.)

## Success Criteria

| Metric | Target |
|--------|--------|
| Worktree isolation | `ckrv term --worktree` creates branch, agent works on it, changes don't touch main |
| Docker isolation | `ckrv term --sandbox` runs agent in container with proper credential mounts |
| Combined mode | `--sandbox --worktree` provides both isolation layers simultaneously |
| Post-session UX | User can view diff, merge, keep, or discard — intuitive prompt |
| No regressions | `ckrv term` (no flags) works exactly as before |
| Cleanup reliability | Ctrl+C always cleans up containers and optionally worktrees |
| Parallel sessions | Two simultaneous `--worktree` sessions don't conflict |

## Next Steps

- [ ] Prototype `--worktree` mode first (no Docker dependency, faster feedback loop)
- [ ] Add post-session prompt with diff/merge/keep/discard flow
- [ ] Add `--sandbox` mode using `docker exec -it` shellout
- [ ] Combine `--sandbox --worktree` mode
- [ ] Add `--name` session persistence
- [ ] Test with all agent types (Claude, Codex, Kilo)
- [ ] Document in `long_about` / `after_help` attributes

## References

- `crates/ckrv-cli/src/commands/term.rs` — Current term implementation (497 lines)
- `crates/ckrv-git/src/worktree.rs` — `DefaultWorktreeManager`, `Worktree` struct, branch naming
- `crates/ckrv-sandbox/src/docker.rs` — `create_session()`, `exec_in_session()`, `stop_session()`
- `crates/ckrv-sandbox/src/agent/mod.rs` — `AgentProvider` trait, `config_mounts()`, `create_agent()`
- `crates/ckrv-sandbox/src/executor.rs` — `ExecuteConfig`, `BindMount`, `DockerSandbox`
- `crates/ckrv-cli/src/services/agent_lookup.rs` — `load_agents_config()`, `AgentConfig`, `AgentType` (CLI-side)
- `crates/ckrv-cli/src/commands/run.rs` — How `run` creates worktrees + uses Docker (reference pattern)
- `guiding_docs/vision.md` — "Isolation Is Safety", "Git-Native", "Fire and Forget"
- `brainstorming/dogfooding-ckrv-on-ckrv/notes.md` — Self-development workflow context
