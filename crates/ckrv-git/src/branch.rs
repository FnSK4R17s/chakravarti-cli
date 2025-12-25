//! Git branch management.

use std::path::Path;
use std::process::Command;

use crate::{GitError, Worktree};

/// Trait for managing git branches.
pub trait BranchManager: Send + Sync {
    /// Create a branch from a worktree.
    ///
    /// # Errors
    ///
    /// Returns an error if branch creation fails.
    fn create_from_worktree(
        &self,
        worktree: &Worktree,
        branch_name: &str,
        force: bool,
    ) -> Result<(), GitError>;

    /// Push a branch to a remote.
    ///
    /// # Errors
    ///
    /// Returns an error if push fails.
    fn push(&self, branch_name: &str, remote: &str, force: bool) -> Result<(), GitError>;

    /// Check if a branch exists.
    fn exists(&self, branch_name: &str) -> bool;

    /// Delete a branch.
    ///
    /// # Errors
    ///
    /// Returns an error if deletion fails.
    fn delete(&self, branch_name: &str, force: bool) -> Result<(), GitError>;
}

/// Git-based branch manager.
pub struct GitBranchManager {
    repo_root: std::path::PathBuf,
}

impl GitBranchManager {
    /// Create a new branch manager for a repository.
    pub fn new(repo_root: impl Into<std::path::PathBuf>) -> Self {
        Self {
            repo_root: repo_root.into(),
        }
    }

    /// Get repository root.
    #[must_use]
    pub fn repo_root(&self) -> &Path {
        &self.repo_root
    }

    fn git(&self, args: &[&str]) -> Result<String, GitError> {
        let output = Command::new("git")
            .args(args)
            .current_dir(&self.repo_root)
            .output()
            .map_err(|e| GitError::CommandFailed(format!("git {}: {}", args.join(" "), e)))?;

        if output.status.success() {
            Ok(String::from_utf8_lossy(&output.stdout).to_string())
        } else {
            Err(GitError::CommandFailed(format!(
                "git {}: {}",
                args.join(" "),
                String::from_utf8_lossy(&output.stderr)
            )))
        }
    }
}

impl BranchManager for GitBranchManager {
    fn create_from_worktree(
        &self,
        worktree: &Worktree,
        branch_name: &str,
        force: bool,
    ) -> Result<(), GitError> {
        // First, create a commit in the worktree if there are changes
        let worktree_path = &worktree.path;

        // Check for changes
        let status_output = Command::new("git")
            .args(["status", "--porcelain"])
            .current_dir(worktree_path)
            .output()
            .map_err(|e| GitError::CommandFailed(e.to_string()))?;

        let has_changes = !status_output.stdout.is_empty();

        if has_changes {
            // Stage all changes
            Command::new("git")
                .args(["add", "-A"])
                .current_dir(worktree_path)
                .output()
                .map_err(|e| GitError::CommandFailed(e.to_string()))?;

            // Commit
            Command::new("git")
                .args([
                    "commit",
                    "-m",
                    &format!("chakravarti: changes for {}", branch_name),
                ])
                .current_dir(worktree_path)
                .output()
                .map_err(|e| GitError::CommandFailed(e.to_string()))?;
        }

        // Get the current HEAD of the worktree
        let head_output = Command::new("git")
            .args(["rev-parse", "HEAD"])
            .current_dir(worktree_path)
            .output()
            .map_err(|e| GitError::CommandFailed(e.to_string()))?;

        let head_ref = String::from_utf8_lossy(&head_output.stdout)
            .trim()
            .to_string();

        // Create/update the branch in the main repo
        let branch_args = if force {
            vec!["branch", "-f", branch_name, &head_ref]
        } else {
            vec!["branch", branch_name, &head_ref]
        };

        self.git(&branch_args)?;

        Ok(())
    }

    fn push(&self, branch_name: &str, remote: &str, force: bool) -> Result<(), GitError> {
        let args = if force {
            vec!["push", "--force", remote, branch_name]
        } else {
            vec!["push", remote, branch_name]
        };

        self.git(&args)?;
        Ok(())
    }

    fn exists(&self, branch_name: &str) -> bool {
        self.git(&[
            "rev-parse",
            "--verify",
            &format!("refs/heads/{}", branch_name),
        ])
        .is_ok()
    }

    fn delete(&self, branch_name: &str, force: bool) -> Result<(), GitError> {
        let flag = if force { "-D" } else { "-d" };
        self.git(&["branch", flag, branch_name])?;
        Ok(())
    }
}

/// Result of a promote operation.
#[derive(Debug, Clone)]
pub struct PromoteResult {
    /// The branch that was created.
    pub branch_name: String,
    /// Whether the branch was pushed to remote.
    pub pushed: bool,
    /// The commit hash.
    pub commit: String,
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn init_git_repo(dir: &Path) {
        Command::new("git")
            .args(["init"])
            .current_dir(dir)
            .output()
            .expect("git init");

        Command::new("git")
            .args(["config", "user.email", "test@test.com"])
            .current_dir(dir)
            .output()
            .expect("config email");

        Command::new("git")
            .args(["config", "user.name", "Test"])
            .current_dir(dir)
            .output()
            .expect("config name");

        // Create initial commit
        std::fs::write(dir.join("README.md"), "# Test").expect("write readme");
        Command::new("git")
            .args(["add", "-A"])
            .current_dir(dir)
            .output()
            .expect("git add");
        Command::new("git")
            .args(["commit", "-m", "Initial commit"])
            .current_dir(dir)
            .output()
            .expect("git commit");
    }

    #[test]
    fn test_branch_exists_false() {
        let dir = TempDir::new().expect("temp dir");
        init_git_repo(dir.path());

        let manager = GitBranchManager::new(dir.path());
        assert!(!manager.exists("nonexistent-branch"));
    }

    #[test]
    fn test_branch_manager_git_helper() {
        let dir = TempDir::new().expect("temp dir");
        init_git_repo(dir.path());

        let manager = GitBranchManager::new(dir.path());
        let result = manager.git(&["branch", "--list"]);
        assert!(result.is_ok());
    }

    #[test]
    fn test_branch_delete_nonexistent() {
        let dir = TempDir::new().expect("temp dir");
        init_git_repo(dir.path());

        let manager = GitBranchManager::new(dir.path());
        let result = manager.delete("nonexistent", false);
        assert!(result.is_err());
    }
}
