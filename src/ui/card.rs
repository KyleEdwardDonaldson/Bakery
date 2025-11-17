//! Information card component for displaying structured data

use super::theme::Theme;
use super::terminal::Terminal;

/// Card component for displaying information in bordered boxes
pub struct Card {
    theme: Theme,
    terminal: Terminal,
}

impl Card {
    /// Create a new card
    pub fn new(theme: Theme, terminal: Terminal) -> Self {
        Self { theme, terminal }
    }

    /// Render a simple card with a title and content
    pub fn render(&self, title: &str, lines: Vec<String>) {
        if self.theme.mode == super::theme::OutputMode::Print {
            // Skip decorative output in print mode
            return;
        }

        let box_chars = self.terminal.box_chars();
        let max_width = if self.terminal.is_narrow() {
            60
        } else if self.terminal.is_wide() {
            120
        } else {
            80
        };

        // Calculate actual content width
        let mut content_width = title.len();
        for line in &lines {
            let line_width = unicode_width::UnicodeWidthStr::width(line.as_str());
            if line_width > content_width {
                content_width = line_width;
            }
        }
        content_width = content_width.min(max_width - 4); // Account for borders and padding

        // Top border with title
        let title_section = format!(" {} ", title);
        let title_width = unicode_width::UnicodeWidthStr::width(title_section.as_str());
        let remaining = if content_width > title_width {
            content_width - title_width
        } else {
            4
        };

        println!("{}{}{}{}",
            box_chars.top_left,
            box_chars.horizontal.repeat(1),
            self.theme.fmt_highlight(&title_section),
            box_chars.horizontal.repeat(remaining + 1).to_string() + box_chars.top_right
        );

        // Content lines
        for line in lines {
            let line_width = unicode_width::UnicodeWidthStr::width(line.as_str());
            let padding = if content_width > line_width {
                content_width - line_width
            } else {
                0
            };

            println!("{} {}{} {}",
                box_chars.vertical,
                line,
                " ".repeat(padding),
                box_chars.vertical
            );
        }

        // Bottom border
        println!("{}{}{}",
            box_chars.bottom_left,
            box_chars.horizontal.repeat(content_width + 2),
            box_chars.bottom_right
        );
    }

    /// Render a simple box (like the AI generation box)
    pub fn render_box(&self, text: &str, width: usize) {
        if self.theme.mode == super::theme::OutputMode::Print {
            return;
        }

        let box_chars = self.terminal.box_chars();

        println!("\n{}{}{}",
            box_chars.top_left,
            box_chars.horizontal.repeat(width - 2),
            box_chars.top_right
        );

        // Center the text
        let text_width = unicode_width::UnicodeWidthStr::width(text);
        let padding_left = (width - text_width - 2) / 2;
        let padding_right = width - text_width - padding_left - 2;

        println!("{}{}{}{}{}",
            box_chars.vertical,
            " ".repeat(padding_left),
            text,
            " ".repeat(padding_right),
            box_chars.vertical
        );

        println!("{}{}{}",
            box_chars.bottom_left,
            box_chars.horizontal.repeat(width - 2),
            box_chars.bottom_right
        );
    }

    /// Render a two-column layout
    pub fn render_two_column(&self, pairs: Vec<(&str, String)>) {
        if self.theme.mode == super::theme::OutputMode::Print {
            return;
        }

        let label_width = pairs.iter()
            .map(|(label, _)| unicode_width::UnicodeWidthStr::width(*label))
            .max()
            .unwrap_or(10);

        for (label, value) in pairs {
            let padded_label = format!("{:width$}", label, width = label_width);
            println!("  {} {}",
                self.theme.fmt_muted(&padded_label),
                self.theme.fmt_primary(&value)
            );
        }
    }

    /// Render a compact header
    pub fn render_header(&self, title: &str, subtitle: &str) {
        if self.theme.mode == super::theme::OutputMode::Print {
            return;
        }

        println!("\n{}", self.theme.fmt_highlight(title));
        if !subtitle.is_empty() {
            println!("{}", self.theme.fmt_muted(subtitle));
        }
        println!();
    }
}
