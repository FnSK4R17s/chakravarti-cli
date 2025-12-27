pub mod components;
pub mod spinner;
pub mod terminal;
pub mod theme;

use terminal::should_enable_rich_ui;
pub use theme::Theme;

/// The main entry point for interaction with the CLI UI.
/// It holds the theme and determines whether to render rich output.
pub struct UiContext {
    pub theme: Theme,
    /// Whether rich interaction (Spinners, Colors) is allowed.
    pub is_interactive: bool,
    /// If true, we are in "Silent Mode" (e.g. JSON output), suppressing banners/decorations.
    pub silent: bool,
}

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

    /// Display an error panel.
    pub fn error(&self, title: &str, msg: &str) {
        self.print(components::Panel::new(title, msg).error());
    }

    /// Start a spinner.
    pub fn spinner(&self, msg: impl Into<String>) -> spinner::SpinnerGuard {
        spinner::SpinnerGuard::new(&msg.into(), self.is_interactive, &self.theme)
    }
}

pub trait Renderable {
    /// Render the component to a string, respecting the provided theme.
    fn render(&self, theme: &Theme) -> String;
}
