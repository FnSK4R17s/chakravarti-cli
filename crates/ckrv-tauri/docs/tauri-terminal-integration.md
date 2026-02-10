# Tauri PTY Terminal Integration

This document explains the split terminal architecture between Web (Axum) and Desktop (Tauri) modes.

## Architecture Overview

```
┌─────────────────────────────────────────────────────────────────┐
│                      Shared Backend (ckrv-sandbox)               │
│  - DockerClient::create_session() - Container creation          │
│  - DockerClient::stop_session()   - Container cleanup           │
│  - Agent environment setup        - API keys, base URLs          │
└─────────────────────────────────────────────────────────────────┘
                              │
         ┌────────────────────┴─────────────────────┐
         ▼                                          ▼
┌─────────────────────────┐          ┌─────────────────────────┐
│   Web Mode (Axum)       │          │   Desktop Mode (Tauri)   │
│                         │          │                          │
│ Backend:                │          │ Backend:                 │
│ - WebSocket server      │          │ - tauri-plugin-pty       │
│ - bollard Docker attach │          │ - Container ID only      │
│                         │          │                          │
│ Frontend:               │          │ Frontend:                │
│ - new WebSocket(...)    │          │ - spawn("docker", [...]) │
│ - ws.onmessage()        │          │ - pty.onData()           │
└─────────────────────────┘          └─────────────────────────┘
```

## Components Using Terminal

| Component | File | Purpose |
|-----------|------|---------|
| AgentCliModal | `components/AgentCliModal.tsx` | Interactive agent shell |
| TestFixModal | `components/TestFixModal.tsx` | Test fix agent shell |
| TaskDetailModal | `components/TaskDetailModal.tsx` | Task execution shell |

## Dependencies

### Rust (ckrv-tauri/Cargo.toml)

```toml
tauri-plugin-pty = "0.2"
```

### NPM (ckrv-ui/frontend/package.json)

```json
"tauri-pty": "^0.2"
```

### Initialization (ckrv-tauri/src/main.rs)

```rust
.plugin(tauri_plugin_pty::init())
```

### Capabilities (ckrv-tauri/capabilities/default.json)

**Required permission** - without this, spawn will fail with permission error:

```json
{
    "permissions": [
        "pty:default"
    ]
}
```

## How It Works

### 1. Container Creation (Shared)

Both modes use the same container creation flow:

```
Frontend calls /api/terminal/start
    ↓
terminal_start command (terminal.rs)
    ↓
DockerClient::create_session()
    ↓
Returns: { container_id, mode: "tauri" | "web" }
```

### 2. Interactive Shell (Different)

**Web Mode:**
```typescript
const ws = new WebSocket(`ws://host/api/terminal/ws?session_id=...`);
ws.onmessage = (e) => term.write(e.data);
term.onData((data) => ws.send(data));
```

**Tauri Mode:**
```typescript
const { spawn } = await import('tauri-pty');
const pty = await spawn('docker', ['exec', '-it', containerId, '/bin/bash', '-l'], {
    cols: term.cols,
    rows: term.rows,
});
pty.onData((data) => term.write(data));
term.onData((data) => pty.write(data));
```

## Upgrade Path

When upgrading Tauri:

1. **Check tauri-plugin-pty compatibility**
   - Visit: https://github.com/Tnze/tauri-plugin-pty
   - Check releases for Tauri 2.x compatibility

2. **Update both Rust and NPM packages**
   ```bash
   # Rust
   cargo update -p tauri-plugin-pty
   
   # NPM
   npm update tauri-pty
   ```

3. **Verify versions match**
   - The Rust crate and NPM package versions should match (e.g., both 0.2.x)

4. **Test terminal functionality**
   ```bash
   cd crates/ckrv-tauri && cargo run
   # Open terminal modal, verify interactive shell works
   ```

## Fallback Behavior

If PTY fails (e.g., plugin issue), the code falls back to IPC polling mode:

```typescript
try {
    const pty = await spawn(...);
    // PTY mode
} catch (ptyError) {
    // Fallback to polling mode
    const pollOutput = async () => { ... };
    pollOutput();
}
```

This ensures the terminal still works (though with reduced interactivity) even if PTY has issues.

## Debug Tips

1. **Check PTY plugin is loaded**
   ```rust
   // main.rs - should see log on startup
   tracing::info!("PTY plugin initialized");
   ```

2. **Check container is running**
   ```bash
   docker ps | grep ckrv-session
   ```

3. **Manual docker exec test**
   ```bash
   docker exec -it <container_id> /bin/bash
   ```

4. **Frontend console logs**
   - Look for `[AgentCliModal] PTY spawn failed:` errors

## Related Files

- `crates/ckrv-tauri/src/commands/terminal.rs` - Container creation/stop
- `crates/ckrv-tauri/src/main.rs` - Plugin initialization
- `crates/ckrv-tauri/Cargo.toml` - Rust dependencies
- `crates/ckrv-ui/frontend/src/hooks/useTauriPty.ts` - Shared PTY hook (documentation)
- `crates/ckrv-ui/frontend/src/components/AgentCliModal.tsx` - Terminal implementation
- `crates/ckrv-sandbox/src/docker.rs` - DockerClient for container management
