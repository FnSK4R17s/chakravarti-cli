use std::io::IsTerminal;
use std::env;

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
    env::var("CLICOLOR_FORCE").map(|v| v != "0").unwrap_or(false)
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
