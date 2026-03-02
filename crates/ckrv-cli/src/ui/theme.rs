//! # Theme
//!
//! Visual theming configuration for CLI output.
//!
//! ## Overview
//!
//! Defines the [`Theme`] struct which holds all color and styling settings
//! used by UI components. The default theme uses a "Royal Gold" palette
//! inspired by modern premium CLIs.
//!
//! ## Theme Elements
//!
//! - **Primary color**: Gold (256-color code 220)
//! - **Secondary color**: Cyan
//! - **Box characters**: Unicode box-drawing characters for borders
//! - **Markdown skin**: Styled rendering for markdown content
//!
//! ## Example
//!
//! ```rust
//! use ckrv_cli::ui::theme::Theme;
//!
//! let theme = Theme::default();
//! // Use theme.primary_color, theme.success_style, etc.
//! ```

// ============================================================
// IMPORTS
// ============================================================

use console::{Color, Style};
use termimad::{MadSkin, StyledChar};

// ============================================================
// TYPES
// ============================================================

/// Visual theme holding all colors, styles, and rendering configuration.
#[derive(Debug, Clone)]
pub struct Theme {
    /// Primary brand color (Royal Gold, 256-color code 220).
    pub primary_color: Color,
    /// Secondary accent color (Cyan).
    pub secondary_color: Color,
    /// Style for success messages (green + bold).
    pub success_style: Style,
    /// Style for error messages (red + bold).
    pub error_style: Style,
    /// Style for warning messages (yellow).
    pub warning_style: Style,
    /// Unicode box-drawing characters for panel borders.
    pub box_chars: BoxChars,
    /// Styled markdown rendering skin for terminal output.
    pub markdown_skin: MadSkin,
}

/// Unicode box-drawing characters for rendering bordered panels.
#[derive(Debug, Clone)]
pub struct BoxChars {
    /// Top-left corner character.
    pub top_left: char,
    /// Top-right corner character.
    pub top_right: char,
    /// Bottom-left corner character.
    pub bottom_left: char,
    /// Bottom-right corner character.
    pub bottom_right: char,
    /// Horizontal line character.
    pub horizontal: char,
    /// Vertical line character.
    pub vertical: char,
}

// ============================================================
// IMPLEMENTATION
// ============================================================

impl Default for Theme {
    fn default() -> Self {
        use termimad::crossterm::style::{Attribute, Color as CColor};

        // Option A: Hardcoded Premium Theme
        // We use a distinct palette inspired by modern CLIs

        // Brand: Chakravarti (Royal Gold)
        let primary_color = Color::Color256(220);
        let secondary_color = Color::Cyan;

        let success_style = Style::new().green().bold();
        let error_style = Style::new().red().bold();
        let warning_style = Style::new().yellow();

        // Use rounded corners for proper Unicode terminals
        // Fallback or ASCII mode detection will happen at usage level,
        // but Theme defines the "Ideal" state.
        let box_chars = BoxChars {
            top_left: '╭',
            top_right: '╮',
            bottom_left: '╰',
            bottom_right: '╯',
            horizontal: '─',
            vertical: '│',
        };

        let mut skin = MadSkin::default();
        // Map to crossterm colors for Termimad
        let p_cc = CColor::Magenta;
        let s_cc = CColor::Cyan;

        skin.bold.set_fg(p_cc);
        skin.italic.set_fg(s_cc);
        // Bullet points
        skin.bullet = StyledChar::from_fg_char(p_cc, '•');

        // Headers
        skin.headers[0].set_fg(p_cc);
        skin.headers[0].add_attr(Attribute::Bold);
        skin.headers[0].align = termimad::Alignment::Left;

        skin.headers[1].set_fg(s_cc);
        skin.headers[1].add_attr(Attribute::Bold);

        Self {
            primary_color,
            secondary_color,
            success_style,
            error_style,
            warning_style,
            box_chars,
            markdown_skin: skin,
        }
    }
}
