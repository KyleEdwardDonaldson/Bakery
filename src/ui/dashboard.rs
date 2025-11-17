//! Summary dashboard component

use super::theme::Theme;
use super::terminal::Terminal;
use super::badge::Badge;
use super::format::{format_file_size, format_duration, format_time_ago};

/// Dashboard for displaying summary information
pub struct Dashboard {
    theme: Theme,
    terminal: Terminal,
    badge: Badge,
}

impl Dashboard {
    /// Create a new dashboard
    pub fn new(theme: Theme, terminal: Terminal) -> Self {
        let badge = Badge::new(theme.clone());
        Self { theme, terminal, badge }
    }

    /// Render a work item summary dashboard
    pub fn render_work_item_summary(
        &self,
        id: u32,
        title: &str,
        state: &str,
        work_item_type: &str,
        attachments: usize,
        comments: usize,
        images: usize,
        acceptance_criteria: usize,
    ) {
        if self.theme.mode == super::theme::OutputMode::Print {
            return;
        }

        let box_chars = self.terminal.box_chars();
        let width = if self.terminal.is_narrow() {
            50
        } else if self.terminal.is_wide() {
            90
        } else {
            70
        };

        // Top border
        let header = format!(" 📋 Work Item #{} ", id);
        println!("\n{}{}{}",
            box_chars.top_left,
            self.theme.fmt_highlight(&header),
            box_chars.horizontal.repeat(width - header.len() - 1).to_string() + box_chars.top_right
        );

        // Title
        let title_display = if title.len() > width - 6 {
            format!("{}...", &title[..width - 9])
        } else {
            title.to_string()
        };
        println!("{} {} {}",
            box_chars.vertical,
            self.theme.fmt_primary(&title_display),
            " ".repeat(width - title_display.len() - 3).to_string() + box_chars.vertical
        );

        // Status line with badges
        let state_badge = self.badge.state(state);
        let type_badge = self.badge.work_item_type(work_item_type);
        let status_line = format!("{} {}", state_badge, type_badge);
        println!("{} {} {}",
            box_chars.vertical,
            status_line,
            " ".repeat(width - status_line.len() - 3).to_string() + box_chars.vertical
        );

        // Separator
        println!("{}{}{}",
            box_chars.left_join,
            box_chars.horizontal.repeat(width - 2),
            box_chars.right_join
        );

        // Content counts
        let content_lines = vec![
            self.badge.count("attachments", attachments),
            self.badge.count("comments", comments),
            self.badge.count("images", images),
            self.badge.count("acceptance criteria", acceptance_criteria),
        ];

        for line in content_lines {
            println!("{} {} {}",
                box_chars.vertical,
                line,
                " ".repeat(width - line.len() - 3).to_string() + box_chars.vertical
            );
        }

        // Bottom border
        println!("{}{}{}",
            box_chars.bottom_left,
            box_chars.horizontal.repeat(width - 2),
            box_chars.bottom_right
        );
    }

    /// Render OpenSpec generation summary
    pub fn render_openspec_summary(&self, change_path: &str, validation_passed: bool, requirement_count: usize) {
        if self.theme.mode == super::theme::OutputMode::Print {
            return;
        }

        let validation_badge = self.badge.validation(validation_passed);
        let req_badge = self.badge.count("requirement(s)", requirement_count);

        println!("{} {}", validation_badge, req_badge);
        println!("{} {}",
            self.theme.fmt_info("📁"),
            self.theme.fmt_primary(change_path)
        );
    }

    /// Render operation completion summary
    pub fn render_completion(&self, operation: &str, duration: f64) {
        if self.theme.mode == super::theme::OutputMode::Print {
            return;
        }

        let duration_str = format_duration(duration);
        println!("\n{} {} {}",
            self.theme.fmt_success("✓"),
            self.theme.fmt_highlight(operation),
            self.theme.fmt_muted(&format!("({})", duration_str))
        );
    }

    /// Render next steps
    pub fn render_next_steps(&self, commands: Vec<&str>) {
        if self.theme.mode == super::theme::OutputMode::Print || self.theme.mode == super::theme::OutputMode::Verbose {
            return;
        }

        if commands.is_empty() {
            return;
        }

        print!("\n{} ", self.theme.fmt_primary("Next:"));
        for (i, cmd) in commands.iter().enumerate() {
            if i > 0 {
                print!(" {} ", self.theme.fmt_muted("or"));
            }
            print!("{}", self.theme.fmt_info(cmd));
        }
        println!();
    }

    /// Render error card
    pub fn render_error(&self, title: &str, message: &str, suggestion: Option<&str>) {
        let box_chars = self.terminal.box_chars();
        let width = if self.terminal.is_narrow() {
            50
        } else {
            70
        };

        // Top border with error icon
        let header = format!(" ❌ {} ", title);
        println!("\n{}{}{}",
            box_chars.top_left,
            self.theme.fmt_error(&header),
            box_chars.horizontal.repeat(width - header.len() - 1).to_string() + box_chars.top_right
        );

        // Error message (wrapped if needed)
        let wrapped_lines = super::format::wrap_text(message, width - 4);
        for line in wrapped_lines {
            println!("{} {} {}",
                box_chars.vertical,
                self.theme.fmt_primary(&line),
                " ".repeat(width - line.len() - 3).to_string() + box_chars.vertical
            );
        }

        // Suggestion if provided
        if let Some(sug) = suggestion {
            println!("{}{}{}",
                box_chars.left_join,
                box_chars.horizontal.repeat(width - 2),
                box_chars.right_join
            );

            let suggestion_header = self.theme.fmt_info("💡 Suggestion:");
            println!("{} {} {}",
                box_chars.vertical,
                suggestion_header,
                " ".repeat(width - 14).to_string() + box_chars.vertical
            );

            let wrapped_sug = super::format::wrap_text(sug, width - 4);
            for line in wrapped_sug {
                println!("{} {} {}",
                    box_chars.vertical,
                    line,
                    " ".repeat(width - line.len() - 3).to_string() + box_chars.vertical
                );
            }
        }

        // Bottom border
        println!("{}{}{}",
            box_chars.bottom_left,
            box_chars.horizontal.repeat(width - 2),
            box_chars.bottom_right
        );
    }
}
