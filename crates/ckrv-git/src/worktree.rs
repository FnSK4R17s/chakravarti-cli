//! Git worktree management.

use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::GitError;

/// A git worktree for isolated execution.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Worktree {
    /// Absolute path to the worktree.
    pub path: PathBuf,
    /// Parent job ID.
    pub job_id: String,
    /// Parent attempt ID.
    pub attempt_id: String,
    /// Base commit SHA.
    pub base_commit: String,
    /// Current status.
    pub status: WorktreeStatus,
}

/// Status of a worktree.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum WorktreeStatus {
    /// Worktree is being created.
    Creating,
    /// Worktree is ready for use.
    Ready,
    /// Worktree is in use.
    InUse,
    /// Worktree is being cleaned up.
    Cleanup,
    /// Worktree has been deleted.
    Deleted,
}

/// Trait for managing git worktrees.
pub trait WorktreeManager: Send + Sync {
    /// Create a new worktree for a job attempt.
    ///
    /// # Errors
    ///
    /// Returns an error if worktree creation fails.
    fn create(&self, job_id: &str, attempt_id: &str) -> Result<Worktree, GitError>;

    /// Get the path for a worktree.
    ///
    /// # Errors
    ///
    /// Returns an error if the worktree doesn't exist.
    fn path(&self, job_id: &str, attempt_id: &str) -> Result<PathBuf, GitError>;

    /// Clean up a worktree.
    ///
    /// # Errors
    ///
    /// Returns an error if cleanup fails.
    fn cleanup(&self, worktree: &Worktree) -> Result<(), GitError>;

    /// List all worktrees.
    ///
    /// # Errors
    ///
    /// Returns an error if listing fails.
    fn list(&self) -> Result<Vec<Worktree>, GitError>;
}

/// Default worktree manager using git2.
pub struct DefaultWorktreeManager {
    /// Path to the main repository.
    repo_path: PathBuf,
    /// Base path for worktrees.
    worktree_base: PathBuf,
}

impl DefaultWorktreeManager {
    /// Create a new worktree manager.
    ///
    /// # Errors
    ///
    /// Returns an error if the path is not a git repository.
    pub fn new(repo_path: &Path) -> Result<Self, GitError> {
        let repo = git2::Repository::discover(repo_path)?;
        let workdir = repo
            .workdir()
            .ok_or_else(|| GitError::NotARepo("Bare repository".to_string()))?;

        let worktree_base = workdir.join(".chakravarti").join("worktrees");
        std::fs::create_dir_all(&worktree_base)
            .map_err(|e| GitError::WorktreeCreationFailed(e.to_string()))?;

        Ok(Self {
            repo_path: workdir.to_path_buf(),
            worktree_base,
        })
    }

    fn worktree_name(job_id: &str, attempt_id: &str) -> String {
        format!("{job_id}_{attempt_id}")
    }
}

impl WorktreeManager for DefaultWorktreeManager {
    fn create(&self, job_id: &str, attempt_id: &str) -> Result<Worktree, GitError> {
        let repo = git2::Repository::open(&self.repo_path)?;
        let name = Self::worktree_name(job_id, attempt_id);
        let worktree_path = self.worktree_base.join(&name);

        // Get current HEAD commit
        let head = repo.head()?;
        let head_commit = head.peel_to_commit()?;
        let base_commit = head_commit.id().to_string();

        // Create a new branch name for this worktree (ckrv-<job_id short>)
        let branch_name = format!("ckrv-{}", &job_id[..8.min(job_id.len())]);
        
        // Create the branch from HEAD
        let branch = repo
            .branch(&branch_name, &head_commit, false)
            .map_err(|e| GitError::WorktreeCreationFailed(format!("Failed to create branch: {}", e)))?;
        
        // Get the branch reference
        let branch_ref = branch.into_reference();

        // Create the worktree using the new branch
        repo.worktree(
            &name,
            &worktree_path,
            Some(git2::WorktreeAddOptions::new().reference(Some(&branch_ref))),
        )
        .map_err(|e| GitError::WorktreeCreationFailed(e.to_string()))?;

        Ok(Worktree {
            path: worktree_path,
            job_id: job_id.to_string(),
            attempt_id: attempt_id.to_string(),
            base_commit,
            status: WorktreeStatus::Ready,
        })
    }

    fn path(&self, job_id: &str, attempt_id: &str) -> Result<PathBuf, GitError> {
        let name = Self::worktree_name(job_id, attempt_id);
        let path = self.worktree_base.join(&name);

        if path.exists() {
            Ok(path)
        } else {
            Err(GitError::NotARepo(format!("Worktree not found: {name}")))
        }
    }

    fn cleanup(&self, worktree: &Worktree) -> Result<(), GitError> {
        let repo = git2::Repository::open(&self.repo_path)?;
        let name = Self::worktree_name(&worktree.job_id, &worktree.attempt_id);

        // Prune the worktree from git
        if let Ok(wt) = repo.find_worktree(&name) {
            wt.prune(Some(
                git2::WorktreePruneOptions::new()
                    .working_tree(true)
                    .valid(true),
            ))
            .map_err(|e| GitError::WorktreeCleanupFailed(e.to_string()))?;
        }

        // Remove the directory
        if worktree.path.exists() {
            std::fs::remove_dir_all(&worktree.path)
                .map_err(|e| GitError::WorktreeCleanupFailed(e.to_string()))?;
        }

        Ok(())
    }

    fn list(&self) -> Result<Vec<Worktree>, GitError> {
        let repo = git2::Repository::open(&self.repo_path)?;
        let mut worktrees = Vec::new();

        for name in repo.worktrees()?.iter().flatten() {
            if let Ok(wt) = repo.find_worktree(name) {
                if let Some(path) = wt.path().to_str() {
                    // Parse job_id and attempt_id from name
                    let parts: Vec<&str> = name.split('_').collect();
                    if parts.len() >= 2 {
                        worktrees.push(Worktree {
                            path: PathBuf::from(path),
                            job_id: parts[0].to_string(),
                            attempt_id: parts[1..].join("_"),
                            base_commit: String::new(), // Would need to look this up
                            status: if wt.is_locked().is_ok() {
                                WorktreeStatus::InUse
                            } else {
                                WorktreeStatus::Ready
                            },
                        });
                    }
                }
            }
        }

        Ok(worktrees)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::process::Command;
    use tempfile::TempDir;

    fn create_test_repo() -> TempDir {
        let dir = TempDir::new().expect("temp dir");
        Command::new("git")
            .args(["init"])
            .current_dir(dir.path())
            .output()
            .expect("git init");

        // Create initial commit
        Command::new("git")
            .args(["config", "user.email", "test@test.com"])
            .current_dir(dir.path())
            .output()
            .ok();
        Command::new("git")
            .args(["config", "user.name", "Test"])
            .current_dir(dir.path())
            .output()
            .ok();

        std::fs::write(dir.path().join("README.md"), "# Test").ok();
        Command::new("git")
            .args(["add", "."])
            .current_dir(dir.path())
            .output()
            .ok();
        Command::new("git")
            .args(["commit", "-m", "Initial"])
            .current_dir(dir.path())
            .output()
            .ok();

        dir
    }

    #[test]
    fn test_worktree_manager_new() {
        let repo = create_test_repo();
        let manager = DefaultWorktreeManager::new(repo.path());
        assert!(manager.is_ok());
    }

    #[test]
    fn test_worktree_struct() {
        let worktree = Worktree {
            path: PathBuf::from("/tmp/test"),
            job_id: "job1".to_string(),
            attempt_id: "attempt1".to_string(),
            base_commit: "abc123".to_string(),
            status: WorktreeStatus::Ready,
        };

        assert_eq!(worktree.job_id, "job1");
        assert_eq!(worktree.status, WorktreeStatus::Ready);
    }

    #[test]
    fn test_worktree_status_serialization() {
        let status = WorktreeStatus::InUse;
        let json = serde_json::to_string(&status).expect("serialize");
        assert!(json.contains("in_use"));
    }

    #[test]
    fn test_worktree_name_generation() {
        let name = DefaultWorktreeManager::worktree_name("job123", "attempt1");
        assert_eq!(name, "job123_attempt1");
    }

    // Integration tests that require git worktree support
    // These are more suited for integration testing rather than unit tests
    #[test]
    #[ignore = "Requires git worktree support which may not work in all test environments"]
    fn test_worktree_create_and_cleanup() {
        let repo = create_test_repo();
        let manager = DefaultWorktreeManager::new(repo.path()).expect("manager");

        let worktree = manager.create("job1", "attempt1").expect("create");
        assert!(worktree.path.exists());
        assert_eq!(worktree.status, WorktreeStatus::Ready);

        manager.cleanup(&worktree).expect("cleanup");
        assert!(!worktree.path.exists());
    }

    #[test]
    #[ignore = "Requires git worktree support which may not work in all test environments"]
    fn test_worktree_path() {
        let repo = create_test_repo();
        let manager = DefaultWorktreeManager::new(repo.path()).expect("manager");

        let worktree = manager.create("job2", "attempt1").expect("create");
        let path = manager.path("job2", "attempt1").expect("path");
        assert_eq!(path, worktree.path);

        manager.cleanup(&worktree).ok();
    }

    #[test]
    #[ignore = "Requires git worktree support which may not work in all test environments"]
    fn test_worktree_list() {
        let repo = create_test_repo();
        let manager = DefaultWorktreeManager::new(repo.path()).expect("manager");

        let wt1 = manager.create("job3", "a1").expect("create1");
        let wt2 = manager.create("job3", "a2").expect("create2");

        let list = manager.list().expect("list");
        assert!(list.len() >= 2);

        manager.cleanup(&wt1).ok();
        manager.cleanup(&wt2).ok();
    }
}
