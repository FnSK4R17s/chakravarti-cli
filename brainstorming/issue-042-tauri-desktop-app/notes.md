# Create app using Tauri

**Issue**: [#42](https://github.com/FnSK4R17s/chakravarti-cli/issues/42)
**Created**: 2026-02-04
**Status**: Tasks Generated

---

## Problem Statement

CKRV currently runs as a web application served by an Axum backend. While functional, this has limitations:

1. **Friction**: Users must start the server (`ckrv ui`) before accessing the interface
2. **No system integration**: Can't appear in app menus, OS search, or dock
3. **Distribution complexity**: Requires users to have Rust toolchain and build from source or install via crates.io
4. **No offline-first**: Requires running a localhost server

A native desktop app solves all of these while maintaining the existing web UI.

---

## Current State

### Existing Architecture (ckrv-ui)
```
Browser → HTTP (fetch) → Axum (ckrv-ui) → ckrv-core → Docker/Agents
```

The web frontend lives in `crates/ckrv-ui/frontend/` and uses:
- React 19 with TypeScript
- shadcn/ui + Tailwind CSS
- Vite for bundling
- xterm.js for terminal rendering

The backend (`crates/ckrv-ui/`) uses Axum to serve the frontend and provide REST APIs.

### Pain Points

1. Server must be manually started each time
2. No native file dialogs (constrained to HTML5 file picker)
3. Can't launch from OS Finder/Spotlight/Start Menu
4. Distribution requires `cargo install` or building from source

---

## Proposed Solution

Add a **Tauri v2** desktop wrapper that:
1. **Reuses 100% of the React frontend** (bundled into native WebView)
2. **Replaces Axum HTTP calls with Tauri IPC** (direct Rust function invocation)
3. **Maintains web distribution as parallel option** (same codebase, two targets)
4. **Achieves <20MB installer size** (vs ~150MB for Electron)

### Why Tauri Over Electron?

| Factor | Tauri | Electron |
|--------|-------|----------|
| Bundle size | ~15-20MB | ~150MB+ |
| Memory usage | ~30MB | ~100MB+ |
| Rust backend | ✅ Native | ❌ Requires bridge |
| Startup time | Fast | Slow |
| Security | Strict by default | Loose by default |

**Decision**: Tauri is the obvious choice—we already have a Rust backend.

---

## User Stories

### US1: Desktop Installation
**As a** developer,
**I want** to install CKRV as a native app from a DMG/MSI/DEB,
**So that** I can launch it like any other desktop application.

### US2: Zero-Config Launch
**As a** developer,
**I want** to double-click the app to start working,
**So that** I don't need to run terminal commands first.

### US3: Native File Dialogs
**As a** developer,
**I want** native file/folder pickers,
**So that** selecting project directories feels natural.

### US4: System Integration
**As a** developer,
**I want** CKRV in my OS app menu and Spotlight/search,
**So that** I can launch it quickly from anywhere.

---

## Technical Approach

### Architecture Overview

```
┌─────────────────────────────────────────┐
│         Tauri Native Window             │
│  ┌───────────────────────────────────┐  │
│  │        System WebView             │  │
│  │  ┌─────────────────────────────┐  │  │
│  │  │  React App (unchanged)      │  │  │
│  │  │  shadcn/Tailwind/Lucide     │  │  │
│  │  └─────────────┬───────────────┘  │  │
│  └────────────────┼──────────────────┘  │
│                   │ Tauri IPC (invoke)  │
│  ┌────────────────▼──────────────────┐  │
│  │       Tauri Command Layer         │  │
│  │       (ckrv-tauri crate)          │  │
│  └────────────────┬──────────────────┘  │
│  ┌────────────────▼──────────────────┐  │
│  │          ckrv-core                │  │
│  │     (Orchestrator, Executor)      │  │
│  └───────────────────────────────────┘  │
└─────────────────────────────────────────┘
```

### Frontend API Adapter Strategy

The key insight is creating a **unified API layer** that automatically switches between:
- **HTTP fetch** (when running in browser → uses existing Axum backend)
- **Tauri invoke** (when running in Tauri → direct Rust calls)

```typescript
const IS_TAURI = typeof window !== 'undefined' && '__TAURI__' in window;

export async function loadSpec(path: string): Promise<Spec> {
  if (IS_TAURI) {
    return tauriInvoke('load_spec', { path });
  }
  return httpFetch(`/spec?path=${encodeURIComponent(path)}`);
}
```

**Benefit**: Same component code works in both web and desktop builds.

### Integration with Existing Code

| Existing Crate | Role in Tauri |
|----------------|---------------|
| `ckrv-core` | Orchestration logic (unchanged) |
| `ckrv-types` | Shared types for commands (unchanged) |
| `ckrv-executor` | Docker/local execution (unchanged) |
| `ckrv-ui/frontend` | React app (minor API layer changes) |
| `ckrv-ui` (Axum) | Not used by Tauri (web-only) |

---

## Open Questions (from spec)

### 1. Auto-updates: Defer to v2? ✅ RESOLVED
**Context**: Tauri has a built-in updater plugin.

**Decision**: **Defer to v2**
- Auto-updates require signing infrastructure (code signing certs)
- macOS notarization adds complexity
- Manual updates via GitHub releases are acceptable for v1 power users

### 2. System Tray: Show execution status? ✅ RESOLVED
**Context**: Could show ● running / ✓ complete / ✗ failed in system tray.

**Decision**: **Defer to v2**
- System tray adds platform-specific complexity
- Not core to the value proposition for v1

### 3. Deep linking: Support `ckrv://open?project=/path`? ✅ RESOLVED
**Context**: Could register custom URL scheme to open projects from browser/terminal.

**Decision**: **Defer** - Nice-to-have but not essential for v1.

### 4. Portable mode: Single executable for Windows? ✅ RESOLVED
**Context**: Some Windows users prefer portable apps (no installer, single .exe).

**Decision**: **MSI only for v1** - Consider portable in future versions.

---

## New Questions Raised

### 5. Web vs Desktop: Which is primary distribution? ✅ RESOLVED
**Context**: Both web (via `ckrv ui`) and desktop (native app) will work.

**Decision**: **Both are equal priorities**
- Desktop app: Standalone distribution via GitHub releases, no CLI required
- Web (`ckrv ui`): For CLI power users, remote servers, development
- Neither is deprecated; both are first-class distribution paths

### 6. Naming: "Chakravarti-cli" for app name
**Context**: `tauri.conf.json` needs a `productName`.

**Decision**: **"Chakravarti-cli"** for productName (appears in title bar, installers). This follows the naming conventions from `guiding_docs/vision.md`:
- `chakravarti-cli` is the full project name
- `ckrv` is the CLI command shorthand
- "Chakravarti" alone is only used when explaining the name's meaning

### 7. Combined Interface Crate: `ckrv-interface` (Brainstorming)

**Context**: Currently, the brainstorming plan proposes a frontend `api.ts` adapter that checks `IS_TAURI` at runtime to switch between `fetch()` and `invoke()`. This works but has drawbacks.

**Idea**: Create a **unified Rust interface crate** that abstracts Axum/Tauri at compile time using feature flags. The frontend would import from this single interface, unaware of the underlying transport.

#### Current Approach (Frontend-side switching)
```typescript
// Frontend must know about both transports
if (IS_TAURI) {
  return tauriInvoke('list_agents');
}
return httpFetch('/agents');
```

**Problems**:
1. Every API function needs dual logic
2. Frontend is coupled to transport details
3. 50+ `fetch()` calls scattered across components need migration
4. Runtime overhead for environment detection

#### Proposed Approach (Rust-side switching)
```
                     Frontend
                        │
                        ▼
           ┌────── ckrv-interface ──────┐
           │  (single TypeScript API)   │
           └─────────────┬──────────────┘
                         │
        ┌────────────────┼────────────────┐
        │                │                │
        ▼                ▼                ▼
   [axum feature]  [tauri feature]  [wasm feature?]
        │                │
        ▼                ▼
   ckrv-ui (Axum)   ckrv-tauri
```

**Key insight**: The Rust crate would export a **single interface** that compiles differently based on the target:

```rust
// crates/ckrv-interface/src/lib.rs

#[cfg(feature = "axum")]
mod axum_transport;

#[cfg(feature = "tauri")]
mod tauri_transport;

pub trait BackendInterface {
    async fn list_agents(&self) -> Result<Vec<Agent>, Error>;
    async fn create_spec(&self, req: CreateSpecRequest) -> Result<Spec, Error>;
    // ... all other endpoints
}

#[cfg(feature = "axum")]
pub type Backend = axum_transport::AxumBackend;

#[cfg(feature = "tauri")]
pub type Backend = tauri_transport::TauriBackend;
```

**Frontend would be simpler**:
```typescript
// No IS_TAURI check needed - the correct transport is compiled in
import { listAgents, createSpec } from '@ckrv/interface';

// Just use the API
const agents = await listAgents();
```

#### Trade-offs

| Aspect | Current (Frontend adapter) | Proposed (Rust crate) |
|--------|---------------------------|----------------------|
| **Frontend complexity** | High - dual logic everywhere | Low - single API surface |
| **Build complexity** | Simple - one build | Complex - different builds |
| **Runtime flexibility** | Can switch at runtime | Fixed at compile time |
| **Type safety** | Manual sync needed | Compile-time guarantees |
| **Testing** | Test both paths in one build | Need to test each build variant |
| **Maintenance** | Change in 3 places | Change in 1 place |

#### Implementation Options

**Option A: Full Rust interface crate**
- Create `crates/ckrv-interface` with the trait-based approach
- Build produces `@ckrv/interface-web` and `@ckrv/interface-tauri` npm packages
- Frontend imports from the appropriate package based on build target

**Option B: Hybrid approach**
- Keep simple frontend adapter (`api.ts`)
- Move complex logic to Rust-side where it makes sense
- Use compile-time feature flags only for truly different behavior

**Option C: Code generation**
- Define API spec in OpenAPI or similar
- Generate both Axum handlers and Tauri commands from spec
- Generate TypeScript adapter automatically
- Guarantees parity by construction

**Option D: Rust Transport Crate (`ckrv-transport`)** ← NEW

This is the cleanest Rust-native approach. Create a separate crate that defines the API interface with feature-flagged implementations:

```
crates/
├── ckrv-transport/          # NEW: Transport abstraction
│   ├── Cargo.toml
│   └── src/
│       ├── lib.rs           # Trait definitions + re-exports
│       ├── handlers.rs      # Shared handler logic
│       ├── axum.rs          # Axum-specific (feature = "axum")
│       ├── tauri.rs         # Tauri-specific (feature = "tauri")
│       └── types.rs         # Request/Response types (with ts-rs)
├── ckrv-ui/                  # Web UI (uses ckrv-transport with "axum" feature)
├── ckrv-tauri/              # Desktop app (uses ckrv-transport with "tauri" feature)
└── ckrv-core/               # Business logic (unchanged)
```

##### How It Works

```rust
// crates/ckrv-transport/Cargo.toml
[package]
name = "ckrv-transport"
version = "0.1.0"

[features]
default = []
axum = ["dep:axum", "dep:tower-http"]
tauri = ["dep:tauri"]

[dependencies]
ckrv-core = { path = "../ckrv-core" }
ckrv-types = { path = "../ckrv-types" }
serde = { version = "1", features = ["derive"] }
ts-rs = { version = "7", optional = true }

# Feature-gated
axum = { version = "0.7", optional = true }
tower-http = { version = "0.5", optional = true }
tauri = { version = "2", optional = true }
```

```rust
// crates/ckrv-transport/src/lib.rs

mod handlers;
pub mod types;

#[cfg(feature = "axum")]
pub mod axum;

#[cfg(feature = "tauri")]
pub mod tauri;

// Re-export the appropriate API surface
#[cfg(feature = "axum")]
pub use axum::create_router;

#[cfg(feature = "tauri")]
pub use tauri::get_invoke_handlers;
```

```rust
// crates/ckrv-transport/src/handlers.rs
// SHARED business logic - used by both axum and tauri

use ckrv_core::Orchestrator;
use crate::types::*;

pub async fn list_agents_handler(
    orchestrator: &Orchestrator
) -> Result<Vec<Agent>, TransportError> {
    orchestrator.list_agents()
        .await
        .map_err(TransportError::from)
}

pub async fn create_spec_handler(
    orchestrator: &Orchestrator,
    req: CreateSpecRequest,
) -> Result<Spec, TransportError> {
    orchestrator.create_spec(&req.name, req.description.as_deref())
        .await
        .map_err(TransportError::from)
}
```

```rust
// crates/ckrv-transport/src/axum.rs
use axum::{Router, Json, extract::State};
use crate::handlers;

pub fn create_router() -> Router<AppState> {
    Router::new()
        .route("/agents", get(list_agents))
        .route("/specs", post(create_spec))
        // ... all routes
}

async fn list_agents(State(state): State<AppState>) -> Json<Vec<Agent>> {
    Json(handlers::list_agents_handler(&state.orchestrator).await.unwrap())
}
```

```rust
// crates/ckrv-transport/src/tauri.rs
use tauri::State;
use crate::handlers;

#[tauri::command]
pub async fn list_agents(state: State<'_, AppState>) -> Result<Vec<Agent>, String> {
    handlers::list_agents_handler(&state.orchestrator)
        .await
        .map_err(|e| e.to_string())
}

pub fn get_invoke_handlers() -> impl Fn(tauri::Invoke) {
    tauri::generate_handler![
        list_agents,
        create_spec,
        // ... all commands
    ]
}
```

##### Consumer Crates

```rust
// crates/ckrv-ui/Cargo.toml
[dependencies]
ckrv-transport = { path = "../ckrv-transport", features = ["axum"] }

// crates/ckrv-ui/src/main.rs
use ckrv_transport::create_router;

fn main() {
    let app = create_router();
    axum::serve(listener, app).await.unwrap();
}
```

```rust
// crates/ckrv-tauri/Cargo.toml  
[dependencies]
ckrv-transport = { path = "../ckrv-transport", features = ["tauri"] }

// crates/ckrv-tauri/src/main.rs
use ckrv_transport::get_invoke_handlers;

fn main() {
    tauri::Builder::default()
        .invoke_handler(get_invoke_handlers())
        .run(tauri::generate_context!())
        .unwrap();
}
```

##### What About the Frontend?

The frontend STILL needs TypeScript to call the backend. But now:

1. **Types are generated once** from `ckrv-transport/src/types.rs` via `ts-rs`
2. **API surface is identical** because both Axum and Tauri use the same `handlers.rs`
3. **Frontend adapter is simpler** because it trusts the API contract

```
Rust (ckrv-transport)
    │
    ├──[ts-rs]──▶ frontend/src/types/api.generated.ts
    │
    └── Guarantees: Same handler logic → Same behavior
    
Frontend (TypeScript)
    │
    └── Still needs transport/http.ts and transport/tauri.ts
        (but they're thin wrappers now)
```

##### Benefits of This Approach

| Aspect | Before (Separate backends) | After (ckrv-transport) |
|--------|---------------------------|------------------------|
| Handler logic | Duplicated in ckrv-ui + ckrv-tauri | Single `handlers.rs` |
| Type definitions | Duplicated or manual sync | Single `types.rs` with ts-rs |
| Adding new endpoint | Change 3 files | Change 1 file + re-export |
| Testing | Test each backend separately | Test `handlers.rs` once |
| Parity guarantee | Manual discipline | Compile-time (same code) |

##### Why This Is Better Than Frontend-Only Adapter

The frontend adapter (Option B) still requires:
- Axum handlers in `ckrv-ui/src/api/*.rs`
- Tauri commands in `ckrv-tauri/src/commands/*.rs`  
- Manual sync between them

With `ckrv-transport`:
- Handler logic is **written once** in `handlers.rs`
- Axum/Tauri modules are **thin wrappers** that dispatch to handlers
- New endpoints are added in **one place**



#### Questions to Resolve

1. **How does the frontend know which build to use?**
   - Vite `define` plugin to set build-time constants?
   - Separate npm packages (`@ckrv/interface-web` vs `@ckrv/interface-tauri`)?
   - Import maps or aliases?

2. **Does this add too much build complexity?**
   - Need to build interface crate twice
   - CI needs to test both variants
   - Deployment is more complex

3. **Is runtime flexibility ever needed?**
   - Could a single build ever need to switch between Axum and Tauri?
   - If not, compile-time switching is cleaner

4. **What about WASM targets?**
   - Could the interface crate also target WASM for cloud-native deployments?
   - This would add a third compile target

#### Recommendation: **Option D (ckrv-transport crate)** ← UPDATED

After further analysis, the **Rust transport crate** is the cleanest approach:

| Approach | Pros | Cons |
|----------|------|------|
| **Option B (Frontend adapter)** | Simple, no Rust changes | Duplicated handler logic, manual sync |
| **Option D (ckrv-transport)** | Single source of truth, compile-time guarantees | Slightly more upfront work |

**Primary recommendation**: Create `ckrv-transport` crate with feature-flagged Axum/Tauri modules.

**Fallback**: If time-constrained, start with Option B (frontend adapter) and refactor to Option D later.

##### Why ckrv-transport Is Better

1. **Handler logic is written ONCE** in `handlers.rs` - no duplication
2. **Types are defined ONCE** in `types.rs` with `ts-rs` - no manual sync
3. **Adding a new endpoint** requires changes in ONE place
4. **Compile-time feature flags** ensure only the relevant transport code is included
5. **Testing is simpler** - test `handlers.rs` once, not both backends

##### Recommended Architecture

```
┌─────────────────────────────────────────────────────────────────────────┐
│                           RUST LAYER                                    │
│                                                                         │
│  ┌─────────────────────────────────────────────────────────────────┐   │
│  │                    ckrv-transport (NEW)                          │   │
│  │  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐              │   │
│  │  │ handlers.rs │  │  types.rs   │  │   lib.rs    │              │   │
│  │  │ (shared     │  │ (with ts-rs │  │ (feature    │              │   │
│  │  │  logic)     │  │  export)    │  │  gates)     │              │   │
│  │  └──────┬──────┘  └──────┬──────┘  └──────┬──────┘              │   │
│  │         │                │                │                      │   │
│  │         │    ┌───────────┴───────────┐    │                      │   │
│  │         │    │                       │    │                      │   │
│  │         ▼    ▼                       ▼    ▼                      │   │
│  │  ┌──────────────────┐       ┌──────────────────┐                │   │
│  │  │     axum.rs      │       │    tauri.rs      │                │   │
│  │  │ (feature="axum") │       │ (feature="tauri")│                │   │
│  │  └────────┬─────────┘       └────────┬─────────┘                │   │
│  └───────────┼──────────────────────────┼──────────────────────────┘   │
│              │                          │                              │
│              ▼                          ▼                              │
│  ┌───────────────────┐      ┌───────────────────┐                     │
│  │    ckrv-ui        │      │   ckrv-tauri      │                     │
│  │ (uses "axum"      │      │ (uses "tauri"     │                     │
│  │  feature)         │      │  feature)         │                     │
│  └───────────────────┘      └───────────────────┘                     │
│                                                                         │
└─────────────────────────────────────────────────────────────────────────┘

                                    │
                                    │ ts-rs generates
                                    ▼

┌─────────────────────────────────────────────────────────────────────────┐
│                        TYPESCRIPT LAYER                                 │
│                                                                         │
│  ┌─────────────────────────────────────────────────────────────────┐   │
│  │              frontend/src/types/api.generated.ts                 │   │
│  │                    (auto-generated from types.rs)                │   │
│  └─────────────────────────────────────────────────────────────────┘   │
│                                    │                                    │
│                                    ▼                                    │
│  ┌─────────────────────────────────────────────────────────────────┐   │
│  │                   frontend/src/lib/api.ts                        │   │
│  │           (thin wrapper - trusts Rust API contract)              │   │
│  └────────────────────────────┬────────────────────────────────────┘   │
│                               │                                         │
│         ┌─────────────────────┼─────────────────────┐                  │
│         ▼                     ▼                     ▼                  │
│  ┌─────────────┐      ┌─────────────┐      ┌─────────────┐            │
│  │ http.ts     │      │ tauri.ts    │      │ mock.ts     │            │
│  │ (fetch)     │      │ (invoke)    │      │ (testing)   │            │
│  └─────────────┘      └─────────────┘      └─────────────┘            │
│                                                                         │
└─────────────────────────────────────────────────────────────────────────┘
```

**Key insight**: The Rust `handlers.rs` contains ALL the business logic. The `axum.rs` and `tauri.rs` are just thin wrappers that:
1. Parse framework-specific inputs (HTTP request vs Tauri state)
2. Call the shared handler
3. Format framework-specific outputs (JSON response vs Tauri result)

##### Implementation Steps

**Step 1: Create transport layer files**

```typescript
// frontend/src/lib/transport/types.ts
export interface Transport {
  get<T>(endpoint: string): Promise<T>;
  post<T>(endpoint: string, body?: unknown): Promise<T>;
  delete<T>(endpoint: string): Promise<T>;
  stream(endpoint: string): AsyncIterable<unknown>;
}

// frontend/src/lib/transport/http.ts
export const httpTransport: Transport = {
  async get<T>(endpoint: string): Promise<T> {
    const res = await fetch(`/api${endpoint}`);
    if (!res.ok) throw new Error(await res.text());
    return res.json();
  },
  // ... post, delete, stream
};

// frontend/src/lib/transport/tauri.ts
import { invoke } from '@tauri-apps/api/core';

export const tauriTransport: Transport = {
  async get<T>(endpoint: string): Promise<T> {
    const cmd = endpointToCommand(endpoint); // '/agents' → 'list_agents'
    return invoke<T>(cmd);
  },
  // ... post, delete, stream (via Tauri events)
};
```

**Step 2: Configure Vite for build-time switching**

```typescript
// frontend/vite.config.ts
export default defineConfig(({ mode }) => ({
  define: {
    'import.meta.env.VITE_BACKEND': JSON.stringify(
      process.env.VITE_BACKEND || (mode === 'tauri' ? 'tauri' : 'axum')
    ),
  },
  resolve: {
    alias: {
      '@transport': 
        process.env.VITE_BACKEND === 'tauri'
          ? './src/lib/transport/tauri.ts'
          : './src/lib/transport/http.ts'
    }
  }
}));
```

**Step 3: Create unified api.ts**

```typescript
// frontend/src/lib/api.ts
import { transport } from '@transport';
import type { Agent, Spec, CreateSpecRequest } from '../types/api.generated';

// All API functions use the same transport - no if/else!
export const listAgents = () => transport.get<Agent[]>('/agents');
export const createSpec = (req: CreateSpecRequest) => 
  transport.post<Spec>('/specs', req);
export const getDockerStatus = () => transport.get<DockerStatus>('/docker');
// ... all 50+ endpoints
```

**Step 4: Add ts-rs for type generation**

```rust
// crates/ckrv-types/src/api_types.rs
use ts_rs::TS;

#[derive(TS, Serialize, Deserialize)]
#[ts(export)]
pub struct Agent {
    pub name: String,
    pub model: String,
    pub is_default: bool,
}

#[derive(TS, Serialize, Deserialize)]
#[ts(export)]
pub struct CreateSpecRequest {
    pub name: String,
    pub description: Option<String>,
}
```

Then run `cargo test` (ts-rs exports on test) or use a build script.

##### Benefits of This Approach

| Aspect | Benefit |
|--------|---------|
| **Single source of truth** | Rust types → TypeScript via `ts-rs` |
| **No runtime detection** | Build-time alias switching |
| **Clean frontend** | Components just call `api.listAgents()` |
| **Easy testing** | Add a `mock` transport for unit tests |
| **Incremental migration** | Convert endpoints one at a time |
| **Type safety** | Generated types ensure Rust/TS stay in sync |

##### Migration Path

1. **Phase 1**: Create `api.ts` + `transport/http.ts` that wraps current `fetch()` calls
2. **Phase 2**: Migrate components from direct `fetch()` to `api.*` calls
3. **Phase 3**: Add `transport/tauri.ts` and Vite alias switching
4. **Phase 4**: Add `ts-rs` type generation for compile-time safety

##### Open Question: Endpoint-to-Command Mapping

For Tauri, we need to map REST endpoints to command names:
- `GET /agents` → `list_agents`
- `POST /agents` → `upsert_agent`
- `DELETE /agents/:id` → `delete_agent`

**Options**:
1. **Convention-based**: `/agents` + GET → `list_agents` (auto-generate)
2. **Explicit mapping**: Maintain a map in `api.ts`
3. **Code generation**: Generate both Axum routes and Tauri commands from a single spec

Leaning toward **explicit mapping** for now—it's simple and transparent.

---



### 8. Tauri and ckrv-ui share the same frontend
**Context**: Tauri is a container, not a separate frontend.

**Key insight**: There is no duplication to maintain. Tauri wraps the exact same React frontend that `ckrv ui` serves via Axum. Both use:
- `crates/ckrv-ui/frontend/` (React + shadcn/ui)
- Same components, same styling, same behavior

The only difference is the transport layer:
- **Web (`ckrv ui`)**: HTTP fetch → Axum → ckrv-core
- **Desktop (Tauri)**: Tauri IPC → ckrv-tauri → ckrv-core

**No sunset consideration needed**—both are valid distribution mechanisms for the same UI.

### 8. Complete API Audit: Axum → Tauri Command Mapping
**Context**: The spec only defines basic commands. Full audit reveals complete API surface.

#### Existing ckrv-ui API Modules → Tauri Commands

**Decision**: All API modules will be supported for full parity.

| Module | Description | Tauri Status |
|--------|-------------|--------------|
| `agents.rs` | Agent configuration CRUD | ✅ Will implement |
| `cloud.rs` | Cloud connection status | ✅ Will implement |
| `commands.rs` | CLI command execution | ✅ Will implement |
| `console.rs` | Interactive command console | ✅ Will implement |
| `diff.rs` | Git diff viewing | ✅ Will implement |
| `docker.rs` | Docker status checks | ✅ Will implement |
| `events.rs` | Server-Sent Events stream | ✅ Will implement (Tauri events) |
| `execution.rs` | Batch execution control | ✅ In spec |
| `history.rs` | Run history management | ✅ Will implement |
| `plans.rs` | Execution plan management | ✅ Will implement |
| `qa.rs` | QA command handlers | ✅ Will implement |
| `session.rs` | Docker session management | ✅ Will implement |
| `specs.rs` | Specification CRUD | ✅ In spec |
| `status.rs` | System status endpoint | ✅ Will implement |
| `tasks.rs` | Task management | ✅ Will implement |
| `terminal.rs` | Interactive terminal WebSocket | ✅ Will implement (Tauri PTY) |
| `test.rs` | Test command handlers | ✅ Will implement |

#### Agent Commands (from `agents.rs`)

| Axum Handler | Tauri Command | Status |
|--------------|---------------|--------|
| `list_agents` | `list_agents` | ✅ Will implement |
| `upsert_agent` | `upsert_agent` | ✅ Will implement |
| `delete_agent` | `delete_agent` | ✅ Will implement |
| `set_default_agent` | `set_default_agent` | ✅ Will implement |
| `set_qa_agent` | `set_qa_agent` | ✅ Will implement |
| `set_test_writer_agent` | `set_test_writer_agent` | ✅ Will implement |
| `test_agent` | `test_agent` | ✅ Will implement |
| `get_openrouter_models` | `get_openrouter_models` | ✅ Will implement |

#### Implementation Notes

**Events (SSE → Tauri Events)**:
- Axum uses Server-Sent Events for real-time updates
- Tauri will use `tauri::Manager::emit()` for the same functionality
- Frontend adapter will abstract the difference

**Terminal (WebSocket → Tauri PTY)**:
- Axum uses WebSocket for terminal I/O
- Tauri will use `tauri-plugin-shell` or custom PTY solution
- xterm.js frontend remains the same
---

## Implementation Concerns

### 1. Frontend Path Migration
The spec has frontend at `web/` but our actual structure is `crates/ckrv-ui/frontend/`.

**Action**: Update `tauri.conf.json` paths:
```json
{
  "build": {
    "beforeBuildCommand": "cd ../../crates/ckrv-ui/frontend && npm run build",
    "frontendDist": "../../crates/ckrv-ui/frontend/dist",
    "devUrl": "http://localhost:5173"
  }
}
```

### 2. xterm.js in Tauri WebView
**Concern**: Terminal rendering currently works via xterm.js. Will it work in Tauri's WebView?

**Assessment**: Yes—Tauri uses system WebView (WebKit on macOS, WebView2 on Windows, WebKitGTK on Linux). xterm.js should work identically.

**Testing needed**: Verify terminal themes, special characters, and scrolling behavior.

### 3. Docker Access from Tauri
**Concern**: Tauri commands need to spawn Docker containers.

**Assessment**: No issue—Tauri's Rust backend can call Docker CLI the same way `ckrv-core` does. Unlike Electron, there's no IPC overhead.

### 4. Development Workflow
**Question**: How does `cargo tauri dev` interact with hot reload?

**Answer**: Tauri dev mode:
1. Runs `beforeDevCommand` (starts Vite dev server)
2. Opens WebView pointing to `devUrl`
3. Hot module replacement works normally
4. Rust changes require restart (Tauri watches and rebuilds)

---

## Success Criteria

| Metric | Target |
|--------|--------|
| Installer size (macOS) | < 20MB |
| Bundle size (Windows) | < 25MB |
| Cold start time | < 2 seconds |
| Memory usage (idle) | < 50MB |
| All existing UI features | Functional parity |
| Platform coverage | macOS, Windows, Linux |

---

## Workflow Parity: Tauri ↔ ckrv-ui

Since Tauri and `ckrv ui` share the same React frontend but have different transport layers (Tauri IPC vs HTTP), we need workflows and conventions to ensure parity is maintained as the codebase evolves.

### Important: Mutually Exclusive at Runtime

**Tauri replaces Axum one-to-one**—they never run simultaneously:

| Distribution | What's Running | Frontend Calls |
|--------------|----------------|----------------|
| **Desktop (Tauri)** | Tauri app only | `invoke('list_agents')` → Tauri command → ckrv-core |
| **Web (`ckrv ui`)** | Axum server only | `fetch('/api/agents')` → Axum handler → ckrv-core |

When you launch the Tauri desktop app, **Axum is not running**. Tauri commands are the sole backend.

When you run `ckrv ui`, **Tauri is not involved**. Axum serves everything.

The parity concern is about **keeping both backends in sync during development** so users get the same features regardless of which distribution they use.

```
┌──────────────────────────────────────────────────────────────────┐
│                     React Frontend                                │
│                (crates/ckrv-ui/frontend)                         │
│                                                                  │
│         api.ts detects: IS_TAURI ? invoke() : fetch()           │
└────────────────────────┬─────────────────────────────────────────┘
                         │
          ┌──────────────┴──────────────┐
          │                             │
          ▼                             ▼
┌─────────────────────┐     ┌─────────────────────┐
│   Tauri Desktop     │     │   Web (ckrv ui)     │
│                     │     │                     │
│  invoke('cmd')      │     │  fetch('/api/cmd')  │
│       │             │     │       │             │
│       ▼             │     │       ▼             │
│  ckrv-tauri/        │     │  ckrv-ui/api/       │
│  commands/*.rs      │     │  *.rs (Axum)        │
│       │             │     │       │             │
│       └─────────────┼─────┼───────┘             │
│                     │     │                     │
│              ckrv-core (shared)                 │
└─────────────────────┘     └─────────────────────┘
```

### The Parity Principle

```
┌─────────────────────────────────────────────────────────────────┐
│                     React Frontend                               │
│                 (crates/ckrv-ui/frontend)                       │
│  ┌───────────────────────────────────────────────────────────┐  │
│  │                    api.ts Adapter                          │  │
│  │   IS_TAURI ? invoke('cmd', args) : fetch('/api/cmd')      │  │
│  └───────────────────┬───────────────────┬───────────────────┘  │
└──────────────────────┼───────────────────┼──────────────────────┘
                       │                   │
         ┌─────────────▼─────┐   ┌─────────▼─────────┐
         │   ckrv-tauri      │   │   ckrv-ui (Axum)  │
         │   commands/*.rs   │   │   api/*.rs        │
         └─────────┬─────────┘   └─────────┬─────────┘
                   │                       │
                   └───────────┬───────────┘
                               │
                   ┌───────────▼───────────┐
                   │      ckrv-core        │
                   │   (shared logic)      │
                   └───────────────────────┘
```

**Key insight**: The frontend adapter (`api.ts`) is the **single point of truth** for API contracts. Both Tauri commands and Axum handlers must implement the same interface.

---

### Required Convention Updates

#### 1. API Contract Convention

**New rule**: Every new API endpoint must be added in **three places**:

| File | Purpose |
|------|---------|
| `crates/ckrv-ui/src/api/*.rs` | Axum HTTP handler |
| `crates/ckrv-tauri/src/commands/*.rs` | Tauri IPC command |
| `crates/ckrv-ui/frontend/src/lib/api.ts` | Frontend adapter |

**Enforcement**: Add a checklist to PR templates:
```markdown
## API Changes Checklist
- [ ] Added Axum handler in `crates/ckrv-ui/src/api/`
- [ ] Added Tauri command in `crates/ckrv-tauri/src/commands/`
- [ ] Added frontend adapter in `crates/ckrv-ui/frontend/src/lib/api.ts`
- [ ] Verified both web and desktop builds work
```

#### 2. Shared Types Convention

**New rule**: Request/response types should be defined in `ckrv-types` or a shared module, not duplicated.

```
crates/ckrv-types/src/
├── api/           # NEW: Shared API types
│   ├── mod.rs
│   ├── agents.rs  # AgentConfig, UpsertAgentPayload, etc.
│   ├── specs.rs   # Spec, CreateSpecRequest, etc.
│   └── ...
```

**Why**: Prevents drift between Axum and Tauri implementations.

#### 3. Command Naming Convention

**New rule**: Tauri commands must use snake_case matching the Axum handler names:

| Axum Handler | Tauri Command | ✓/✗ |
|--------------|---------------|-----|
| `list_agents` | `list_agents` | ✓ |
| `listAgents` | `list_agents` | ✗ (inconsistent) |

**Frontend adapter** handles any camelCase conversion if needed.

---

### Development Workflow Changes

#### 1. Testing Both Backends

Add npm scripts to test both modes:

```json
// crates/ckrv-ui/frontend/package.json
{
  "scripts": {
    "dev": "vite",
    "dev:web": "VITE_BACKEND=web vite",
    "dev:tauri": "VITE_BACKEND=tauri vite",
    "test:parity": "npm run test:web && npm run test:tauri"
  }
}
```

**CI requirement**: Both modes must be tested before merge.

#### 2. Hot Reload Development

| Mode | Command | Backend |
|------|---------|---------|
| Web development | `ckrv ui` + `npm run dev` | Axum |
| Tauri development | `cargo tauri dev` | Tauri IPC |
| Frontend-only | `npm run dev` | Mock/none |

**Recommendation**: Default to `cargo tauri dev` for primary development once Tauri is stable.

#### 3. Feature Flag Pattern

For features that need different behavior:

```typescript
// api.ts
export async function openFileDialog(): Promise<string | null> {
  if (IS_TAURI) {
    // Native file dialog via Tauri plugin
    const { open } = await import('@tauri-apps/plugin-dialog');
    return open({ directory: true });
  }
  // Web: HTML5 file input or not supported
  return null;
}
```

**Convention**: Feature differences should be isolated in `api.ts`, not scattered across components.

---

### Documentation Requirements

#### 1. Update AGENTS.md

Add Tauri-specific guidance:

```markdown
## Working with the UI

### Distribution Modes

| Mode | When to Use |
|------|-------------|
| Desktop (Tauri) | Primary end-user experience |
| Web (`ckrv ui`) | Remote servers, CLI users |

### Adding New API Endpoints

1. Add Axum handler in `crates/ckrv-ui/src/api/`
2. Add Tauri command in `crates/ckrv-tauri/src/commands/`
3. Add frontend adapter in `crates/ckrv-ui/frontend/src/lib/api.ts`
4. Test both modes before submitting PR
```

#### 2. Create API Parity Checklist

`crates/docs/api-parity-checklist.md`:

```markdown
# API Parity Checklist

Use this checklist when adding or modifying API endpoints.

## New Endpoint Checklist
- [ ] Define types in `ckrv-types` (if shared)
- [ ] Implement Axum handler
- [ ] Implement Tauri command
- [ ] Update `api.ts` adapter
- [ ] Add tests for both backends
- [ ] Update API documentation

## Modification Checklist
- [ ] Update Axum handler
- [ ] Update Tauri command
- [ ] Update `api.ts` adapter
- [ ] Verify type compatibility
- [ ] Run parity tests
```

#### 3. Update README

Add distribution section:

```markdown
## Installation

### Desktop App (Recommended)
Download from [Releases](link) - available for macOS, Windows, Linux.

### Via Cargo (CLI + Web UI)
```bash
cargo install chakravarti-cli
ckrv ui  # Opens web interface
```
```

---

### CI/CD Parity Enforcement

#### 1. Build Both Targets

```yaml
# .github/workflows/build.yml
jobs:
  build-web:
    runs-on: ubuntu-latest
    steps:
      - run: cargo build --package ckrv-ui

  build-tauri:
    strategy:
      matrix:
        os: [ubuntu-latest, macos-latest, windows-latest]
    runs-on: ${{ matrix.os }}
    steps:
      - run: cargo tauri build
```

#### 2. API Parity Test

```yaml
  parity-test:
    runs-on: ubuntu-latest
    steps:
      - name: Start Axum backend
        run: cargo run --package ckrv-ui &
      
      - name: Run frontend tests (web mode)
        run: npm run test:web
      
      - name: Run frontend tests (tauri mode)
        run: npm run test:tauri
```

#### 3. Automated Parity Report

Script to detect drift:

```bash
#!/bin/bash
# scripts/check-api-parity.sh

# Extract Axum handlers
axum_handlers=$(grep -h "pub async fn" crates/ckrv-ui/src/api/*.rs | wc -l)

# Extract Tauri commands
tauri_commands=$(grep -h "#\[tauri::command\]" crates/ckrv-tauri/src/commands/*.rs | wc -l)

# Compare
if [ "$axum_handlers" -ne "$tauri_commands" ]; then
  echo "⚠️  API Parity Warning: $axum_handlers Axum handlers vs $tauri_commands Tauri commands"
  exit 1
fi
```

---

### Migration Strategy

#### Phase 1: Foundation
1. Create `api.ts` adapter with `IS_TAURI` detection
2. Migrate existing `fetch()` calls to use adapter
3. Verify web mode still works

#### Phase 2: Tauri Commands
1. Implement Tauri commands matching Axum handlers
2. Test each endpoint in Tauri mode
3. Document any behavioral differences

#### Phase 3: Parity Enforcement
1. Add CI checks for both modes
2. Add parity test suite
3. Update AGENTS.md with conventions
4. Create PR template checklist

---

## Implementation Phases (Updated for Full Parity)

### Phase 1: Setup (Day 1)
- [ ] Create `crates/ckrv-tauri` directory structure
- [ ] Add Cargo.toml and tauri.conf.json (with correct frontend paths)
- [ ] Create stub `main.rs` that opens empty window
- [ ] Verify `cargo tauri dev` works

### Phase 2: Core Commands (Day 1-2)
- [ ] Implement state.rs with AppState
- [ ] Port spec commands: `create_spec`, `load_spec`, `save_spec`, `generate_tasks`
- [ ] Port execution commands: `run_plan`, `get_status`, `cancel_execution`, `get_task_output`
- [ ] Port config commands: `get_config`, `update_config`, `get_available_models`
- [ ] Port project commands: `open_project`, `get_project_info`, `list_worktrees`

### Phase 3: Agent & Status Commands (Day 2-3)
- [ ] Port all agent commands (8 endpoints):
  - `list_agents`, `upsert_agent`, `delete_agent`, `set_default_agent`
  - `set_qa_agent`, `set_test_writer_agent`, `test_agent`, `get_openrouter_models`
- [ ] Port status commands: `get_system_status`, `get_docker_status`
- [ ] Port history commands: `list_runs`, `get_run`, `delete_run`

### Phase 4: Full Feature Commands (Day 3-4)
- [ ] Port plans commands: `list_plans`, `get_plan`, `create_plan`, `delete_plan`
- [ ] Port tasks commands: `list_tasks`, `get_task`, `update_task_status`
- [ ] Port diff commands: `get_diff`, `get_file_diff`
- [ ] Port QA commands: `run_qa_review`, `get_qa_report`
- [ ] Port test commands: `run_tests`, `get_test_results`, `write_tests`
- [ ] Port cloud commands: `get_cloud_status`, `connect_cloud`

### Phase 5: Real-time Features (Day 4-5)
- [ ] Implement Tauri events to replace SSE:
  - `execution_progress`, `task_complete`, `agent_status`
- [ ] Implement terminal support:
  - Evaluate `tauri-plugin-shell` vs custom PTY
  - Connect xterm.js to Tauri PTY
- [ ] Implement console/commands execution

### Phase 6: Frontend Adapter (Day 5-6)
- [ ] Create unified API layer (`api.ts`) with all endpoints
- [ ] Add @tauri-apps dependencies
- [ ] Update all components to use api.* functions
- [ ] Test in browser (HTTP mode still works)
- [ ] Test in Tauri (invoke mode works)

### Phase 7: Polish & CI (Day 6-7)
- [ ] Add app icons (all sizes for all platforms)
- [ ] Configure bundle metadata
- [ ] Test on Windows, macOS, Linux
- [ ] Set up CI/CD for releases
- [ ] Add parity check script to CI
- [ ] Update AGENTS.md with Tauri conventions

---

## Next Steps

1. [x] **Complete API audit** ✅ (all 17 modules identified)
2. [x] **Decision: Full parity** ✅ (all modules will be implemented)
3. [x] **Consolidate spec into brainstorm** ✅ (all code specs below)
4. [ ] **Begin Phase 1** (scaffold crate and verify dev workflow)

---

## Code Specifications

### File Structure

```
crates/ckrv-tauri/
├── Cargo.toml
├── build.rs
├── tauri.conf.json
├── capabilities/
│   └── default.json
├── icons/
│   ├── icon.ico          # Windows
│   ├── icon.icns         # macOS
│   ├── icon.png          # Linux (512x512)
│   ├── 32x32.png
│   ├── 128x128.png
│   └── 128x128@2x.png
└── src/
    ├── main.rs           # Entry point + Tauri setup
    ├── commands/
    │   ├── mod.rs
    │   ├── agents.rs     # Agent management commands
    │   ├── spec.rs       # Spec-related commands
    │   ├── execution.rs  # Run/status commands
    │   ├── config.rs     # Settings commands
    │   ├── project.rs    # Project commands
    │   ├── history.rs    # Run history commands
    │   ├── plans.rs      # Plan management
    │   ├── tasks.rs      # Task management
    │   ├── diff.rs       # Git diff viewing
    │   ├── qa.rs         # QA commands
    │   ├── test.rs       # Test commands
    │   ├── cloud.rs      # Cloud status
    │   ├── docker.rs     # Docker status
    │   ├── status.rs     # System status
    │   ├── terminal.rs   # PTY/terminal
    │   └── events.rs     # Tauri event emitters
    └── state.rs          # App state management

crates/ckrv-ui/frontend/src/lib/
└── api.ts                # API adapter (Tauri/HTTP switch)
```

---

### 1. Cargo Configuration

**`crates/ckrv-tauri/Cargo.toml`**

```toml
[package]
name = "ckrv-tauri"
version = "0.1.0"
edition = "2021"
description = "Chakravarti-cli desktop application"
default-run = "ckrv-tauri"

[build-dependencies]
tauri-build = { version = "2", features = [] }

[dependencies]
tauri = { version = "2", features = [] }
tauri-plugin-shell = "2"
tauri-plugin-dialog = "2"
tauri-plugin-fs = "2"
serde = { version = "1", features = ["derive"] }
serde_json = "1"
serde_yaml = "0.9"
tokio = { version = "1", features = ["full"] }
thiserror = "1"
tracing = "0.1"

# Internal crates
ckrv-core = { path = "../ckrv-core" }
ckrv-types = { path = "../ckrv-types" }
ckrv-executor = { path = "../ckrv-executor" }

[features]
default = ["custom-protocol"]
custom-protocol = ["tauri/custom-protocol"]
```

**`crates/ckrv-tauri/build.rs`**

```rust
fn main() {
    tauri_build::build()
}
```

---

### 2. Tauri Configuration

**`crates/ckrv-tauri/tauri.conf.json`**

```json
{
  "$schema": "https://schema.tauri.app/config/2",
  "productName": "Chakravarti-cli",
  "identifier": "dev.ckrv.app",
  "version": "0.1.0",
  "build": {
    "beforeBuildCommand": "cd ../../crates/ckrv-ui/frontend && npm run build",
    "beforeDevCommand": "cd ../../crates/ckrv-ui/frontend && npm run dev",
    "frontendDist": "../../crates/ckrv-ui/frontend/dist",
    "devUrl": "http://localhost:5173"
  },
  "bundle": {
    "active": true,
    "targets": "all",
    "icon": [
      "icons/32x32.png",
      "icons/128x128.png",
      "icons/128x128@2x.png",
      "icons/icon.icns",
      "icons/icon.ico"
    ],
    "resources": [],
    "category": "DeveloperTool",
    "shortDescription": "AI Code Orchestration",
    "longDescription": "Coordinate multiple AI coding agents working in parallel on separate Git worktrees.",
    "windows": {
      "certificateThumbprint": null,
      "digestAlgorithm": "sha256",
      "timestampUrl": ""
    },
    "macOS": {
      "minimumSystemVersion": "10.15"
    },
    "linux": {
      "desktop": {
        "categories": ["Development", "IDE"]
      }
    }
  },
  "app": {
    "windows": [
      {
        "title": "Chakravarti-cli",
        "width": 1280,
        "height": 800,
        "minWidth": 800,
        "minHeight": 600,
        "resizable": true,
        "fullscreen": false,
        "center": true
      }
    ],
    "security": {
      "csp": null
    }
  },
  "plugins": {
    "shell": {
      "open": true
    }
  }
}
```

**`crates/ckrv-tauri/capabilities/default.json`**

```json
{
  "$schema": "https://schema.tauri.app/config/2",
  "identifier": "default",
  "description": "Default capabilities for CKRV",
  "windows": ["main"],
  "permissions": [
    "core:default",
    "shell:allow-open",
    "dialog:allow-open",
    "dialog:allow-save",
    "fs:allow-read-text-file",
    "fs:allow-write-text-file"
  ]
}
```

---

### 3. Rust Implementation

**`crates/ckrv-tauri/src/main.rs`**

```rust
#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

mod commands;
mod state;

use state::AppState;
use std::sync::Arc;
use tauri::Manager;
use tokio::sync::Mutex;
use tracing_subscriber;

fn main() {
    tracing_subscriber::fmt::init();

    let state = AppState::new().expect("Failed to initialize app state");

    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .manage(Arc::new(Mutex::new(state)))
        .invoke_handler(tauri::generate_handler![
            // Spec commands
            commands::spec::create_spec,
            commands::spec::load_spec,
            commands::spec::save_spec,
            commands::spec::generate_tasks,
            // Execution commands
            commands::execution::run_plan,
            commands::execution::get_status,
            commands::execution::cancel_execution,
            commands::execution::get_task_output,
            // Config commands
            commands::config::get_config,
            commands::config::update_config,
            commands::config::get_available_models,
            // Project commands
            commands::project::open_project,
            commands::project::get_project_info,
            commands::project::list_worktrees,
            // Agent commands
            commands::agents::list_agents,
            commands::agents::upsert_agent,
            commands::agents::delete_agent,
            commands::agents::set_default_agent,
            commands::agents::set_qa_agent,
            commands::agents::set_test_writer_agent,
            commands::agents::test_agent,
            commands::agents::get_openrouter_models,
            // Status commands
            commands::status::get_system_status,
            commands::docker::get_docker_status,
            // History commands
            commands::history::list_runs,
            commands::history::get_run,
            commands::history::delete_run,
            // And more... (add as implemented)
        ])
        .setup(|app| {
            #[cfg(debug_assertions)]
            {
                let window = app.get_webview_window("main").unwrap();
                window.open_devtools();
            }
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
```

**`crates/ckrv-tauri/src/state.rs`**

```rust
use ckrv_core::{Config, Orchestrator};
use ckrv_executor::LocalExecutor;
use std::path::PathBuf;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum StateError {
    #[error("Failed to initialize orchestrator: {0}")]
    OrchestratorInit(String),
    #[error("No project loaded")]
    NoProject,
}

pub struct AppState {
    pub orchestrator: Orchestrator,
    pub config: Config,
    pub current_project: Option<PathBuf>,
}

impl AppState {
    pub fn new() -> Result<Self, StateError> {
        let config = Config::load_or_default()
            .map_err(|e| StateError::OrchestratorInit(e.to_string()))?;
        
        let executor = Box::new(LocalExecutor::new(&config.execution.local)?);
        let orchestrator = Orchestrator::new(executor, config.clone())
            .map_err(|e| StateError::OrchestratorInit(e.to_string()))?;

        Ok(Self {
            orchestrator,
            config,
            current_project: None,
        })
    }

    pub fn project_path(&self) -> Result<&PathBuf, StateError> {
        self.current_project.as_ref().ok_or(StateError::NoProject)
    }
}
```

**`crates/ckrv-tauri/src/commands/mod.rs`**

```rust
pub mod agents;
pub mod config;
pub mod docker;
pub mod execution;
pub mod history;
pub mod project;
pub mod spec;
pub mod status;
// Add more as implemented

use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Error, Debug, Serialize, Deserialize)]
pub enum CommandError {
    #[error("Invalid input: {0}")]
    InvalidInput(String),
    #[error("Execution failed: {0}")]
    ExecutionFailed(String),
    #[error("Not found: {0}")]
    NotFound(String),
    #[error("Internal error: {0}")]
    Internal(String),
}

impl From<CommandError> for String {
    fn from(err: CommandError) -> String {
        err.to_string()
    }
}
```

---

### 4. Frontend API Adapter

**`crates/ckrv-ui/frontend/src/lib/api.ts`**

```typescript
const IS_TAURI = typeof window !== 'undefined' && '__TAURI__' in window;

type InvokeFn = <T>(cmd: string, args?: Record<string, unknown>) => Promise<T>;

let invoke: InvokeFn | null = null;

async function getInvoke(): Promise<InvokeFn | null> {
  if (!IS_TAURI) return null;
  if (invoke) return invoke;
  
  const tauri = await import('@tauri-apps/api/core');
  invoke = tauri.invoke;
  return invoke;
}

async function tauriInvoke<T>(cmd: string, args?: Record<string, unknown>): Promise<T> {
  const inv = await getInvoke();
  if (!inv) throw new Error('Tauri not available');
  return inv<T>(cmd, args);
}

async function httpFetch<T>(
  endpoint: string,
  options?: RequestInit
): Promise<T> {
  const res = await fetch(`/api${endpoint}`, {
    headers: { 'Content-Type': 'application/json' },
    ...options,
  });
  if (!res.ok) {
    const error = await res.text();
    throw new Error(error);
  }
  return res.json();
}

// =============================================================================
// Agent Commands
// =============================================================================

export async function listAgents() {
  if (IS_TAURI) {
    return tauriInvoke('list_agents');
  }
  return httpFetch('/agents');
}

export async function upsertAgent(agent: AgentConfig) {
  if (IS_TAURI) {
    return tauriInvoke('upsert_agent', { agent });
  }
  return httpFetch('/agents', {
    method: 'POST',
    body: JSON.stringify({ agent }),
  });
}

// ... (add all other commands following the same pattern)

// =============================================================================
// Utilities
// =============================================================================

export function isTauri(): boolean {
  return IS_TAURI;
}

export async function openFileDialog(): Promise<string | null> {
  if (IS_TAURI) {
    const { open } = await import('@tauri-apps/plugin-dialog');
    const selected = await open({
      directory: true,
      multiple: false,
      title: 'Select Project Directory',
    });
    return selected as string | null;
  }
  return null;
}
```

---

### 5. Frontend Package Updates

**`crates/ckrv-ui/frontend/package.json`** (additions)

```json
{
  "dependencies": {
    "@tauri-apps/api": "^2.0.0",
    "@tauri-apps/plugin-dialog": "^2.0.0",
    "@tauri-apps/plugin-fs": "^2.0.0",
    "@tauri-apps/plugin-shell": "^2.0.0"
  },
  "scripts": {
    "tauri": "tauri",
    "tauri:dev": "tauri dev",
    "tauri:build": "tauri build"
  }
}
```

---

## Build & Distribution

### Development

```bash
# From repo root
cd crates/ckrv-tauri
cargo tauri dev
```

### Production Build

```bash
# Build for current platform
cargo tauri build

# Cross-compile (requires additional setup)
cargo tauri build --target x86_64-pc-windows-msvc
cargo tauri build --target x86_64-apple-darwin
cargo tauri build --target x86_64-unknown-linux-gnu
```

### Output Locations

| Platform | Output Path | Format |
|----------|-------------|--------|
| Windows | `target/release/bundle/msi/` | `.msi` installer |
| macOS | `target/release/bundle/dmg/` | `.dmg` disk image |
| Linux | `target/release/bundle/deb/` | `.deb` package |
| Linux | `target/release/bundle/appimage/` | `.AppImage` |

---

## CI/CD Pipeline

**`.github/workflows/release-desktop.yml`**

```yaml
name: Release Desktop

on:
  push:
    tags:
      - 'v*'

jobs:
  build:
    strategy:
      matrix:
        include:
          - os: ubuntu-latest
            target: x86_64-unknown-linux-gnu
          - os: macos-latest
            target: x86_64-apple-darwin
          - os: macos-latest
            target: aarch64-apple-darwin
          - os: windows-latest
            target: x86_64-pc-windows-msvc

    runs-on: ${{ matrix.os }}

    steps:
      - uses: actions/checkout@v4

      - name: Install Rust
        uses: dtolnay/rust-action@stable
        with:
          targets: ${{ matrix.target }}

      - name: Install dependencies (Linux)
        if: matrix.os == 'ubuntu-latest'
        run: |
          sudo apt-get update
          sudo apt-get install -y libwebkit2gtk-4.1-dev libappindicator3-dev librsvg2-dev

      - name: Setup Node
        uses: actions/setup-node@v4
        with:
          node-version: 20

      - name: Install frontend deps
        run: cd crates/ckrv-ui/frontend && npm ci

      - name: Build
        run: |
          cd crates/ckrv-tauri
          cargo tauri build --target ${{ matrix.target }}

      - name: Upload artifacts
        uses: actions/upload-artifact@v4
        with:
          name: ckrv-${{ matrix.target }}
          path: |
            target/${{ matrix.target }}/release/bundle/**/*.dmg
            target/${{ matrix.target }}/release/bundle/**/*.msi
            target/${{ matrix.target }}/release/bundle/**/*.deb
            target/${{ matrix.target }}/release/bundle/**/*.AppImage
```

---

## Testing Requirements

### Unit Tests

```rust
// crates/ckrv-tauri/src/commands/spec.rs
#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_create_and_load_spec() {
        let temp = TempDir::new().unwrap();
        let path = temp.path().join("spec.yaml");
        
        let spec = Spec::new("test".into(), "Test spec".into());
        save_spec(path.to_string_lossy().into(), spec.clone())
            .await
            .unwrap();
        
        let loaded = load_spec(path.to_string_lossy().into())
            .await
            .unwrap();
        assert_eq!(loaded.name, spec.name);
    }
}
```

### E2E Tests

```typescript
// tests/e2e/workflow.spec.ts
import { test, expect } from '@playwright/test';

test('can create and run spec', async ({ page }) => {
  await page.goto('/');
  await page.click('[data-testid="new-spec"]');
  await page.fill('[data-testid="spec-name"]', 'Test Spec');
  await page.click('[data-testid="generate-tasks"]');
  await expect(page.locator('[data-testid="task-list"]')).toBeVisible();
});
```

---

## References

- [Tauri v2 Documentation](https://v2.tauri.app)
- [Tauri Plugin System](https://v2.tauri.app/develop/plugins/)
- [Tauri IPC](https://v2.tauri.app/develop/calling-rust/)
- [Tauri GitHub Actions](https://github.com/tauri-apps/tauri-action)
- [Vision Document](../../guiding_docs/vision.md) - Product principles alignment

---

## 9. Frontend Restructuring Proposal ✅ NEW

**Date Added**: 2026-02-04
**Status**: Proposed - Pending Implementation

### Problem

Currently, the React frontend lives inside `crates/ckrv-ui/frontend/`. This creates an issue for Tauri:

```
crates/ckrv-ui/
├── src/                    # Rust - Axum server
└── frontend/               # React app (bundled into Rust via rust-embed)

crates/ckrv-tauri/          # How does this access the frontend?
└── src-tauri/
```

**Options considered**:
1. **Symlink** - Fragile, doesn't work on Windows
2. **Copy at build time** - Duplicates artifacts, confusing
3. **Move to ckrv-transport** - Wrong abstraction level
4. **Separate ckrv-frontend folder** ← RECOMMENDED

### Proposed Structure

Move the frontend to a standalone location that both `ckrv-ui` and `ckrv-tauri` can reference:

```
crates/
├── ckrv-frontend/              # ← NEW: Standalone React app
│   ├── package.json
│   ├── vite.config.ts
│   ├── tsconfig.json
│   ├── index.html
│   └── src/
│       ├── App.tsx
│       ├── main.tsx
│       ├── components/
│       │   ├── ui/             # shadcn/ui components
│       │   └── ...
│       ├── pages/
│       │   ├── SpecsPage.tsx
│       │   ├── AgentsPage.tsx
│       │   └── ...
│       ├── hooks/
│       ├── lib/
│       │   ├── api.ts          # Unified API layer
│       │   └── transport/
│       │       ├── http.ts     # Axum transport (fetch)
│       │       ├── tauri.ts    # Tauri transport (invoke)
│       │       └── mock.ts     # Test transport
│       └── types/
│           └── api.generated.ts  # Generated from ckrv-transport
│
├── ckrv-ui/                    # Web server (Axum)
│   └── src/
│       └── server.rs           # rust-embed: "../ckrv-frontend/dist"
│
├── ckrv-tauri/                 # Desktop app (future)
│   ├── src-tauri/
│   │   ├── Cargo.toml
│   │   ├── tauri.conf.json     # frontendDist: "../ckrv-frontend/dist"
│   │   └── src/
│   │       └── main.rs
│   └── (no src/ - uses ckrv-frontend)
│
└── ckrv-transport/             # API layer (unchanged)
    └── src/
        ├── handlers/           # Shared business logic
        ├── axum/               # Axum wrappers
        ├── tauri/              # Tauri wrappers
        └── types/              # With ts-rs for TS generation
```

### How Each Consumer Uses the Frontend

**ckrv-ui (Axum)**:
```rust
// crates/ckrv-ui/src/server.rs
use rust_embed::RustEmbed;

#[derive(RustEmbed)]
#[folder = "../ckrv-frontend/dist"]  // Points to shared frontend
struct FrontendAssets;
```

**ckrv-tauri (Desktop)**:
```json
// crates/ckrv-tauri/src-tauri/tauri.conf.json
{
  "build": {
    "beforeBuildCommand": "cd ../ckrv-frontend && npm run build",
    "beforeDevCommand": "cd ../ckrv-frontend && npm run dev",
    "frontendDist": "../ckrv-frontend/dist",
    "devUrl": "http://localhost:5173"
  }
}
```

### Build Workflow

```bash
# Development (web)
cd crates/ckrv-frontend && npm run dev
cd crates/ckrv-ui && cargo run         # Proxies to Vite dev server

# Development (desktop)
cd crates/ckrv-tauri && cargo tauri dev  # Runs beforeDevCommand automatically

# Production (web)
cd crates/ckrv-frontend && npm run build
cd crates/ckrv-ui && cargo build --release

# Production (desktop)
cd crates/ckrv-tauri && cargo tauri build  # Runs beforeBuildCommand automatically
```

### TypeScript Type Generation

Types flow from Rust to TypeScript:

```
ckrv-transport/src/types/*.rs
        │
        │ [cargo test -p ckrv-transport --features typescript]
        │ [ts-rs generates types]
        ▼
ckrv-frontend/src/types/api.generated.ts
```

The `generate:types` npm script in `ckrv-frontend/package.json`:
```json
{
  "scripts": {
    "generate:types": "cd ../ckrv-transport && cargo test --features typescript export_typescript_types -- --ignored"
  }
}
```

### Migration Steps

1. **Create `crates/ckrv-frontend/`** as a new directory
2. **Move `crates/ckrv-ui/frontend/*`** → `crates/ckrv-frontend/`
3. **Update rust-embed path** in `ckrv-ui/src/server.rs`:
   ```rust
   #[folder = "../ckrv-frontend/dist"]
   ```
4. **Update README** and scripts to reflect new location
5. **Update type generation path** in export_types.rs:
   ```rust
   .join("ckrv-frontend/src/types/api.generated.ts")
   ```
6. **Test both web and desktop builds**

### Benefits

| Aspect | Before (ckrv-ui/frontend) | After (ckrv-frontend) |
|--------|--------------------------|----------------------|
| **Ownership** | Tied to Axum server | Standalone, shared |
| **Tauri access** | Needs symlink/copy | Direct reference |
| **Type generation** | Path: `ckrv-ui/frontend/...` | Path: `ckrv-frontend/...` |
| **Build isolation** | Mixed with Rust crate | Clean separation |
| **Dev workflow** | `cd ckrv-ui/frontend` | `cd ckrv-frontend` |

### Architecture Diagram

```
                    ┌─────────────────┐
                    │   ckrv-core     │
                    └────────┬────────┘
                             │
                             ▼
                    ┌─────────────────┐
                    │ ckrv-transport  │  API handlers + types
                    └────────┬────────┘
                             │
              ┌──────────────┴──────────────┐
              │                             │
              ▼                             ▼
     ┌─────────────────┐          ┌─────────────────┐
     │    ckrv-ui      │          │  ckrv-tauri     │
     │  (Axum server)  │          │  (Desktop app)  │
     └────────┬────────┘          └────────┬────────┘
              │                             │
              │    ┌─────────────────┐      │
              └───►│  ckrv-frontend  │◄─────┘
                   │  (React app)    │
                   │  Shared by both │
                   └─────────────────┘
```

### Decision

**Recommendation**: Proceed with restructuring to `crates/ckrv-frontend/`

**Rationale**:
- Clean separation of concerns
- Standard pattern for multi-platform apps
- Enables Tauri without code duplication
- No runtime overhead (compile-time choice)

**Trade-offs**:
- One-time migration effort
- Path updates in multiple files
- Breaking change for anyone with custom scripts pointing to old path

