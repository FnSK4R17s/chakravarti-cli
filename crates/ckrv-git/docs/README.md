---
last_commit: f92f604
last_updated: 2026-03-01
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

| Type | Module | Purpose |
|------|--------|---------|
| `Worktree` | worktree.rs | Isolated git worktree for execution |
| `WorktreeStatus` | worktree.rs | Creating, Ready, InUse, Cleanup, Deleted |
| `WorktreeManager` | worktree.rs | Trait for creating/managing worktrees |
| `DefaultWorktreeManager` | worktree.rs | git2-based implementation |
| `Diff` | diff.rs | Generated diff with file changes |
| `FileDiff` | diff.rs | Per-file additions/deletions |
| `DiffStat` | diff.rs | Summary stats (files, ins, del) |
| `DiffGenerator` | diff.rs | Trait for producing diffs |
| `BranchManager` | branch.rs | Trait for branch operations |
| `GitBranchManager` | branch.rs | Command-line git implementation |
| `PromoteResult` | branch.rs | Result of promote operation |
| `GitError` | error.rs | Error types |

## Module Structure

```
src/
├── lib.rs        # Public exports + utility functions
├── worktree.rs   # Worktree management (11KB)
├── branch.rs     # Branch operations (7KB)
├── diff.rs       # Diff generation (7KB)
└── error.rs      # Error types
```

## Why Worktrees?

Worktrees provide isolated execution:
- Each agent runs in its own worktree
- Main branch is never modified directly
- Changes can be reviewed before merge
- Easy cleanup on failure

## Utility Functions

Exported from `lib.rs`:

```rust
// Check if path is in a git repo
pub fn is_git_repo(path: &Path) -> Result<bool, GitError>;

// Check if Chakravarti is initialized (.specs/ + .chakravarti/)
pub fn is_initialized(repo_root: &Path) -> bool;

// Get repository root from any path inside it
pub fn repo_root(path: &Path) -> Result<PathBuf, GitError>;
```

## Usage

### Worktree Management

```rust
use ckrv_git::{DefaultWorktreeManager, WorktreeManager};

let manager = DefaultWorktreeManager::new(repo_root)?;

// Create an isolated worktree (by job/attempt IDs)
let worktree = manager.create("job-001", "attempt-1")?;
assert!(worktree.path.exists());

// Get worktree path
let path = manager.path("job-001", "attempt-1")?;

// List all worktrees
let worktrees = manager.list()?;

// Cleanup when done
manager.cleanup(&worktree)?;
```

### Diff Generation

```rust
use ckrv_git::{DefaultDiffGenerator, DiffGenerator};

let generator = DefaultDiffGenerator::new();

// Generate diff from worktree changes
let diff = generator.diff(&worktree)?;
println!("Changed files: {:?}", diff.files);
println!("Raw diff:\n{}", diff.content);

// Get statistics
let stat = generator.diffstat(&worktree)?;
println!("{} files, +{} -{}", stat.files_changed, stat.insertions, stat.deletions);

// Or diff any path directly
let diff = generator.diff_path(Path::new("/path/to/repo"))?;
```

### Branch Management

```rust
use ckrv_git::{GitBranchManager, BranchManager};

let manager = GitBranchManager::new(repo_root);

// Check if branch exists
if !manager.exists("feature-branch") {
    // Create branch from worktree changes
    manager.create_from_worktree(&worktree, "feature-branch", false)?;
    
    // Push to remote
    manager.push("feature-branch", "origin", false)?;
}

// Delete branch (force = true for unmerged)
manager.delete("old-branch", true)?;
```

## Traits

### WorktreeManager

```rust
pub trait WorktreeManager: Send + Sync {
    fn create(&self, job_id: &str, attempt_id: &str) -> Result<Worktree, GitError>;
    fn path(&self, job_id: &str, attempt_id: &str) -> Result<PathBuf, GitError>;
    fn cleanup(&self, worktree: &Worktree) -> Result<(), GitError>;
    fn list(&self) -> Result<Vec<Worktree>, GitError>;
}
```

### DiffGenerator

```rust
pub trait DiffGenerator: Send + Sync {
    fn diff(&self, worktree: &Worktree) -> Result<Diff, GitError>;
    fn diffstat(&self, worktree: &Worktree) -> Result<DiffStat, GitError>;
}
```

### BranchManager

```rust
pub trait BranchManager: Send + Sync {
    fn create_from_worktree(&self, worktree: &Worktree, branch_name: &str, force: bool) -> Result<(), GitError>;
    fn push(&self, branch_name: &str, remote: &str, force: bool) -> Result<(), GitError>;
    fn exists(&self, branch_name: &str) -> bool;
    fn delete(&self, branch_name: &str, force: bool) -> Result<(), GitError>;
}
```

## Dependencies

| Crate | Purpose |
|-------|---------|
| `git2` | Git operations (libgit2 bindings) |
| `serde` | Serialization |
| `thiserror` | Error handling |
