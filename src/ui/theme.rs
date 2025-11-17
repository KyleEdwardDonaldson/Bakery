//! Color theme and output mode configuration

use owo_colors::{OwoColorize, Style, colors::*};
use super::terminal::{Terminal, ColorDepth};

/// Output mode for different use cases
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum OutputMode {
    /// Clean output with subtle animations and colors (default)
    Default,
    /// Detailed logging with expandable sections
    Verbose,
    /// Machine-readable format for automation
    Print,
    /// Maximum visual features and animations
    Rich,
    /// Minimal output for constrained environments
    Compact,
    /// Monochrome output for compatibility
    NoColor,
}

/// Theme with semantic colors
#[derive(Clone)]
pub struct Theme {
    pub mode: OutputMode,
    terminal: Terminal,
}

impl Theme {
    /// Create a new theme with output mode
    pub fn new(mode: OutputMode, terminal: Terminal) -> Self {
        Self { mode, terminal }
    }

    /// Check if colors should be used
    pub fn use_colors(&self) -> bool {
        self.mode != OutputMode::NoColor
            && self.mode != OutputMode::Print
            && self.terminal.supports_color
    }

    /// Check if emojis should be used
    pub fn use_emojis(&self) -> bool {
        self.mode != OutputMode::Print
            && self.mode != OutputMode::Compact
            && self.terminal.supports_unicode
    }

    /// Check if animations should be used
    pub fn use_animations(&self) -> bool {
        self.mode == OutputMode::Default || self.mode == OutputMode::Rich
    }

    /// Get success color style
    pub fn success(&self) -> Style {
        if self.use_colors() {
            Style::new().bright_green()
        } else {
            Style::new()
        }
    }

    /// Get warning color style
    pub fn warning(&self) -> Style {
        if self.use_colors() {
            Style::new().bright_yellow()
        } else {
            Style::new()
        }
    }

    /// Get error color style
    pub fn error(&self) -> Style {
        if self.use_colors() {
            Style::new().bright_red()
        } else {
            Style::new()
        }
    }

    /// Get info color style
    pub fn info(&self) -> Style {
        if self.use_colors() {
            Style::new().bright_cyan()
        } else {
            Style::new()
        }
    }

    /// Get muted/secondary color style
    pub fn muted(&self) -> Style {
        if self.use_colors() {
            Style::new().bright_black()
        } else {
            Style::new()
        }
    }

    /// Get accent color style
    pub fn accent(&self) -> Style {
        if self.use_colors() {
            Style::new().bright_magenta()
        } else {
            Style::new()
        }
    }

    /// Get primary text color style
    pub fn primary(&self) -> Style {
        if self.use_colors() {
            Style::new().bright_white()
        } else {
            Style::new()
        }
    }

    /// Get highlighted text style
    pub fn highlight(&self) -> Style {
        if self.use_colors() {
            Style::new().bright_white().bold()
        } else {
            Style::new().bold()
        }
    }

    /// Get dim text style
    pub fn dim(&self) -> Style {
        if self.use_colors() {
            Style::new().dimmed()
        } else {
            Style::new()
        }
    }

    /// Get bold text style
    pub fn bold(&self) -> Style {
        Style::new().bold()
    }

    /// Format success text
    pub fn fmt_success(&self, text: &str) -> String {
        if self.use_colors() {
            text.bright_green().to_string()
        } else {
            text.to_string()
        }
    }

    /// Format warning text
    pub fn fmt_warning(&self, text: &str) -> String {
        if self.use_colors() {
            text.bright_yellow().to_string()
        } else {
            text.to_string()
        }
    }

    /// Format error text
    pub fn fmt_error(&self, text: &str) -> String {
        if self.use_colors() {
            text.bright_red().to_string()
        } else {
            text.to_string()
        }
    }

    /// Format info text
    pub fn fmt_info(&self, text: &str) -> String {
        if self.use_colors() {
            text.bright_cyan().to_string()
        } else {
            text.to_string()
        }
    }

    /// Format muted text
    pub fn fmt_muted(&self, text: &str) -> String {
        if self.use_colors() {
            text.bright_black().to_string()
        } else {
            text.to_string()
        }
    }

    /// Format primary text
    pub fn fmt_primary(&self, text: &str) -> String {
        if self.use_colors() {
            text.bright_white().to_string()
        } else {
            text.to_string()
        }
    }

    /// Format highlighted text
    pub fn fmt_highlight(&self, text: &str) -> String {
        if self.use_colors() {
            text.bright_white().bold().to_string()
        } else {
            text.to_string()
        }
    }

    /// Format accent text
    pub fn fmt_accent(&self, text: &str) -> String {
        if self.use_colors() {
            text.bright_magenta().to_string()
        } else {
            text.to_string()
        }
    }
}
