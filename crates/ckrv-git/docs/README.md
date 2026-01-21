---
last_commit: c1bb442
last_updated: 2026-01-21
related_files:
  - src/lib.rs
  - src/worktree.rs
  - src/branch.rs
  - src/diff.rs
---

# ckrv-git

Git operations and worktree management for Chakravarti.

## Overview

This crate provides git functionality including worktree creation, diff generation, and branch management. Worktrees are central to Chakravarti's isolation strategy.

## Key Types

| Type | Purpose |
|------|---------|
| `Worktree` | Isolated git worktree for execution |
| `WorktreeManager` | Creates/manages worktrees |
| `Diff` | Generated diff between branches |
| `DiffGenerator` | Produces diffs |
| `BranchManager` | Branch operations |

## Why Worktrees?

Worktrees provide isolated execution:
- Each agent runs in its own worktree
- Main branch is never modified directly
- Changes can be reviewed before merge
- Easy cleanup on failure

## Usage

```rust
use ckrv_git::{WorktreeManager, DefaultWorktreeManager};

let manager = DefaultWorktreeManager::new(repo_root)?;

// Create an isolated worktree
let worktree = manager.create("feature-branch", "batch-1")?;
assert!(worktree.path.exists());

// Work in isolation...

// Cleanup
manager.remove(&worktree)?;
```

## Module Structure

```
src/
├── worktree.rs   # Worktree management
├── branch.rs     # Branch operations
├── diff.rs       # Diff generation
└── error.rs      # Error types
```

## Traits

### WorktreeManager

```rust
pub trait WorktreeManager {
    fn create(&self, branch: &str, name: &str) -> Result<Worktree>;
    fn remove(&self, worktree: &Worktree) -> Result<()>;
    fn list(&self) -> Result<Vec<Worktree>>;
}
```

### DiffGenerator

```rust
pub trait DiffGenerator {
    fn generate(&self, base: &str, head: &str) -> Result<Diff>;
    fn stat(&self, base: &str, head: &str) -> Result<DiffStat>;
}
```

## Dependencies

| Crate | Purpose |
|-------|---------|
| `git2` | Git operations |
| `thiserror` | Error handling |
