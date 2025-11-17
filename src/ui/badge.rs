//! Badge component for status indicators

use super::theme::Theme;

/// Badge builder for status indicators
pub struct Badge {
    theme: Theme,
}

impl Badge {
    /// Create a new badge builder
    pub fn new(theme: Theme) -> Self {
        Self { theme }
    }

    /// Create a state badge
    pub fn state(&self, state: &str) -> String {
        let (icon, style) = match state.to_lowercase().as_str() {
            "active" | "in progress" | "doing" => {
                let icon = if self.theme.use_emojis() { "→" } else { ">" };
                (icon, &self.theme.info())
            }
            "completed" | "done" | "closed" => {
                let icon = if self.theme.use_emojis() { "✓" } else { "+" };
                (icon, &self.theme.success())
            }
            "blocked" | "waiting" => {
                let icon = if self.theme.use_emojis() { "⚠" } else { "!" };
                (icon, &self.theme.warning())
            }
            "new" | "pending" | "to do" => {
                let icon = if self.theme.use_emojis() { "○" } else { "o" };
                (icon, &self.theme.muted())
            }
            _ => {
                let icon = if self.theme.use_emojis() { "◐" } else { "-" };
                (icon, &self.theme.primary())
            }
        };

        if self.theme.use_colors() {
            format!("[{} {}]", icon, state)
        } else {
            format!("[{} {}]", icon, state)
        }
    }

    /// Create a priority badge
    pub fn priority(&self, priority: &str) -> String {
        let priority_lower = priority.to_lowercase();
        let (icon, text) = match priority_lower.as_str() {
            "critical" | "1" => {
                let icon = if self.theme.use_emojis() { "🔥" } else { "!!" };
                (icon, "Critical")
            }
            "high" | "2" => {
                let icon = if self.theme.use_emojis() { "⚡" } else { "!" };
                (icon, "High")
            }
            "medium" | "3" => {
                let icon = if self.theme.use_emojis() { "◐" } else { "*" };
                (icon, "Medium")
            }
            "low" | "4" => {
                let icon = if self.theme.use_emojis() { "○" } else { "o" };
                (icon, "Low")
            }
            _ => {
                let icon = if self.theme.use_emojis() { "◐" } else { "-" };
                (icon, priority)
            }
        };

        if priority_lower == "critical" || priority_lower == "1" {
            self.theme.fmt_error(&format!("[{} {}]", icon, text))
        } else if priority_lower == "high" || priority_lower == "2" {
            self.theme.fmt_warning(&format!("[{} {}]", icon, text))
        } else {
            self.theme.fmt_muted(&format!("[{} {}]", icon, text))
        }
    }

    /// Create a work item type badge
    pub fn work_item_type(&self, item_type: &str) -> String {
        let (icon, _color) = match item_type.to_lowercase().as_str() {
            "bug" => {
                let icon = if self.theme.use_emojis() { "🐛" } else { "B" };
                (icon, &self.theme.error())
            }
            "feature" | "user story" => {
                let icon = if self.theme.use_emojis() { "✨" } else { "F" };
                (icon, &self.theme.info())
            }
            "task" => {
                let icon = if self.theme.use_emojis() { "📝" } else { "T" };
                (icon, &self.theme.primary())
            }
            "improvement" | "enhancement" => {
                let icon = if self.theme.use_emojis() { "🔧" } else { "I" };
                (icon, &self.theme.accent())
            }
            "epic" => {
                let icon = if self.theme.use_emojis() { "📚" } else { "E" };
                (icon, &self.theme.accent())
            }
            _ => {
                let icon = if self.theme.use_emojis() { "📋" } else { "*" };
                (icon, &self.theme.primary())
            }
        };

        if self.theme.use_colors() {
            self.theme.fmt_info(&format!("[{} {}]", icon, item_type))
        } else {
            format!("[{} {}]", icon, item_type)
        }
    }

    /// Create a validation status badge
    pub fn validation(&self, passed: bool) -> String {
        if passed {
            let icon = if self.theme.use_emojis() { "✓" } else { "+" };
            self.theme.fmt_success(&format!("[{} Valid]", icon))
        } else {
            let icon = if self.theme.use_emojis() { "✗" } else { "x" };
            self.theme.fmt_error(&format!("[{} Invalid]", icon))
        }
    }

    /// Create a count badge
    pub fn count(&self, label: &str, count: usize) -> String {
        let icon = if self.theme.use_emojis() {
            match label.to_lowercase().as_str() {
                "attachments" => "📎",
                "comments" => "💬",
                "images" => "🖼️",
                "requirements" => "📋",
                _ => "•",
            }
        } else {
            "*"
        };

        if count == 0 {
            self.theme.fmt_muted(&format!("{} {} {}", icon, count, label))
        } else {
            self.theme.fmt_primary(&format!("{} {} {}", icon, count, label))
        }
    }

    /// Create an inline badge (minimal style)
    pub fn inline(&self, icon: &str, text: &str) -> String {
        let icon_str = if self.theme.use_emojis() {
            icon
        } else {
            match icon {
                "✓" | "✅" => "+",
                "✗" | "❌" => "x",
                "⚠️" => "!",
                "→" => ">",
                "○" => "o",
                _ => "*",
            }
        };

        format!("{} {}", icon_str, text)
    }
}
