//! Progress indicators and spinners

use indicatif::{ProgressBar, ProgressStyle};
use std::time::Duration;
use super::theme::Theme;

/// Progress indicator builder
pub struct Progress {
    theme: Theme,
}

impl Progress {
    /// Create a new progress indicator
    pub fn new(theme: Theme) -> Self {
        Self { theme }
    }

    /// Create a spinner for indeterminate progress
    pub fn spinner(&self, message: &str) -> ProgressBar {
        let pb = ProgressBar::new_spinner();

        if self.theme.use_animations() {
            let style = if self.theme.use_emojis() {
                ProgressStyle::default_spinner()
                    .tick_strings(&["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"])
                    .template("{spinner:.cyan} {msg}")
                    .unwrap()
            } else {
                ProgressStyle::default_spinner()
                    .tick_strings(&["-", "\\", "|", "/"])
                    .template("{spinner} {msg}")
                    .unwrap()
            };

            pb.set_style(style);
            pb.set_message(message.to_string());
            pb.enable_steady_tick(Duration::from_millis(80));
        } else {
            // No animation for print/compact modes
            pb.set_draw_target(indicatif::ProgressDrawTarget::hidden());
        }

        pb
    }

    /// Create a progress bar for determinate progress
    pub fn bar(&self, len: u64, message: &str) -> ProgressBar {
        let pb = ProgressBar::new(len);

        if self.theme.use_animations() {
            let style = if self.theme.use_emojis() {
                ProgressStyle::default_bar()
                    .template("{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {pos}/{len} {msg}")
                    .unwrap()
                    .progress_chars("█▓▒░")
            } else {
                ProgressStyle::default_bar()
                    .template("[{elapsed_precise}] [{bar:40}] {pos}/{len} {msg}")
                    .unwrap()
                    .progress_chars("##-")
            };

            pb.set_style(style);
            pb.set_message(message.to_string());
        } else {
            pb.set_draw_target(indicatif::ProgressDrawTarget::hidden());
        }

        pb
    }

    /// Show a simple status message
    pub fn status(&self, icon: &str, message: &str) {
        if self.theme.mode == super::theme::OutputMode::Print {
            return;
        }

        let icon_str = if self.theme.use_emojis() {
            icon
        } else {
            match icon {
                "🔄" => ">",
                "✓" | "✅" => "+",
                "❌" => "x",
                "⚠️" => "!",
                "📋" => "*",
                "🤖" => "AI",
                _ => "*",
            }
        };

        println!("{} {}",
            self.theme.fmt_info(icon_str),
            self.theme.fmt_primary(message)
        );
    }
}
