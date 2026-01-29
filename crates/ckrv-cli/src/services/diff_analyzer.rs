//! Git diff analyzer - analyze changes vs base branch.

use std::path::PathBuf;
use std::process::Command;

use serde::Serialize;

/// Type of file change
#[derive(Debug, Clone, Serialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum ChangeType {
    Added,
    Modified,
    Deleted,
    Renamed,
}

/// A file that changed
#[derive(Debug, Clone, Serialize)]
pub struct ChangedFile {
    pub path: PathBuf,
    pub change_type: ChangeType,
    pub lines_added: u32,
    pub lines_removed: u32,
}

/// Get the default base branch (usually main or master)
pub fn get_base_branch() -> anyhow::Result<String> {
    // Try to get the default branch from remote
    let output = Command::new("git")
        .args(["symbolic-ref", "refs/remotes/origin/HEAD"])
        .output()?;

    if output.status.success() {
        let branch = String::from_utf8_lossy(&output.stdout)
            .trim()
            .replace("refs/remotes/origin/", "");
        return Ok(branch);
    }

    // Fall back to checking if main or master exists
    for branch in &["main", "master"] {
        let check = Command::new("git")
            .args(["rev-parse", "--verify", branch])
            .output()?;
        if check.status.success() {
            return Ok(branch.to_string());
        }
    }

    // Default to main
    Ok("main".to_string())
}

/// Get the current branch name
pub fn get_current_branch() -> anyhow::Result<String> {
    let output = Command::new("git")
        .args(["branch", "--show-current"])
        .output()?;

    if output.status.success() {
        Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
    } else {
        anyhow::bail!("Failed to get current branch")
    }
}

/// Get list of changed files compared to base branch
pub fn get_changed_files(base: &str) -> anyhow::Result<Vec<ChangedFile>> {
    let output = Command::new("git")
        .args(["diff", "--name-status", &format!("{}...HEAD", base)])
        .output()?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        anyhow::bail!("git diff failed: {}", stderr);
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let mut files = Vec::new();

    for line in stdout.lines() {
        let parts: Vec<&str> = line.split('\t').collect();
        if parts.len() >= 2 {
            let path = PathBuf::from(parts.last().unwrap_or(&""));

            // Skip paths that should be excluded
            if should_exclude_path(&path) {
                continue;
            }

            let change_type = match parts[0].chars().next() {
                Some('A') => ChangeType::Added,
                Some('M') => ChangeType::Modified,
                Some('D') => ChangeType::Deleted,
                Some('R') => ChangeType::Renamed,
                _ => ChangeType::Modified,
            };

            // Get line counts
            let (lines_added, lines_removed) = get_file_diff_stats(base, &path)?;

            files.push(ChangedFile {
                path,
                change_type,
                lines_added,
                lines_removed,
            });
        }
    }

    Ok(files)
}

/// Check if a path should be excluded from analysis
fn should_exclude_path(path: &PathBuf) -> bool {
    let path_str = path.to_string_lossy();

    // Common directories to exclude
    let excluded_dirs = [
        "node_modules/",
        "vendor/",
        "target/",
        "dist/",
        "build/",
        ".git/",
        ".next/",
        "__pycache__/",
        ".pytest_cache/",
        ".mypy_cache/",
        "coverage/",
        ".nyc_output/",
        ".turbo/",
        "out/",
        ".vercel/",
        ".cache/",
        "pkg/",
        "bin/",
        ".venv/",
        "venv/",
        "env/",
        ".env/",
    ];

    for dir in &excluded_dirs {
        if path_str.contains(dir) {
            return true;
        }
    }

    // Exclude common generated/lock files
    let excluded_files = [
        "package-lock.json",
        "yarn.lock",
        "pnpm-lock.yaml",
        "Cargo.lock",
        "poetry.lock",
        "Gemfile.lock",
        ".DS_Store",
    ];

    let filename = path
        .file_name()
        .map(|f| f.to_string_lossy().to_string())
        .unwrap_or_default();

    for file in &excluded_files {
        if filename == *file {
            return true;
        }
    }

    false
}

/// Get line counts for a specific file
fn get_file_diff_stats(base: &str, file: &PathBuf) -> anyhow::Result<(u32, u32)> {
    let output = Command::new("git")
        .args([
            "diff",
            "--numstat",
            &format!("{}...HEAD", base),
            "--",
            file.to_str().unwrap_or(""),
        ])
        .output()?;

    if output.status.success() {
        let stdout = String::from_utf8_lossy(&output.stdout);
        if let Some(line) = stdout.lines().next() {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 2 {
                let added = parts[0].parse().unwrap_or(0);
                let removed = parts[1].parse().unwrap_or(0);
                return Ok((added, removed));
            }
        }
    }

    Ok((0, 0))
}

/// Get the full diff content for QA review
pub fn get_diff_content(base: &str) -> anyhow::Result<String> {
    let output = Command::new("git")
        .args(["diff", &format!("{}...HEAD", base)])
        .output()?;

    if output.status.success() {
        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        anyhow::bail!("git diff failed: {}", stderr)
    }
}

/// Check if there are any changes vs base
pub fn has_changes(base: &str) -> bool {
    get_changed_files(base)
        .map(|files| !files.is_empty())
        .unwrap_or(false)
}

/// Get a summary of changes for display
pub fn get_change_summary(base: &str) -> anyhow::Result<String> {
    let files = get_changed_files(base)?;

    if files.is_empty() {
        return Ok(format!("No changes compared to {}", base));
    }

    let added = files
        .iter()
        .filter(|f| f.change_type == ChangeType::Added)
        .count();
    let modified = files
        .iter()
        .filter(|f| f.change_type == ChangeType::Modified)
        .count();
    let deleted = files
        .iter()
        .filter(|f| f.change_type == ChangeType::Deleted)
        .count();

    let total_lines_added: u32 = files.iter().map(|f| f.lines_added).sum();
    let total_lines_removed: u32 = files.iter().map(|f| f.lines_removed).sum();

    Ok(format!(
        "{} files changed ({} added, {} modified, {} deleted)\n+{} lines, -{} lines",
        files.len(),
        added,
        modified,
        deleted,
        total_lines_added,
        total_lines_removed
    ))
}
