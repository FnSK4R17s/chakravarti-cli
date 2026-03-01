---
command: ui
generated_from: lib.rs
last_commit: f92f604
---

# ckrv ui

Start the Web UI dashboard.

## Description

Start the Web UI dashboard.

Launches a local web server providing a visual interface for managing specifications, viewing execution progress, and reviewing AI agent output.

Opens automatically in your default browser.

## Options

| Flag | Description |
|------|-------------|
| `--port <PORT>` | Port to listen on (default: 3000) |

## Examples

```bash
# Start UI on default port
ckrv ui

# Start on custom port
ckrv ui --port 8080

# Don't open browser automatically
ckrv ui --no-open
```
