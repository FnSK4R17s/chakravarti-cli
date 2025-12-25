use crate::ui::theme::Theme;
use crate::ui::Renderable;
use console::Style;

pub struct Banner {
    pub title: String, // Kept for future dynamic use, currently ignoring for hardcoded art
    pub subtitle: Option<String>,
}

impl Banner {
    pub fn new(_title: impl Into<String>) -> Self {
        Self {
            title: _title.into(),
            subtitle: None,
        }
    }

    pub fn subtitle(mut self, sub: impl Into<String>) -> Self {
        self.subtitle = Some(sub.into());
        self
    }
}

impl Renderable for Banner {
    fn render(&self, theme: &Theme) -> String {
        // Hardcoded ASCII art for 'CHAKRAVARTI'
        // Using a slant or block font style
        // ANSI Shadow font for "CKRV"
        let art = r#"
 ██████╗██╗  ██╗██████╗ ██╗   ██╗
██╔════╝██║ ██╔╝██╔══██╗██║   ██║
██║     █████╔╝ ██████╔╝██║   ██║
██║     ██╔═██╗ ██╔══██╗╚██╗ ██╔╝
╚██████╗██║  ██╗██║  ██║ ╚████╔╝ 
 ╚═════╝╚═╝  ╚═╝╚═╝  ╚═╝  ╚═══╝  "#;
        
        // Royal Gold -> Orange -> Red Gradient
        // We Use trim_matches('\n') to remove the first/last newlines from the raw string
        // while PRESERVING the leading space on the first line of the art!
        let lines: Vec<&str> = art.trim_matches('\n').lines().collect();
        let center_pad = "  "; // simple padding
        
        let mut colored_art = String::new();
        
        let steps = lines.len();
        for (i, line) in lines.iter().enumerate() {
             // Option 2 (Pale Gold): Wheat -> Gold -> Orange
             // Creates a lighter, more royal gold appearance at the top
             let color_code = match i {
                 0 => 229, // Wheat1
                 1 => 228, // Khaki1
                 2 => 220, // Gold1
                 3 => 214, // Orange1
                 _ => 208, // DarkOrange
             };
             
             let style = Style::new().fg(console::Color::Color256(color_code)).bold();
             colored_art.push_str(&format!("{}{}\n", center_pad, style.apply_to(line)));
        }

        match &self.subtitle {
            Some(s) => {
                // Gold subtitle
                let sub_style = Style::new().fg(console::Color::Color256(220)).italic();
                format!("{}\n    {}", colored_art, sub_style.apply_to(s))
            },
            None => colored_art,
        }
    }
}

pub struct RichTable {
    pub inner: tabled::Table,
}

impl RichTable {
    pub fn new(table: tabled::Table) -> Self {
        Self { inner: table }
    }
}

impl Renderable for RichTable {
    fn render(&self, theme: &Theme) -> String {
        let mut table = self.inner.clone();
        
        // Apply styling based on theme
        // Tabled's Style is different from console's Style.
        // We'll use Modern style as base.
        let mut style = tabled::settings::Style::modern();
        
        // To strictly usage theme colors for borders is complex in tabled v0.14+
        // We can create a theme-based border style using Theme.box_chars.
        // For MVP, just use modern style.
        
        table.with(style);
        
        // Color headers?
        // tabled requires object safety tweaks or separate `Color` settings.
        // This is complex to do generically without generic types.
        // We'll trust standard implementation for now.
        
        table.to_string()
    }
}

pub enum PanelLevel {
    Info,
    Success,
    Error,
}

pub struct Panel {
    pub title: String,
    pub content: String,
    pub level: PanelLevel,
}

impl Panel {
    pub fn new(title: impl Into<String>, content: impl Into<String>) -> Self {
        Self {
            title: title.into(),
            content: content.into(),
            level: PanelLevel::Info,
        }
    }
    
    pub fn success(mut self) -> Self { self.level = PanelLevel::Success; self }
    pub fn error(mut self) -> Self { self.level = PanelLevel::Error; self }
}

impl Renderable for Panel {
    fn render(&self, theme: &Theme) -> String {
        let (color, icon) = match self.level {
            PanelLevel::Info => (theme.secondary_color, "ℹ"),
            PanelLevel::Success => (console::Color::Green, "✔"), // use Theme.success_style?
            PanelLevel::Error => (console::Color::Red, "✖"),
        };
        
        let style = Style::new().fg(color).bold();
        let border_style = Style::new().fg(color);
        
        // Simple Box drawing manually to control colors easily
        // ╭── Title ──╮
        // │  Content  │
        // ╰───────────╯
        // Actually, let's use a simpler "Block Quote" style for panels to save vertical space?
        // Spec/UserStory: "colored panel/box".
        
        let title_line = format!("{} {} ", icon, self.title);
        let title_styled = style.apply_to(title_line);
        let content_styled = self.content.replace('\n', "\n  ");
        
        // Let's rely on termimad or simple indentation with a colored bar on the left?
        // Or a full box.
        // Let's do a left-border panel (Cloud style).
        // ▌ Title
        // ▌ Content
        
        let bar = border_style.apply_to("▌");
        format!("{} {}\n{} {}", bar, title_styled, bar, content_styled)
    }
}
