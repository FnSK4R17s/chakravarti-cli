---
last_commit: e74f093
last_updated: 2026-01-29
related_files:
  - src/lib.rs
  - src/error.rs
---

# ckrv-integrations

External service integrations for Chakravarti.

> [!WARNING]
> **This crate is a STUB.** The GitHub and GitLab modules are declared behind feature flags but the implementation files do not exist. Only the error type is currently available. This crate is not used anywhere in the codebase.

## Overview

This crate is intended to provide integrations with external services like GitHub and GitLab for PR creation, issue tracking, and other workflows. Currently it only contains error type definitions.

## Current State

| Component | Status |
|-----------|--------|
| `IntegrationError` | ✅ Implemented |
| `github` module | ⚠️ Feature-gated, **file does not exist** |
| `gitlab` module | ⚠️ Feature-gated, **file does not exist** |

## Key Types

| Type | Module | Purpose |
|------|--------|---------|
| `IntegrationError` | error.rs | API and auth errors |

## Module Structure

```
src/
├── lib.rs     # Feature-gated module declarations (15 lines)
└── error.rs   # IntegrationError enum (16 lines)
```

## Planned Features (Not Implemented)

When implemented, the following would be available:

```rust
// Requires `github` feature
#[cfg(feature = "github")]
pub mod github;

// Requires `gitlab` feature  
#[cfg(feature = "gitlab")]
pub mod gitlab;
```

## Dependencies

| Crate | Purpose |
|-------|---------|
| `thiserror` | Error derive macros |

## Future Work

To implement this crate:
1. Create `src/github.rs` with `GitHubClient`
2. Create `src/gitlab.rs` with `GitLabClient`
3. Add `octocrab` (GitHub) and GitLab API client dependencies
4. Wire into CLI commands like `ckrv pr` or `ckrv publish`
