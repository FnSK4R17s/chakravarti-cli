//! Git operations and worktree management for Chakravarti CLI.
//!
//! This crate provides git functionality including worktree creation,
//! diff generation, and branch management.

use std::path::Path;

pub mod branch;
pub mod diff;
pub mod error;
pub mod worktree;

pub use branch::{BranchManager, GitBranchManager, PromoteResult};
pub use diff::{DefaultDiffGenerator, Diff, DiffGenerator, DiffStat, FileDiff};
pub use error::GitError;
pub use worktree::{DefaultWorktreeManager, Worktree, WorktreeManager, WorktreeStatus};

/// Check if a path is inside a git repository.
///
/// # Errors
///
/// Returns an error if the path cannot be accessed.
pub fn is_git_repo(path: &Path) -> Result<bool, GitError> {
    match git2::Repository::discover(path) {
        Ok(_) => Ok(true),
        Err(e) if e.code() == git2::ErrorCode::NotFound => Ok(false),
        Err(e) => Err(GitError::Git2Error(e)),
    }
}

/// Check if Chakravarti has been initialized in the repository.
///
/// Returns true if both `.specs/` and `.chakravarti/` directories exist.
#[must_use]
pub fn is_initialized(repo_root: &Path) -> bool {
    repo_root.join(".specs").exists() && repo_root.join(".chakravarti").exists()
}

/// Get the root directory of the git repository containing the given path.
///
/// # Errors
///
/// Returns an error if the path is not in a git repository.
pub fn repo_root(path: &Path) -> Result<std::path::PathBuf, GitError> {
    let repo = git2::Repository::discover(path)?;
    repo.workdir()
        .map(|p| p.to_path_buf())
        .ok_or_else(|| GitError::NotARepo("Bare repository".to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn create_git_repo() -> TempDir {
        let dir = TempDir::new().expect("Failed to create temp dir");
        git2::Repository::init(dir.path()).expect("Failed to init git repo");
        dir
    }

    // =========================================================================
    // T033: Unit tests for git repo detection
    // =========================================================================

    #[test]
    fn test_is_git_repo_true_for_git_directory() {
        let repo = create_git_repo();
        assert!(is_git_repo(repo.path()).unwrap());
    }

    #[test]
    fn test_is_git_repo_false_for_non_git_directory() {
        let dir = TempDir::new().expect("Failed to create temp dir");
        assert!(!is_git_repo(dir.path()).unwrap());
    }

    #[test]
    fn test_is_git_repo_true_for_subdirectory() {
        let repo = create_git_repo();
        let subdir = repo.path().join("subdir");
        std::fs::create_dir(&subdir).expect("Failed to create subdir");
        assert!(is_git_repo(&subdir).unwrap());
    }

    // =========================================================================
    // T034-T035: Unit tests for is_initialized
    // =========================================================================

    #[test]
    fn test_is_initialized_false_when_nothing_exists() {
        let repo = create_git_repo();
        assert!(!is_initialized(repo.path()));
    }

    #[test]
    fn test_is_initialized_false_when_only_specs_exists() {
        let repo = create_git_repo();
        std::fs::create_dir(repo.path().join(".specs")).expect("create .specs");
        assert!(!is_initialized(repo.path()));
    }

    #[test]
    fn test_is_initialized_false_when_only_chakravarti_exists() {
        let repo = create_git_repo();
        std::fs::create_dir(repo.path().join(".chakravarti")).expect("create .chakravarti");
        assert!(!is_initialized(repo.path()));
    }

    #[test]
    fn test_is_initialized_true_when_both_exist() {
        let repo = create_git_repo();
        std::fs::create_dir(repo.path().join(".specs")).expect("create .specs");
        std::fs::create_dir(repo.path().join(".chakravarti")).expect("create .chakravarti");
        assert!(is_initialized(repo.path()));
    }

    // =========================================================================
    // Unit tests for repo_root
    // =========================================================================

    #[test]
    fn test_repo_root_returns_root_path() {
        let repo = create_git_repo();
        let root = repo_root(repo.path()).unwrap();
        assert_eq!(root, repo.path());
    }

    #[test]
    fn test_repo_root_from_subdirectory() {
        let repo = create_git_repo();
        let subdir = repo.path().join("deep").join("nested");
        std::fs::create_dir_all(&subdir).expect("create nested dirs");
        let root = repo_root(&subdir).unwrap();
        assert_eq!(root, repo.path());
    }

    #[test]
    fn test_repo_root_fails_for_non_git() {
        let dir = TempDir::new().expect("Failed to create temp dir");
        assert!(repo_root(dir.path()).is_err());
    }
}
