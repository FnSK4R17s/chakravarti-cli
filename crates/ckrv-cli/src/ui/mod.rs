//! # CLI UI Subsystem
//!
//! Rich terminal rendering layer for the Chakravarti CLI.
//!
//! ## Overview
//!
//! This module provides a consistent, themed UI for all CLI output. It handles:
//! - Terminal capability detection (colors, unicode, interactivity)
//! - Themed output with consistent styling
//! - Spinner animations for long-running operations
//! - Markdown rendering in the terminal
//! - Panel-based success/error messaging
//!
//! ## Key Types
//!
//! - [`UiContext`] - Main entry point, holds theme and rendering state
//! - [`Theme`] - Color palette and styling configuration
//! - [`Renderable`] - Trait for components that can be rendered
//!
//! ## Example
//!
//! ```rust,ignore
//! use ckrv_cli::ui::UiContext;
//!
//! let ui = UiContext::new(false); // Not in JSON mode
//! ui.success("Done", "Operation completed successfully");
//!
//! let spinner = ui.spinner("Processing...");
//! // ... do work ...
//! spinner.success("Finished!");
//! ```
//!
//! ## Silent Mode
//!
//! When `--json` is passed, the UI enters "silent mode" where all
//! decorative output is suppressed and only structured JSON is emitted.

// ============================================================
// MODULES
// ============================================================

/// Reusable terminal UI components (banners, tables, panels).
pub mod components;
/// Animated loading indicators for long-running operations.
pub mod spinner;
/// Terminal capability detection (color, unicode, interactivity).
pub mod terminal;
/// Visual theming configuration (colors, styles, markdown skin).
pub mod theme;

// ============================================================
// IMPORTS
// ============================================================

use terminal::should_enable_rich_ui;
pub use theme::Theme;

// ============================================================
// TYPES
// ============================================================

/// The main entry point for interaction with the CLI UI.
/// It holds the theme and determines whether to render rich output.
pub struct UiContext {
    /// Active theme controlling colors and styling.
    pub theme: Theme,
    /// Whether rich interaction (Spinners, Colors) is allowed.
    pub is_interactive: bool,
    /// If true, we are in "Silent Mode" (e.g. JSON output), suppressing banners/decorations.
    pub silent: bool,
}

// ============================================================
// IMPLEMENTATION
// ============================================================

impl UiContext {
    /// Create a new UI context.
    ///
    /// # Arguments
    ///
    /// * `json_mode` - If true, enables "Silent Mode" (no branding, no spinners, clean stdout).
    pub fn new(json_mode: bool) -> Self {
        // If JSON mode is on, we are NOT interactive and we ARE silent.
        // If JSON mode is off, we check the terminal.
        let is_interactive = if json_mode {
            false
        } else {
            should_enable_rich_ui()
        };

        Self {
            theme: Theme::default(),
            is_interactive,
            silent: json_mode,
        }
    }

    /// Check if we should render decorative elements (Banners, Panels).
    pub fn should_render_decorations(&self) -> bool {
        !self.silent && self.is_interactive
    }

    /// Print a component to stdout in a rendered form.
    pub fn print(&self, component: impl Renderable) {
        if self.silent {
            // In silent mode, we don't print "UI components" via this method usually?
            // Or we print a minimal representation?
            // Spec says "Silent Mode... disable colors/styles... banners disabled".
            // So we do nothing?
            return;
        }

        // Render with our theme
        let output = component.render(&self.theme);
        println!("{}", output);
    }

    /// Render markdown content to stdout.
    pub fn markdown(&self, content: &str) {
        if self.silent {
            return;
        }
        self.theme.markdown_skin.print_text(content);
    }

    /// Display a success panel.
    pub fn success(&self, title: &str, msg: &str) {
        self.print(components::Panel::new(title, msg).success());
    }

    /// Display an info panel (uses success styling).
    pub fn info(&self, title: &str, msg: &str) {
        self.print(components::Panel::new(title, msg).success());
    }

    /// Display an error panel.
    pub fn error(&self, title: &str, msg: &str) {
        self.print(components::Panel::new(title, msg).error());
    }

    /// Display a warning panel (uses error styling).
    pub fn warn(&self, title: &str, msg: &str) {
        self.print(components::Panel::new(title, msg).error());
    }

    /// Start a spinner.
    pub fn spinner(&self, msg: impl Into<String>) -> spinner::SpinnerGuard {
        spinner::SpinnerGuard::new(&msg.into(), self.is_interactive, &self.theme)
    }
}

/// Trait for UI components that can be rendered to a string with theming.
pub trait Renderable {
    /// Render the component to a string, respecting the provided theme.
    fn render(&self, theme: &Theme) -> String;
}
