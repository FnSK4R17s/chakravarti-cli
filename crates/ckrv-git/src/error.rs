//! Git error types.

use thiserror::Error;

/// Errors from git operations.
#[derive(Debug, Error)]
pub enum GitError {
    /// Not a git repository.
    #[error("Not a git repository: {0}")]
    NotARepo(String),

    /// Worktree creation failed.
    #[error("Worktree creation failed: {0}")]
    WorktreeCreationFailed(String),

    /// Worktree cleanup failed.
    #[error("Worktree cleanup failed: {0}")]
    WorktreeCleanupFailed(String),

    /// Diff generation failed.
    #[error("Diff generation failed: {0}")]
    DiffFailed(String),

    /// Branch operation failed.
    #[error("Branch operation failed: {0}")]
    BranchFailed(String),

    /// Git command failed.
    #[error("Git command failed: {0}")]
    CommandFailed(String),

    /// Git2 library error.
    #[error("Git error: {0}")]
    Git2Error(#[from] git2::Error),
}
