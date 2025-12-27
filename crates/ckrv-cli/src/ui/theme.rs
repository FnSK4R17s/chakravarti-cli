use console::{Color, Style};
use termimad::{MadSkin, StyledChar};

#[derive(Debug, Clone)]
pub struct Theme {
    pub primary_color: Color,
    pub secondary_color: Color,
    pub success_style: Style,
    pub error_style: Style,
    pub warning_style: Style,
    pub box_chars: BoxChars,
    pub markdown_skin: MadSkin,
}

#[derive(Debug, Clone)]
pub struct BoxChars {
    pub top_left: char,
    pub top_right: char,
    pub bottom_left: char,
    pub bottom_right: char,
    pub horizontal: char,
    pub vertical: char,
}

impl Default for Theme {
    fn default() -> Self {
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
        use termimad::crossterm::style::{Attribute, Color as CColor};
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
