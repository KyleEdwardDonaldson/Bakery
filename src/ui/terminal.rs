//! Terminal capability detection and management

use crossterm::{
    terminal::{size, Clear, ClearType},
    style::Color as CrosstermColor,
};
use std::io::{self, Write};

/// Terminal capabilities and state
#[derive(Clone)]
pub struct Terminal {
    pub width: u16,
    pub height: u16,
    pub supports_unicode: bool,
    pub supports_color: bool,
    pub color_depth: ColorDepth,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ColorDepth {
    None,
    Basic16,
    Color256,
    TrueColor,
}

impl Terminal {
    /// Detect terminal capabilities
    pub fn detect() -> Self {
        let (width, height) = size().unwrap_or((80, 24));

        // Check Unicode support
        let supports_unicode = Self::check_unicode_support();

        // Check color support
        let (supports_color, color_depth) = Self::detect_color_support();

        Self {
            width,
            height,
            supports_unicode,
            supports_color,
            color_depth,
        }
    }

    /// Check if terminal supports Unicode
    fn check_unicode_support() -> bool {
        // Check LANG and LC_ALL environment variables
        if let Ok(lang) = std::env::var("LANG") {
            if lang.to_lowercase().contains("utf") {
                return true;
            }
        }

        if let Ok(lc_all) = std::env::var("LC_ALL") {
            if lc_all.to_lowercase().contains("utf") {
                return true;
            }
        }

        // Windows Terminal and modern terminals support Unicode by default
        #[cfg(windows)]
        {
            if std::env::var("WT_SESSION").is_ok() || std::env::var("TERM_PROGRAM").is_ok() {
                return true;
            }
        }

        // Default to true for modern systems
        true
    }

    /// Detect color support level
    fn detect_color_support() -> (bool, ColorDepth) {
        // Check NO_COLOR environment variable
        if std::env::var("NO_COLOR").is_ok() {
            return (false, ColorDepth::None);
        }

        // Check COLORTERM for truecolor support
        if let Ok(colorterm) = std::env::var("COLORTERM") {
            if colorterm.contains("truecolor") || colorterm.contains("24bit") {
                return (true, ColorDepth::TrueColor);
            }
        }

        // Check TERM variable
        if let Ok(term) = std::env::var("TERM") {
            let term_lower = term.to_lowercase();
            if term_lower.contains("256color") {
                return (true, ColorDepth::Color256);
            }
            if term_lower.contains("color") {
                return (true, ColorDepth::Basic16);
            }
            if term == "dumb" {
                return (false, ColorDepth::None);
            }
        }

        // Windows Terminal supports truecolor
        #[cfg(windows)]
        {
            if std::env::var("WT_SESSION").is_ok() {
                return (true, ColorDepth::TrueColor);
            }
        }

        // Default to 256 color for modern terminals
        (true, ColorDepth::Color256)
    }

    /// Get box drawing characters based on Unicode support
    pub fn box_chars(&self) -> BoxChars {
        if self.supports_unicode {
            BoxChars::unicode()
        } else {
            BoxChars::ascii()
        }
    }

    /// Check if terminal is narrow
    pub fn is_narrow(&self) -> bool {
        self.width < 80
    }

    /// Check if terminal is wide
    pub fn is_wide(&self) -> bool {
        self.width > 120
    }

    /// Clear the current line
    pub fn clear_line() -> io::Result<()> {
        let mut stdout = io::stdout();
        crossterm::execute!(stdout, Clear(ClearType::CurrentLine))?;
        stdout.flush()
    }
}

/// Box drawing characters
#[derive(Clone)]
pub struct BoxChars {
    pub horizontal: &'static str,
    pub vertical: &'static str,
    pub top_left: &'static str,
    pub top_right: &'static str,
    pub bottom_left: &'static str,
    pub bottom_right: &'static str,
    pub top_join: &'static str,
    pub bottom_join: &'static str,
    pub left_join: &'static str,
    pub right_join: &'static str,
    pub cross: &'static str,
}

impl BoxChars {
    /// Unicode box drawing characters
    pub fn unicode() -> Self {
        Self {
            horizontal: "─",
            vertical: "│",
            top_left: "┌",
            top_right: "┐",
            bottom_left: "└",
            bottom_right: "┘",
            top_join: "┬",
            bottom_join: "┴",
            left_join: "├",
            right_join: "┤",
            cross: "┼",
        }
    }

    /// ASCII fallback characters
    pub fn ascii() -> Self {
        Self {
            horizontal: "-",
            vertical: "|",
            top_left: "+",
            top_right: "+",
            bottom_left: "+",
            bottom_right: "+",
            top_join: "+",
            bottom_join: "+",
            left_join: "+",
            right_join: "+",
            cross: "+",
        }
    }
}
