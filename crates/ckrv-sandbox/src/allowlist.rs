//! Tool allowlist for sandboxed execution.

/// Trait for checking command allowlist.
pub trait AllowList: Send + Sync {
    /// Check if a command is allowed.
    fn is_allowed(&self, command: &[String]) -> bool;

    /// Get list of allowed commands.
    fn allowed_commands(&self) -> &[String];

    /// Get list of blocked patterns.
    fn blocked_patterns(&self) -> &[String];
}

/// Default allowlist configuration.
#[derive(Debug, Clone)]
pub struct DefaultAllowList {
    allowed: Vec<String>,
    blocked: Vec<String>,
}

impl Default for DefaultAllowList {
    fn default() -> Self {
        Self {
            allowed: vec![
                // Shells (needed for shell commands)
                "sh".to_string(),
                "bash".to_string(),
                // Rust
                "cargo".to_string(),
                "rustc".to_string(),
                "rustfmt".to_string(),
                "clippy".to_string(),
                // Node/JS
                "npm".to_string(),
                "npx".to_string(),
                "node".to_string(),
                "yarn".to_string(),
                "pnpm".to_string(),
                // Python
                "python".to_string(),
                "python3".to_string(),
                "pip".to_string(),
                "pytest".to_string(),
                // Go
                "go".to_string(),
                // General
                "make".to_string(),
                "git".to_string(),
                "echo".to_string(),
                "cat".to_string(),
                "ls".to_string(),
                "test".to_string(),
            ],
            blocked: vec![
                // Network tools
                "curl".to_string(),
                "wget".to_string(),
                "ssh".to_string(),
                "scp".to_string(),
                "nc".to_string(),
                "netcat".to_string(),
                // Dangerous system tools
                "rm".to_string(),
                "dd".to_string(),
                "mkfs".to_string(),
            ],
        }
    }
}

impl AllowList for DefaultAllowList {
    fn is_allowed(&self, command: &[String]) -> bool {
        if command.is_empty() {
            return false;
        }

        let cmd = &command[0];

        // Check blocked patterns first
        for pattern in &self.blocked {
            if cmd.contains(pattern) {
                return false;
            }
        }

        // Check allowed commands
        self.allowed
            .iter()
            .any(|a| cmd == a || cmd.ends_with(&format!("/{a}")))
    }

    fn allowed_commands(&self) -> &[String] {
        &self.allowed
    }

    fn blocked_patterns(&self) -> &[String] {
        &self.blocked
    }
}

impl DefaultAllowList {
    /// Create with custom allowed commands.
    #[must_use]
    pub fn with_allowed(allowed: Vec<String>) -> Self {
        Self {
            allowed,
            blocked: vec![
                "curl".to_string(),
                "wget".to_string(),
                "ssh".to_string(),
                "scp".to_string(),
                "nc".to_string(),
            ],
        }
    }

    /// Add an allowed command.
    #[must_use]
    pub fn allow(mut self, cmd: impl Into<String>) -> Self {
        self.allowed.push(cmd.into());
        self
    }

    /// Add a blocked pattern.
    #[must_use]
    pub fn block(mut self, pattern: impl Into<String>) -> Self {
        self.blocked.push(pattern.into());
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_allowlist_cargo() {
        let allowlist = DefaultAllowList::default();
        assert!(allowlist.is_allowed(&["cargo".to_string(), "test".to_string()]));
    }

    #[test]
    fn test_default_allowlist_npm() {
        let allowlist = DefaultAllowList::default();
        assert!(allowlist.is_allowed(&["npm".to_string(), "run".to_string(), "test".to_string()]));
    }

    #[test]
    fn test_blocked_curl() {
        let allowlist = DefaultAllowList::default();
        assert!(!allowlist.is_allowed(&["curl".to_string(), "https://example.com".to_string()]));
    }

    #[test]
    fn test_blocked_wget() {
        let allowlist = DefaultAllowList::default();
        assert!(!allowlist.is_allowed(&["wget".to_string(), "file.txt".to_string()]));
    }

    #[test]
    fn test_empty_command() {
        let allowlist = DefaultAllowList::default();
        assert!(!allowlist.is_allowed(&[]));
    }

    #[test]
    fn test_custom_allowlist() {
        let allowlist = DefaultAllowList::with_allowed(vec!["python".to_string()]).allow("pytest");

        assert!(allowlist.is_allowed(&["python".to_string()]));
        assert!(allowlist.is_allowed(&["pytest".to_string()]));
        assert!(!allowlist.is_allowed(&["cargo".to_string()]));
    }

    #[test]
    fn test_full_path_allowed() {
        let allowlist = DefaultAllowList::default();
        assert!(allowlist.is_allowed(&["/usr/bin/cargo".to_string()]));
    }
}
