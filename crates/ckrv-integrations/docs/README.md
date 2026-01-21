---
last_commit: c1bb442
last_updated: 2026-01-21
related_files:
  - src/lib.rs
---

# ckrv-integrations

External service integrations for Chakravarti.

## Overview

This crate provides integrations with external services like GitHub for PR creation, issue tracking, and other workflows.

## Key Integrations

| Service | Features |
|---------|----------|
| GitHub | PR creation, issue sync, status checks |

## Usage

```rust
use ckrv_integrations::github::GitHubClient;

let client = GitHubClient::new(token)?;

// Create a pull request
client.create_pr(CreatePrRequest {
    base: "main",
    head: "feature-branch",
    title: "Add user auth",
    body: "Implements OAuth2 login",
})?;
```

## Module Structure

```
src/
├── github.rs    # GitHub API integration
└── lib.rs       # Re-exports
```

## Configuration

Integrations are configured via environment variables:
- `GITHUB_TOKEN`: Personal access token for GitHub API

## Dependencies

| Crate | Purpose |
|-------|---------|
| `octocrab` | GitHub API client |
| `tokio` | Async runtime |
