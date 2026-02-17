# Supercharged `ckrv term` — Bugfix Tasks (04)

**Brainstorm**: [notes.md](./notes.md)
**Created**: 2026-02-16
**Source**: Manual QA — `ckrv term --sandbox` agent exits immediately without showing interactive TUI

## Bugfix Overview

| # | Bug | Severity | Estimate |
|---|-----|----------|----------|
| BF-09 | `execute_in_sandbox()` uses wrong Docker image | Critical | 15m |
| BF-10 | `create_session()` doesn't set non-root user | High | 10m |
| BF-11 | `create_session()` hardcodes HOME to `/home/claude` | High | 10m |
| BF-12 | `docker exec` missing env vars for agent credentials | High | 15m |

**Severity breakdown**: 1 Critical, 3 High
**Total estimate**: ~50m

---

## BF-09: `execute_in_sandbox()` uses wrong Docker image

**Severity**: Critical
**File(s)**: `crates/ckrv-cli/src/commands/term.rs:924-1048`
**Estimate**: 15m

### Problem

`execute_in_sandbox()` creates a `DockerClient` (line 934) which defaults to `ckrv-agent:latest`. It **never calls `docker.set_image()`** to switch to the agent-specific image (`ckrv-claude:latest`, `ckrv-codex:latest`, `ckrv-kilo:latest`).

As a result, the container always uses `ckrv-agent:latest` — an image that:
- Is not built by the Makefile (`make install` only builds per-agent images)
- Has no `USER` directive (runs as root)
- Has no `ENV HOME` set
- Uses `/home/claude` as home dir even for Codex/Kilo

**Contrast with Tauri**, which does it correctly:

```rust
// crates/ckrv-tauri/src/commands/terminal.rs:187-194
let image = if is_codex {
    "ckrv-codex:latest"
} else if is_kilo {
    "ckrv-kilo:latest"
} else {
    "ckrv-claude:latest"
};
docker.set_image(image);
```

### Root Cause

`execute_in_sandbox()` in `term.rs` (line 934) creates a `DockerClient` but never sets the image based on the agent type.

### Fix

Add agent-to-image mapping and call `docker.set_image()` after creating the client:

```rust
async fn execute_in_sandbox(
    binary: &str,
    env_vars: &[(String, String)],
    extra_args: &[String],
    agent: &AgentConfig,
    working_dir: &std::path::Path,
    _session_id: &str,
    ui: &UiContext,
) -> anyhow::Result<std::process::ExitStatus> {
    // Check Docker availability
    let mut docker = DockerClient::new().map_err(|e| { /* ... */ })?;

    // Health check
    docker.health_check().await.map_err(|_| { /* ... */ })?;

    // Set agent-specific Docker image
    let image = match &agent.agent_type {
        t if t.to_string().contains("codex") => "ckrv-codex:latest",
        t if t.to_string().contains("kilo") => "ckrv-kilo:latest",
        _ => "ckrv-claude:latest",  // Claude (all variants: native, openrouter, glm)
    };
    docker.set_image(image);

    // ... rest unchanged ...
}
```

> **Note**: The exact match logic should follow the same pattern used by `to_sandbox_agent_type()` already in `term.rs`. The Tauri code uses `AgentType::Codex` and `AgentType::KiloCode` enum variants which is cleaner. Use an equivalent approach based on what `agent.agent_type` field provides.

### Acceptance Criteria

- [ ] `execute_in_sandbox()` calls `docker.set_image()` with agent-specific image
- [ ] Claude agents use `ckrv-claude:latest`
- [ ] Codex agents use `ckrv-codex:latest`
- [ ] Kilo agents use `ckrv-kilo:latest`
- [ ] `docker` changed to `mut docker` for `set_image()` call

---

## BF-10: `create_session()` doesn't set non-root user

**Severity**: High
**File(s)**: `crates/ckrv-sandbox/src/docker.rs:510-580`
**Estimate**: 10m

### Problem

`create_session()` does NOT set `user` in the container `Config` (line 546-561), unlike `execute()` (line 160) and `execute_streaming()` (line 391) which both set `user: Some(user_spec)` using the host UID/GID.

This means the container always runs with whatever user the Dockerfile specifies (or root if none). Even after BF-07 adds `USER` directives to Dockerfiles, the container user won't match the host user's UID/GID, potentially causing permission issues with bind-mounted files.

```rust
// execute() and execute_streaming() both do this:
let user_spec = format!("{}:{}", uid_gid, gid);
let config = Config {
    user: Some(user_spec),  // ← Present in execute/execute_streaming
    // ...
};

// create_session() does NOT:
let config = Config {
    // user: ???  ← MISSING
    // ...
};
```

### Root Cause

`create_session()` was likely copied from an earlier version before the UID/GID mapping was added to `execute()`.

### Fix

Add the same UID/GID detection and `user` field to `create_session()`:

```rust
pub async fn create_session(
    &self,
    workdir: &str,
    mount_source: &str,
    mount_target: &str,
    env: HashMap<String, String>,
    extra_mounts: Vec<crate::executor::BindMount>,
) -> Result<String, SandboxError> {
    let image = &self.default_image;
    self.ensure_image(image).await?;
    let container_name = format!("ckrv-session-{}", uuid::Uuid::new_v4());

    // Prepare Env and Mounts
    let env_vec: Vec<String> = env.into_iter().map(|(k, v)| format!("{k}={v}")).collect();

    // ... mounts setup unchanged ...

    // Get current user UID:GID for proper permission handling
    let uid_gid = std::process::Command::new("id")
        .args(["-u"])
        .output()
        .ok()
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .map(|s| s.trim().to_string())
        .unwrap_or_else(|| "1000".to_string());

    let gid = std::process::Command::new("id")
        .args(["-g"])
        .output()
        .ok()
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .map(|s| s.trim().to_string())
        .unwrap_or_else(|| "1000".to_string());

    let user_spec = format!("{}:{}", uid_gid, gid);

    let config = Config {
        image: Some(image.to_string()),
        cmd: Some(vec!["tail".to_string(), "-f".to_string(), "/dev/null".to_string()]),
        working_dir: Some(workdir.to_string()),
        user: Some(user_spec),  // ← ADD THIS
        env: Some(env_vec),
        host_config: Some(HostConfig {
            mounts: Some(mounts),
            network_mode: Some("host".to_string()),
            ..Default::default()
        }),
        ..Default::default(),
    };

    // ... rest unchanged ...
}
```

### Acceptance Criteria

- [ ] `create_session()` sets `user: Some(user_spec)` in the container config
- [ ] UID/GID detection uses the same pattern as `execute()` and `execute_streaming()`
- [ ] Container filesystem permissions match host user (bind-mounts writable)

---

## BF-11: `create_session()` hardcodes HOME to `/home/claude`

**Severity**: High
**File(s)**: `crates/ckrv-sandbox/src/docker.rs:524-525`, `crates/ckrv-cli/src/commands/term.rs:961`
**Estimate**: 10m

### Problem

Two locations hardcode the container HOME to `/home/claude`:

1. **`docker.rs:524-525`** — `create_session()`:
   ```rust
   let container_home = "/home/claude".to_string();
   env_vec.push(format!("HOME={}", container_home));
   ```

2. **`term.rs:961`** — `execute_in_sandbox()`:
   ```rust
   let container_home = "/home/claude".to_string();
   ```

This is incorrect for Codex (`/home/codex`) and Kilo (`/home/kilo`) agents. The Tauri code handles this correctly:

```rust
// crates/ckrv-tauri/src/commands/terminal.rs:114-120
let home = if matches!(agent.agent_type, AgentType::Codex) {
    "/home/codex"
} else if matches!(agent.agent_type, AgentType::KiloCode) {
    "/home/kilo"
} else {
    "/home/claude"
};
```

### Fix

**Part A**: Remove hardcoded HOME from `create_session()`. Instead, let the caller pass HOME via the `env` HashMap (just like the Tauri code does).

```rust
// docker.rs — create_session()
// REMOVE these two lines:
// let container_home = "/home/claude".to_string();
// env_vec.push(format!("HOME={}", container_home));

// The caller (term.rs) is responsible for setting HOME in the env HashMap
```

**Part B**: In `execute_in_sandbox()` (term.rs), set the correct HOME based on agent type:

```rust
// term.rs — execute_in_sandbox()
// Replace:
//   let container_home = "/home/claude".to_string();
// With:
let container_home = match &agent.agent_type {
    t if t.to_string().contains("codex") => "/home/codex",
    t if t.to_string().contains("kilo") => "/home/kilo",
    _ => "/home/claude",
};

// Add HOME to env_map
env_map.insert("HOME".to_string(), container_home.to_string());
```

### Acceptance Criteria

- [ ] `create_session()` does NOT hardcode HOME — caller provides it
- [ ] `execute_in_sandbox()` sets HOME based on agent type
- [ ] Codex agents get `HOME=/home/codex`
- [ ] Kilo agents get `HOME=/home/kilo`
- [ ] Claude agents (all variants) get `HOME=/home/claude`

---

## BF-12: `docker exec` missing env vars for agent credentials

**Severity**: High
**File(s)**: `crates/ckrv-cli/src/commands/term.rs:1033-1037`
**Estimate**: 15m

### Problem

`execute_in_sandbox()` passes env vars to `create_session()` (which sets them on the container), and then runs the agent via `docker exec -it`. However, while Docker generally makes container-level env vars available to exec sessions, some environments or Docker versions may not propagate them reliably, and importantly the `docker exec` on line 1034 doesn't explicitly pass env vars:

```rust
// Current — no -e flags
let status = Command::new("docker")
    .args(["exec", "-it", "-w", container_workdir, &container_id])
    .args(&agent_cmd)
    .status();
```

The Tauri Dockerfile setup passes `TERM` and `COLORTERM` env vars for TUI rendering. The CLI's `execute_in_sandbox()` doesn't set these at all, which may cause TUI-based agents (Claude Code, Codex) to not render properly.

### Fix

Pass critical env vars explicitly via `-e` flags on `docker exec`:

```rust
// Build agent command
let mut agent_cmd: Vec<String> = vec![binary.to_string()];

// Add extra args from agent config
if let Some(config_args) = &agent.extra_args {
    agent_cmd.extend(config_args.clone());
}

// Add passthrough args
agent_cmd.extend(extra_args.to_vec());

// Build docker exec command with env vars
let mut docker_args = vec![
    "exec".to_string(),
    "-it".to_string(),
    "-w".to_string(),
    container_workdir.to_string(),
];

// Pass env vars explicitly — ensures they're available even if
// Docker doesn't propagate container-level vars to exec sessions
for (key, value) in env_vars {
    docker_args.push("-e".to_string());
    docker_args.push(format!("{}={}", key, value));
}

// Add TERM for TUI rendering (Claude Code, Codex, Kilo all use TUI)
docker_args.push("-e".to_string());
docker_args.push("TERM=xterm-256color".to_string());
docker_args.push("-e".to_string());
docker_args.push("COLORTERM=truecolor".to_string());

// Container ID
docker_args.push(container_id.clone());

// Execute via docker exec -it for interactive PTY
let status = Command::new("docker")
    .args(&docker_args)
    .args(&agent_cmd)
    .status();
```

### Acceptance Criteria

- [ ] `docker exec` passes env vars via `-e` flags
- [ ] `TERM=xterm-256color` is set for TUI rendering
- [ ] `COLORTERM=truecolor` is set for color support
- [ ] Agent-specific env vars (API keys, base URLs) are passed through
- [ ] Agent can authenticate and render TUI inside the container

---

## Verification

After all bugfixes are applied:

- [ ] `cargo build -p ckrv-cli` and `cargo build -p ckrv-sandbox` succeed
- [ ] `ckrv term --sandbox` → Claude shows interactive TUI in terminal
- [ ] `ckrv term --sandbox --agent codex-agent` → uses `ckrv-codex:latest` image
- [ ] `ckrv term --sandbox --agent kilo-agent` → uses `ckrv-kilo:latest` image
- [ ] Container runs as host user (not root)
- [ ] `docker exec <container> whoami` shows non-root user
- [ ] Agent can read bind-mounted credential files
- [ ] Agent can write to `/workspace` inside container
- [ ] Agent TUI renders correctly (not blank/broken)
- [ ] Ctrl+C cleanly stops container

## Notes

- BF-09 is the **most likely cause** of the "agent exits immediately" issue — `ckrv-agent:latest` may have a stale build or misconfigured environment
- BF-10, BF-11, BF-12 are contributing factors that compound the problem
- The Tauri code (`crates/ckrv-tauri/src/commands/terminal.rs`) already handles all of these correctly — **use it as the reference implementation**
- These fixes are in two crates: `ckrv-cli` (BF-09, BF-11, BF-12) and `ckrv-sandbox` (BF-10)
- The `ckrv-agent:latest` image (from `Dockerfile.agent`) is the default but is **never built by `make install`** — consider adding it to the Makefile or removing the fallback
