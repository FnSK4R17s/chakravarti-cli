# Supercharged `ckrv term` — Tasks

**Brainstorm**: [notes.md](./notes.md)
**Created**: 2026-02-15

## Task Overview

| Phase | Tasks | Estimate |
|-------|-------|----------|
| Phase 1: Worktree Mode | 4 tasks | ~3h |
| Phase 2: Sandbox Mode | 4 tasks | ~3h |
| Phase 3: Combined Mode + Sessions | 4 tasks | ~3h |
| Phase 4: Polish & Docs | 3 tasks | ~2h |
| **Total** | **15 tasks** | **~11h** |

---

## Phase 1: Worktree Mode (`--worktree`)

*Goal: Agent runs in an isolated git worktree, changes live on a branch, post-session prompt lets user merge/diff/keep/discard.*

### Task 1.1: Add `--worktree` flag to `TermArgs`
**Priority**: P0
**Estimate**: 20m
**Complexity**: 2
**File**: `crates/ckrv-cli/src/commands/term.rs`

Add the `--worktree` boolean flag to the `TermArgs` struct with clap attributes. Update `long_about` and `after_help` to document the new flag with examples.

```rust
/// Run agent in an isolated git worktree
#[arg(long)]
worktree: bool,
```

**Acceptance Criteria**:
- [ ] `ckrv term --help` shows `--worktree` flag with description
- [ ] `ckrv term` without `--worktree` behaves exactly as before (no regression)
- [ ] `long_about` and `after_help` updated with worktree examples

---

### Task 1.2: Worktree creation and agent spawn
**Priority**: P0
**Estimate**: 1h
**Complexity**: 3
**File**: `crates/ckrv-cli/src/commands/term.rs`
**Depends on**: Task 1.1

When `--worktree` is set, use `DefaultWorktreeManager` from `ckrv-git` to create an isolated worktree before spawning the agent. Set the agent's working directory to the worktree path.

- Import `ckrv_git::{DefaultWorktreeManager, WorktreeManager}` in `term.rs`
- Generate a session ID: `term-{uuid_short}` (e.g., `term-a1b2c3d4`)
- Call `manager.create(&session_id, "1")` to create worktree at `.chakravarti/worktrees/term-{id}_1`
- Set `cmd.current_dir(&worktree.path)` before `cmd.status()`
- Display worktree path and branch name to user via `ui.success()`

**Acceptance Criteria**:
- [ ] `ckrv term --worktree` creates a Git worktree under `.chakravarti/worktrees/`
- [ ] A new branch `worktree/<current>/ckrv-term-{id}` is created
- [ ] The agent process runs inside the worktree directory
- [ ] Changes made by the agent appear only in the worktree, not in the main repo
- [ ] `cargo build -p ckrv-cli` succeeds with the new import

---

### Task 1.3: Post-session prompt (diff/merge/keep/discard)
**Priority**: P0
**Estimate**: 1h
**Complexity**: 3
**File**: `crates/ckrv-cli/src/commands/term.rs`
**Depends on**: Task 1.2

After the agent process exits (in worktree mode), present an interactive prompt with four actions:

1. **View diff** — Run `git diff HEAD` in the worktree and display output
2. **Merge into current branch** — Commit any changes in the worktree, then `git merge --no-ff --no-edit <branch>` from the main repo
3. **Keep worktree for later** — Leave the worktree intact, print its path
4. **Discard all changes** — Run `manager.cleanup(&worktree)` to remove worktree and branch

Create:
- `PostAction` enum: `Merge`, `Diff`, `Keep`, `Discard`
- `post_session_prompt(cwd, worktree) -> Result<PostAction>` — uses `dialoguer::Select`
- `merge_worktree(cwd, worktree) -> Result<()>` — commits + merges using git CLI commands (pattern from `run.rs` lines 1186-1230)
- `show_diff(worktree) -> Result<()>` — runs `git diff HEAD` in worktree dir

After "View diff", re-prompt so user can then merge, keep, or discard.

**Acceptance Criteria**:
- [ ] After agent exits, user sees 4-option interactive prompt
- [ ] "View diff" shows changes and re-prompts
- [ ] "Merge" commits changes in worktree and merges into current branch
- [ ] "Keep" prints the worktree path and exits cleanly
- [ ] "Discard" removes the worktree and branch completely
- [ ] If agent made no changes, prompt is skipped with "No changes to review" message

---

### Task 1.4: Parallel session safety
**Priority**: P1
**Estimate**: 30m
**Complexity**: 2
**File**: `crates/ckrv-cli/src/commands/term.rs`
**Depends on**: Task 1.2

Ensure multiple simultaneous `ckrv term --worktree` sessions work correctly. Each generates a unique session ID via UUID, so worktree paths and branch names never collide. Verify by:

- Confirming UUID-based `session_id` creates unique worktree names
- Checking `DefaultWorktreeManager::create()` doesn't conflict when called concurrently
- Adding a guard that checks if the generated branch name already exists

**Acceptance Criteria**:
- [ ] Two concurrent `ckrv term --worktree` sessions create separate worktrees
- [ ] Each session has a unique branch name
- [ ] No data corruption when both sessions merge to the same parent branch sequentially

---

## Phase 2: Sandbox Mode (`--sandbox`)

*Goal: Agent runs inside a Docker container with proper credential mounts, using the existing `create_session()` → `docker exec -it` → `stop_session()` pipeline.*

### Task 2.1: Add `--sandbox` flag to `TermArgs`
**Priority**: P0
**Estimate**: 20m
**Complexity**: 2
**File**: `crates/ckrv-cli/src/commands/term.rs`
**Depends on**: Task 1.1

Add the `--sandbox` boolean flag to `TermArgs`. Update help text.

```rust
/// Run agent in a Docker sandbox container
#[arg(long)]
sandbox: bool,
```

**Acceptance Criteria**:
- [ ] `ckrv term --help` shows `--sandbox` flag
- [ ] `long_about` / `after_help` updated with sandbox examples
- [ ] No regression on existing `ckrv term` behavior

---

### Task 2.2: Agent type mapping (CLI ↔ Sandbox)
**Priority**: P0
**Estimate**: 30m
**Complexity**: 2
**File**: `crates/ckrv-cli/src/commands/term.rs`
**Depends on**: Task 2.1

Create a `to_sandbox_agent_type()` function that maps the CLI's 5-variant `AgentType` to the sandbox crate's 3-variant `AgentType`. This is needed to call `create_agent()` from `ckrv-sandbox`.

```rust
fn to_sandbox_agent_type(cli_type: &AgentType) -> ckrv_sandbox::AgentType {
    match cli_type {
        AgentType::Claude | AgentType::ClaudeOpenRouter | AgentType::ClaudeGlm
            => ckrv_sandbox::AgentType::Claude,
        AgentType::Codex => ckrv_sandbox::AgentType::Codex,
        AgentType::KiloCode => ckrv_sandbox::AgentType::KiloCode,
    }
}
```

Also create an `env_vars_to_hashmap()` helper to convert the `Vec<(String, String)>` from `build_agent_command()` into `HashMap<String, String>` for Docker's `create_session()`.

**Acceptance Criteria**:
- [ ] All 5 CLI agent types map correctly to sandbox agent types
- [ ] OpenRouter and GLM env vars (API keys, base URL, model overrides) are preserved in the HashMap
- [ ] `cargo build -p ckrv-cli` succeeds with new `ckrv-sandbox` dependency usage

---

### Task 2.3: Docker session lifecycle in `execute()`
**Priority**: P0
**Estimate**: 1h 30m
**Complexity**: 4
**File**: `crates/ckrv-cli/src/commands/term.rs`
**Depends on**: Task 2.2

When `--sandbox` is set, create a Docker session container and exec the agent inside it interactively:

1. `DockerClient::new()` — connect to Docker (fail with helpful error if Docker unavailable)
2. `create_agent(sandbox_type)` — get the `AgentProvider` for credential mounts
3. `agent_provider.config_mounts(host_home, container_home)` — get credential mount list
4. Convert bollard `Mount` list to `BindMount` list for `create_session()`
5. `docker.create_session(workdir, mount_source, mount_target, env, extra_mounts)` — start container
6. **Interactive exec via `docker exec -it`** — shell out to `Command::new("docker").args(["exec", "-it", &container_id, ...agent_cmd])` for proper PTY forwarding
7. `docker.stop_session(&container_id)` — cleanup on exit

The workspace mount source is either `cwd` (default) or `worktree.path` (if `--worktree` also set).

**Acceptance Criteria**:
- [ ] `ckrv term --sandbox` creates a Docker container with agent mounted
- [ ] Agent credentials (`.claude.json`, `.codex/`, etc.) are mounted read-only
- [ ] Agent env vars from `build_agent_command()` are passed into the container
- [ ] Agent process is interactive with full PTY (stdin/stdout/stderr forwarded)
- [ ] Container is removed after agent exits
- [ ] Clear error message if Docker is not available

---

### Task 2.4: Ctrl+C signal handling for container cleanup
**Priority**: P0
**Estimate**: 30m
**Complexity**: 3
**File**: `crates/ckrv-cli/src/commands/term.rs`
**Depends on**: Task 2.3

Register a signal handler so that Ctrl+C during a sandbox session stops and removes the container. Without this, orphaned containers accumulate.

Use `tokio::signal::ctrl_c()` or the `ctrlc` crate to spawn a cleanup task that calls `docker.stop_session(&container_id)` on interrupt. Must handle the case where the container has already stopped naturally.

**Acceptance Criteria**:
- [ ] Ctrl+C during sandbox session stops and removes the container
- [ ] No orphaned `ckrv-session-*` containers left after any exit path
- [ ] Signal handler doesn't interfere with agent's own Ctrl+C handling (agent receives SIGINT first)
- [ ] Graceful cleanup message shown to user

---

## Phase 3: Combined Mode + Named Sessions

*Goal: `--sandbox --worktree` provides both isolation layers, `--name` enables resume.*

### Task 3.1: Combined `--sandbox --worktree` mode
**Priority**: P0
**Estimate**: 45m
**Complexity**: 3
**File**: `crates/ckrv-cli/src/commands/term.rs`
**Depends on**: Task 1.2, Task 2.3

When both flags are set:
1. Create worktree first (Task 1.2 logic)
2. Mount the **worktree path** (not `cwd`) as the Docker workspace
3. Exec agent inside the container
4. On exit: stop container first, THEN show the post-session prompt for the worktree

The key difference from `--sandbox` alone: `mount_source = worktree.path` instead of `cwd`.

**Acceptance Criteria**:
- [ ] `ckrv term --sandbox --worktree` creates worktree AND Docker container
- [ ] Docker container mounts the worktree directory, not the main repo
- [ ] Agent changes appear only in the worktree branch
- [ ] Container cleaned up before worktree prompt appears
- [ ] Post-session prompt (merge/diff/keep/discard) works after container exit

---

### Task 3.2: `--name` flag and session state persistence
**Priority**: P1
**Estimate**: 1h
**Complexity**: 3
**File**: `crates/ckrv-cli/src/commands/term.rs`
**Depends on**: Task 3.1

Add `--name <session_name>` flag that:
1. Uses a deterministic session ID: `term-<name>` instead of `term-<uuid>`
2. Saves session state to `.chakravarti/sessions/<name>.yaml` on creation
3. Loads session state for `--resume` and `--cleanup` commands

Session state YAML structure:
```yaml
name: fix-auth-bug
agent_id: claude-native
container_id: "abc123def456"   # only if --sandbox
worktree_path: "/repo/.chakravarti/worktrees/term-fix-auth-bug_1"
worktree_branch: "worktree/main/ckrv-term-fix-auth-bug"
created_at: "2026-02-15T20:54:00Z"
status: active  # active | stopped | merged | discarded
```

Add `SessionState` struct with `Serialize`/`Deserialize`, `save()` and `load()` methods.

**Acceptance Criteria**:
- [ ] `ckrv term --worktree --name fix-auth` creates a named worktree `term-fix-auth`
- [ ] Session state written to `.chakravarti/sessions/fix-auth.yaml`
- [ ] Duplicate `--name` fails with clear error if session already active
- [ ] `.chakravarti/sessions/` directory auto-created if missing

---

### Task 3.3: `--resume` flag
**Priority**: P1
**Estimate**: 45m
**Complexity**: 3
**File**: `crates/ckrv-cli/src/commands/term.rs`
**Depends on**: Task 3.2

Add `--resume <name>` flag that:
1. Loads session state from `.chakravarti/sessions/<name>.yaml`
2. Verifies the worktree still exists
3. If `--sandbox`: creates a **new** container mounting the existing worktree (containers are ephemeral)
4. Spawns the agent in the existing worktree/container
5. On exit: updates session state, shows post-session prompt

`--resume` conflicts with `--agent` (agent is stored in session state).

**Acceptance Criteria**:
- [ ] `ckrv term --resume fix-auth` reconnects to the named worktree
- [ ] Agent type restored from session state (no need to re-specify `--agent`)
- [ ] Clear error if session doesn't exist or worktree was deleted
- [ ] Post-session prompt works the same as a fresh session

---

### Task 3.4: `--list-sessions` and `--cleanup` flags
**Priority**: P2
**Estimate**: 30m
**Complexity**: 2
**File**: `crates/ckrv-cli/src/commands/term.rs`
**Depends on**: Task 3.2

Add two session management flags:

**`--list-sessions`**: Lists all `.chakravarti/sessions/*.yaml` files with their status, agent, and age:
```
Active sessions:
  fix-auth    claude-native    worktree+sandbox    2h ago
  refactor    codex            worktree            45m ago
```

**`--cleanup <name>`**: Removes session state file, stops any running container, and optionally removes the worktree (with confirmation prompt).

**Acceptance Criteria**:
- [ ] `ckrv term --list-sessions` shows table of active sessions
- [ ] `ckrv term --cleanup fix-auth` removes session state and worktree
- [ ] Cleanup with no matching session gives clear error
- [ ] JSON output supported for both flags (`--json`)

---

## Phase 4: Polish & Documentation

*Goal: Robust edge cases, tests, and documentation.*

### Task 4.1: Error handling and edge cases
**Priority**: P1
**Estimate**: 45m
**Complexity**: 3
**File**: `crates/ckrv-cli/src/commands/term.rs`
**Depends on**: Phase 1, Phase 2

Handle edge cases:
- Agent exits with non-zero code → still show post-session prompt
- Docker not installed → clear error with install instructions
- Worktree creation fails (dirty index, branch exists) → helpful error
- Merge conflicts during post-session merge → offer abort or manual resolution path
- `--sandbox` without Docker running → detect and suggest `docker info`
- Permission errors on `.chakravarti/` directory

**Acceptance Criteria**:
- [ ] Every error path shows a clear, actionable message
- [ ] No panics or bare `unwrap()`s in new code
- [ ] Agent failure doesn't orphan containers or worktrees

---

### Task 4.2: Add `.chakravarti/sessions/` to `.gitignore`
**Priority**: P1
**Estimate**: 10m
**Complexity**: 1
**File**: `.gitignore`
**Depends on**: Task 3.2

Add `.chakravarti/sessions/` to the project `.gitignore`. Session state files contain container IDs and are machine-specific — they should never be committed.

**Acceptance Criteria**:
- [ ] `.chakravarti/sessions/` is gitignored
- [ ] `.chakravarti/worktrees/` already handled (verify)

---

### Task 4.3: Update CLI documentation attributes
**Priority**: P1
**Estimate**: 45m
**Complexity**: 2
**File**: `crates/ckrv-cli/src/commands/term.rs`
**Depends on**: All prior tasks

Update `long_about` and `after_help` on `TermArgs` to document all new flags with comprehensive examples covering every mode combination:

```
Examples:
  # Default (current behavior, no isolation)
  ckrv term

  # Isolated worktree — changes on a branch, merge when ready
  ckrv term --worktree

  # Docker sandbox — agent in a container
  ckrv term --sandbox

  # Maximum isolation — worktree + sandbox
  ckrv term --sandbox --worktree

  # Named session for resume
  ckrv term --worktree --name fix-auth

  # Resume a session
  ckrv term --resume fix-auth

  # Session management
  ckrv term --list-sessions
  ckrv term --cleanup fix-auth
```

Run `make docs` / `make skill` to regenerate documentation.

**Acceptance Criteria**:
- [ ] `ckrv term --help` shows all new flags with descriptions
- [ ] `after_help` contains examples for every mode combination
- [ ] Generated SKILL.md reflects the updated term command
- [ ] `long_about` explains the isolation model (worktree vs sandbox vs combined)

---

## Dependencies

```
Phase 1 (Worktree)                   Phase 2 (Sandbox)
──────────────────                   ─────────────────
T1.1 ─→ T1.2 ─→ T1.3               T2.1 ─→ T2.2 ─→ T2.3 ─→ T2.4
              └─→ T1.4
                   │                              │
                   └──────────┬───────────────────┘
                              ▼
                   Phase 3 (Combined + Sessions)
                   ─────────────────────────────
                   T3.1 ─→ T3.2 ─→ T3.3
                                └─→ T3.4
                                     │
                              ┌──────┘
                              ▼
                   Phase 4 (Polish)
                   ───────────────
                   T4.1 (edge cases)
                   T4.2 (gitignore)
                   T4.3 (docs)
```

**Phase 1 and Phase 2 can run in parallel** — they share no code paths until Phase 3 combines them.

## Blockers

- [ ] Docker must be available for sandbox mode testing (Phases 2, 3)
- [ ] `ckrv-sandbox` crate dependency must be added to `ckrv-cli/Cargo.toml` (may already be there via `ckrv-core`)

## Notes

- **All new code lives in `term.rs`** — no modifications needed in `ckrv-git`, `ckrv-sandbox`, or agent providers
- Use `run.rs` (lines 1096-1377) as the reference pattern for worktree creation, git commit, and merge flow
- `docker exec -it` shellout is the v1 approach for PTY forwarding; bollard-native PTY can be a future optimization
- Phase 1 has the fastest feedback loop (no Docker dependency) — prototype here first
- Consider extracting `TermSession` struct if `term.rs` exceeds 700 lines after all phases
