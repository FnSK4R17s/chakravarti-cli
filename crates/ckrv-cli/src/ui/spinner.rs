use indicatif::{ProgressBar, ProgressStyle, ProgressDrawTarget};
use std::time::Duration;
use crate::ui::theme::Theme;

pub struct SpinnerGuard {
    inner: Option<ProgressBar>,
}

impl SpinnerGuard {
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
            .tick_strings(&[
                "⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"
            ])
            .template("{spinner:.cyan} {msg}")
            .unwrap_or_else(|_| ProgressStyle::default_spinner());

        pb.set_style(style);

        Self { inner: Some(pb) }
    }

    pub fn set_message(&self, msg: &str) {
        if let Some(pb) = &self.inner {
            pb.set_message(msg.to_string());
        }
    }

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

    pub fn error(&self, msg: &str) {
        if let Some(pb) = &self.inner {
            pb.finish_with_message(format!("{} {}", "✖", msg));
        }
    }
    
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
