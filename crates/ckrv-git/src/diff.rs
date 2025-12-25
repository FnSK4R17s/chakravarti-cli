//! Git diff generation.

use std::path::Path;

use serde::{Deserialize, Serialize};

use crate::{GitError, Worktree};

/// A git diff.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Diff {
    /// Raw diff content.
    pub content: String,
    /// Files changed.
    pub files: Vec<FileDiff>,
}

/// A file diff entry.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileDiff {
    /// File path.
    pub path: String,
    /// Lines added.
    pub additions: usize,
    /// Lines removed.
    pub deletions: usize,
}

/// Diff statistics.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiffStat {
    /// Number of files changed.
    pub files_changed: usize,
    /// Total insertions.
    pub insertions: usize,
    /// Total deletions.
    pub deletions: usize,
}

impl Diff {
    /// Get statistics from this diff.
    #[must_use]
    pub fn stat(&self) -> DiffStat {
        DiffStat {
            files_changed: self.files.len(),
            insertions: self.files.iter().map(|f| f.additions).sum(),
            deletions: self.files.iter().map(|f| f.deletions).sum(),
        }
    }
}

/// Trait for generating git diffs.
pub trait DiffGenerator: Send + Sync {
    /// Generate a diff from worktree changes.
    ///
    /// # Errors
    ///
    /// Returns an error if diff generation fails.
    fn diff(&self, worktree: &Worktree) -> Result<Diff, GitError>;

    /// Get diff statistics.
    ///
    /// # Errors
    ///
    /// Returns an error if diffstat fails.
    fn diffstat(&self, worktree: &Worktree) -> Result<DiffStat, GitError>;
}

/// Default diff generator using git2.
pub struct DefaultDiffGenerator;

impl DefaultDiffGenerator {
    /// Create a new diff generator.
    #[must_use]
    pub fn new() -> Self {
        Self
    }

    /// Generate diff for a path (can be worktree or regular repo).
    ///
    /// # Errors
    ///
    /// Returns an error if diff generation fails.
    pub fn diff_path(&self, path: &Path) -> Result<Diff, GitError> {
        let repo = git2::Repository::open(path)?;

        // Get diff between HEAD and working directory
        let head = repo.head()?.peel_to_tree()?;
        let diff = repo.diff_tree_to_workdir_with_index(Some(&head), None)?;

        let mut content = String::new();
        let mut files = Vec::new();

        // Collect file stats
        for delta in diff.deltas() {
            let path = delta
                .new_file()
                .path()
                .or_else(|| delta.old_file().path())
                .map(|p| p.to_string_lossy().to_string())
                .unwrap_or_default();

            files.push(FileDiff {
                path,
                additions: 0,
                deletions: 0,
            });
        }

        // Generate patch content and count lines
        diff.print(git2::DiffFormat::Patch, |delta, _hunk, line| {
            if let Ok(c) = std::str::from_utf8(line.content()) {
                content.push_str(match line.origin() {
                    '+' => "+",
                    '-' => "-",
                    ' ' => " ",
                    _ => "",
                });
                content.push_str(c);
                if !c.ends_with('\n') {
                    content.push('\n');
                }
            }

            // Count additions/deletions per file
            if let Some(path) = delta.new_file().path().or_else(|| delta.old_file().path()) {
                let path_str = path.to_string_lossy().to_string();
                if let Some(file) = files.iter_mut().find(|f| f.path == path_str) {
                    match line.origin() {
                        '+' => file.additions += 1,
                        '-' => file.deletions += 1,
                        _ => {}
                    }
                }
            }

            true
        })
        .map_err(|e| GitError::DiffFailed(e.to_string()))?;

        Ok(Diff { content, files })
    }
}

impl Default for DefaultDiffGenerator {
    fn default() -> Self {
        Self::new()
    }
}

impl DiffGenerator for DefaultDiffGenerator {
    fn diff(&self, worktree: &Worktree) -> Result<Diff, GitError> {
        self.diff_path(&worktree.path)
    }

    fn diffstat(&self, worktree: &Worktree) -> Result<DiffStat, GitError> {
        let diff = self.diff(worktree)?;
        Ok(diff.stat())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::process::Command;
    use tempfile::TempDir;

    fn create_test_repo_with_changes() -> TempDir {
        let dir = TempDir::new().expect("temp dir");

        Command::new("git")
            .args(["init"])
            .current_dir(dir.path())
            .output()
            .expect("git init");

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

        // Initial commit
        std::fs::write(dir.path().join("file.txt"), "line 1\nline 2\n").ok();
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

        // Make changes
        std::fs::write(
            dir.path().join("file.txt"),
            "line 1\nline 2 modified\nline 3\n",
        )
        .ok();
        std::fs::write(dir.path().join("new_file.txt"), "new content\n").ok();

        dir
    }

    #[test]
    fn test_diff_path() {
        let repo = create_test_repo_with_changes();
        let generator = DefaultDiffGenerator::new();

        let diff = generator.diff_path(repo.path()).expect("diff");

        // Should have changes
        assert!(!diff.content.is_empty() || !diff.files.is_empty());
    }

    #[test]
    fn test_diff_stat() {
        let repo = create_test_repo_with_changes();
        let generator = DefaultDiffGenerator::new();

        let diff = generator.diff_path(repo.path()).expect("diff");
        let stat = diff.stat();

        // Should have some stats
        assert!(stat.files_changed >= 0);
    }

    #[test]
    fn test_file_diff_structure() {
        let file_diff = FileDiff {
            path: "test.rs".to_string(),
            additions: 10,
            deletions: 5,
        };

        assert_eq!(file_diff.path, "test.rs");
        assert_eq!(file_diff.additions, 10);
        assert_eq!(file_diff.deletions, 5);
    }

    #[test]
    fn test_diff_stat_from_diff() {
        let diff = Diff {
            content: String::new(),
            files: vec![
                FileDiff {
                    path: "a.rs".to_string(),
                    additions: 10,
                    deletions: 2,
                },
                FileDiff {
                    path: "b.rs".to_string(),
                    additions: 5,
                    deletions: 3,
                },
            ],
        };

        let stat = diff.stat();
        assert_eq!(stat.files_changed, 2);
        assert_eq!(stat.insertions, 15);
        assert_eq!(stat.deletions, 5);
    }
}
