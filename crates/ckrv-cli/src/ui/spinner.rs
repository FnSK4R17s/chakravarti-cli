//! # Spinner
//!
//! Animated loading indicators for long-running CLI operations.
//!
//! ## Overview
//!
//! Provides [`SpinnerGuard`], a RAII-style spinner that displays progress
//! during async operations like API calls and task execution. The spinner
//! automatically draws to stderr (per FR-005) and respects TTY detection.
//!
//! ## Usage
//!
//! ```rust
//! let spinner = SpinnerGuard::new("Loading tasks...", is_interactive, &theme);
//! // ... do work ...
//! spinner.success("Tasks loaded!");
//! ```
//!
//! ## Behavior
//!
//! - Non-interactive mode: No spinner shown
//! - Interactive mode: Animated braille spinner with message
//! - Supports `.success()`, `.error()`, and `.finish()` termination styles

// ============================================================
// IMPORTS
// ============================================================

use crate::ui::theme::Theme;
use indicatif::{ProgressBar, ProgressDrawTarget, ProgressStyle};
use std::time::Duration;

// ============================================================
// TYPES
// ============================================================

/// RAII-style spinner guard that displays an animated loading indicator.
///
/// The spinner draws to stderr (per FR-005) and respects TTY detection.
/// In non-interactive mode, the spinner is a no-op.
pub struct SpinnerGuard {
    inner: Option<ProgressBar>,
}

// ============================================================
// IMPLEMENTATION
// ============================================================

impl SpinnerGuard {
    /// Create a new spinner with the given message.
    ///
    /// If `is_interactive` is false, no spinner is displayed.
    pub fn new(msg: &str, is_interactive: bool, _theme: &Theme) -> Self {
        if !is_interactive {
            return Self { inner: None };
        }

        let pb = ProgressBar::new_spinner();
        // ENSURE it draws to stderr (Specification FR-005)
        pb.set_draw_target(ProgressDrawTarget::stderr());

        pb.set_message(msg.to_string());
        pb.enable_steady_tick(Duration::from_millis(80));

        // We use a premium looking spinner
        // TODO: Use theme colors if possible, but ProgressString template syntax is specific.
        // We'll use widely supported standard colors (cyan/blue) for now to match the hardcoded theme intent.
        let style = ProgressStyle::default_spinner()
            .tick_strings(&["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"])
            .template("{spinner:.cyan} {msg}")
            .unwrap_or_else(|_| ProgressStyle::default_spinner());

        pb.set_style(style);

        Self { inner: Some(pb) }
    }

    /// Update the spinner's display message.
    pub fn set_message(&self, msg: &str) {
        if let Some(pb) = &self.inner {
            pb.set_message(msg.to_string());
        }
    }

    /// Finish the spinner with a green checkmark and success message.
    pub fn success(&self, msg: &str) {
        if let Some(pb) = &self.inner {
            // Replace spinner with Green Check
            let style = ProgressStyle::default_spinner()
                .template("{spinner:.green} {msg}")
                .unwrap_or_else(|_| ProgressStyle::default_spinner());
            pb.set_style(style);
            // We use a tick char as a static symbol if steady tick is disabled?
            // Actually, finish_with_message stops the tick.
            // We want a static result.
            // Common pattern: print line and clear spinner.
            // Or finish the spinner with a specific symbol.
            // finish_and_clear puts strict cleanliness.
            // But we want "✔ Done".
            // indicatif doesn't have a "finish_with_symbol" natively easy without altering tick chars or prefix.
            // Simplest:
            pb.finish_with_message(format!("{} {}", "✔", msg));
        }
    }

    /// Finish the spinner with a red cross and error message.
    pub fn error(&self, msg: &str) {
        if let Some(pb) = &self.inner {
            pb.finish_with_message(format!("{} {}", "✖", msg));
        }
    }

    /// Finish and clear the spinner without any final message.
    pub fn finish(&self) {
        if let Some(pb) = &self.inner {
            pb.finish_and_clear();
        }
    }
}

// Drop safety: ensure spinner is cleared if dropped without explicit finish?
// or allow it to persist?
// best practice: finish_and_clear on drop if not finished?
// For now, let's keep it manual.
