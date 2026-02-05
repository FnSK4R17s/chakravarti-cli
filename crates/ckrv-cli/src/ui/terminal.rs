//! # Terminal Detection
//!
//! Utilities for detecting terminal capabilities and color preferences.
//!
//! ## Overview
//!
//! This module provides functions to detect whether the CLI is running in
//! an interactive terminal and whether color output should be enabled.
//! It respects the [NO_COLOR](https://no-color.org/) and `CLICOLOR_FORCE`
//! environment variables per common CLI conventions.
//!
//! ## Functions
//!
//! - [`is_tty`] - Check if stdout is a terminal
//! - [`is_no_color`] - Check if NO_COLOR is set
//! - [`is_force_color`] - Check if CLICOLOR_FORCE is set
//! - [`should_enable_rich_ui`] - Combined check for rich UI features

use std::env;
use std::io::IsTerminal;

/// Detects if the standard output is a terminal and if color should be enabled.
pub fn is_tty() -> bool {
    std::io::stdout().is_terminal()
}

/// Detects if the user has requested no color via NO_COLOR env var.
/// See https://no-color.org/
pub fn is_no_color() -> bool {
    env::var("NO_COLOR").is_ok()
}

/// Detects if the user has requested forced color via CLICOLOR_FORCE env var.
pub fn is_force_color() -> bool {
    env::var("CLICOLOR_FORCE")
        .map(|v| v != "0")
        .unwrap_or(false)
}

/// Determines if rich UI features (colors, spinners) should be enabled.
/// This considers TTY availability and environment variables.
/// Note: Silent mode (--json) is checked separately in UiContext.
pub fn should_enable_rich_ui() -> bool {
    if is_force_color() {
        return true;
    }
    if is_no_color() {
        return false;
    }
    is_tty()
}
