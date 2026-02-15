---
description: Interactive terminal in Tauri using tauri-plugin-pty with xterm.js
---

# Tauri PTY Terminal Skill

Implement interactive PTY terminals in Tauri apps with xterm.js. This skill covers the architecture split between Web (WebSocket) and Desktop (PTY) modes.

## Architecture Overview

```
┌─────────────────────────────────────────────────────────────────┐
│                      Shared Backend (ckrv-sandbox)               │
│  - DockerClient::create_session() - Container creation          │
│  - DockerClient::stop_session()   - Container cleanup           │
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

## Dependencies

### Rust (Cargo.toml)

```toml
tauri-plugin-pty = "0.2"
```

### NPM (package.json)

```json
"tauri-pty": "^0.2"
```

**Important:** Versions must match! Both 0.2.x

## Setup Checklist

### 1. Rust Backend

```rust
// main.rs - Add to builder
.plugin(tauri_plugin_pty::init())
```

### 2. Capabilities (capabilities/default.json)

```json
{
    "identifier": "default",
    "windows": ["main"],
    "permissions": [
        "pty:default"
    ]
}
```

**Critical:** Use `"windows": ["main"]` NOT `"local": true` for dev mode!

### 3. Frontend Pattern

```typescript
// Check mode from API response
const isTauriMode = data.mode === 'tauri' || (window as any).__TAURI__;

if (isTauriMode && data.container_id) {
    // Tauri mode: spawn PTY
    const { spawn } = await import('tauri-pty');
    
    const pty = spawn('docker', [
        'exec', '-it', containerId, '/bin/bash', '-l'
    ], {
        cols: term.cols,
        rows: term.rows,
    });

    // CRITICAL: Data comes as array-like, wrap in Uint8Array
    pty.onData((data) => {
        term.write(new Uint8Array(data));
    });

    term.onData((data) => pty.write(data));
    term.onResize(({ cols, rows }) => pty.resize(cols, rows));
    
} else {
    // Web mode: use WebSocket
    const ws = new WebSocket(`ws://host/api/terminal/ws?session_id=...`);
    ws.onmessage = (e) => term.write(e.data);
    term.onData((data) => ws.send(data));
}
```

## Common Gotchas

### 1. Permission Denied Error

```
pty.spawn not allowed. Permissions: pty:allow-spawn
```

**Fix:** Add `"pty:default"` to capabilities AND use `"windows": ["main"]`

### 2. Type Error in onData

```
TypeError: decode error / Reading error
```

**Fix:** Data is array-like, NOT Uint8Array directly:
```typescript
// WRONG
pty.onData((data) => term.write(data));

// CORRECT
pty.onData((data) => term.write(new Uint8Array(data)));
```

### 3. Terminal Shows Messages But No Shell

Container was created but PTY spawn failed. Check:
1. Container is running: `docker ps | grep ckrv-session`
2. Manual test: `docker exec -it <container_id> /bin/bash`

### 4. Dev Mode URL Mismatch

If using `"local": true` in capabilities, it only works for bundled app (tauri://localhost), not dev server (http://localhost:5173).

**Fix:** Use `"windows": ["main"]` instead.

## Component Files

| Component | Path | Purpose |
|-----------|------|---------|
| AgentCliModal | `src/components/AgentCliModal.tsx` | Agent terminal |
| TestFixModal | `src/components/TestFixModal.tsx` | Test fix terminal |
| TaskDetailModal | `src/components/TaskDetailModal.tsx` | Task terminal |
| useTauriPty | `src/hooks/useTauriPty.ts` | Shared PTY hook (docs) |

## Cleanup Pattern

```typescript
// Store PTY reference for cleanup
(term as any).__pty = pty;

// In cleanup/unmount
return () => {
    // Kill PTY if exists
    if (xtermRef.current?.__pty) {
        try { xtermRef.current.__pty.kill(); } catch {}
    }
    xtermRef.current?.dispose();
    stopTerminalSession(sessionId);
};
```

## Testing

1. Start Tauri dev: `cd crates/ckrv-tauri && cargo tauri dev`
2. Open terminal modal
3. Should see:
   - "# Running in Tauri desktop mode (PTY)"
   - "# Connected! Type commands below."
   - Bash prompt (e.g., `bash-5.x$`)
4. Test commands work interactively

## Upgrade Path

When upgrading Tauri or the PTY plugin:

1. Check compatibility at https://github.com/Tnze/tauri-plugin-pty
2. Update both:
   ```bash
   # Rust
   cargo update -p tauri-plugin-pty
   
   # NPM
   npm update tauri-pty
   ```
3. Verify versions match
4. Test terminal functionality

## References

- [tauri-plugin-pty GitHub](https://github.com/Tnze/tauri-plugin-pty)
- [Official Example](https://github.com/Tnze/tauri-plugin-pty/tree/main/examples/vanilla)
- [xterm.js Docs](https://xtermjs.org/docs/)
- Local docs: `crates/ckrv-tauri/docs/tauri-terminal-integration.md`
