//! Text formatting utilities

use chrono::{DateTime, Utc, Local, Duration};
use unicode_width::UnicodeWidthStr;

/// Truncate text intelligently while preserving word boundaries
pub fn truncate_text(text: &str, max_len: usize) -> String {
    if text.len() <= max_len {
        return text.to_string();
    }

    // Try to find a word boundary
    let mut truncate_pos = max_len;
    if let Some(pos) = text[..max_len].rfind(' ') {
        truncate_pos = pos;
    }

    let remaining = text.len() - truncate_pos;
    format!("{}... ({} more chars)", &text[..truncate_pos], remaining)
}

/// Truncate text to first paragraph or N characters
pub fn truncate_to_paragraph(text: &str, max_chars: usize) -> String {
    // Find first paragraph break
    if let Some(pos) = text.find("\n\n") {
        if pos < max_chars {
            let remaining = text.len() - pos;
            return format!("{}\n... ({} more chars)", &text[..pos].trim(), remaining);
        }
    }

    // No paragraph break, use character limit
    truncate_text(text, max_chars)
}

/// Format timestamp as relative time (e.g., "2 hours ago")
pub fn format_time_ago(timestamp: &str) -> String {
    // Try to parse the timestamp
    if let Ok(dt) = DateTime::parse_from_rfc3339(timestamp) {
        let now = Utc::now();
        let duration = now.signed_duration_since(dt.with_timezone(&Utc));

        if duration.num_seconds() < 60 {
            return "just now".to_string();
        } else if duration.num_minutes() < 60 {
            let mins = duration.num_minutes();
            return format!("{} minute{} ago", mins, if mins == 1 { "" } else { "s" });
        } else if duration.num_hours() < 24 {
            let hours = duration.num_hours();
            return format!("{} hour{} ago", hours, if hours == 1 { "" } else { "s" });
        } else if duration.num_days() < 7 {
            let days = duration.num_days();
            return format!("{} day{} ago", days, if days == 1 { "" } else { "s" });
        } else if duration.num_weeks() < 4 {
            let weeks = duration.num_weeks();
            return format!("{} week{} ago", weeks, if weeks == 1 { "" } else { "s" });
        } else if duration.num_days() < 365 {
            // Format as date
            let local_dt = dt.with_timezone(&Local);
            return local_dt.format("%b %d, %Y").to_string();
        } else {
            // Format with year
            let local_dt = dt.with_timezone(&Local);
            return local_dt.format("%b %d, %Y").to_string();
        }
    }

    // Fallback: return original timestamp
    timestamp.to_string()
}

/// Check if timestamp is recent (within 24 hours)
pub fn is_recent(timestamp: &str) -> bool {
    if let Ok(dt) = DateTime::parse_from_rfc3339(timestamp) {
        let now = Utc::now();
        let duration = now.signed_duration_since(dt.with_timezone(&Utc));
        duration.num_hours() < 24
    } else {
        false
    }
}

/// Format file size with appropriate units
pub fn format_file_size(bytes: u64) -> String {
    const KB: u64 = 1024;
    const MB: u64 = KB * 1024;
    const GB: u64 = MB * 1024;

    if bytes >= GB {
        format!("{:.2} GB", bytes as f64 / GB as f64)
    } else if bytes >= MB {
        format!("{:.2} MB", bytes as f64 / MB as f64)
    } else if bytes >= KB {
        format!("{:.2} KB", bytes as f64 / KB as f64)
    } else {
        format!("{} B", bytes)
    }
}

/// Format duration in seconds to human readable format
pub fn format_duration(seconds: f64) -> String {
    if seconds < 1.0 {
        format!("{}ms", (seconds * 1000.0) as u32)
    } else if seconds < 60.0 {
        format!("{:.1}s", seconds)
    } else if seconds < 3600.0 {
        let mins = (seconds / 60.0) as u32;
        let secs = (seconds % 60.0) as u32;
        format!("{}m {}s", mins, secs)
    } else {
        let hours = (seconds / 3600.0) as u32;
        let mins = ((seconds % 3600.0) / 60.0) as u32;
        format!("{}h {}m", hours, mins)
    }
}

/// Pad text to width, respecting Unicode width
pub fn pad_to_width(text: &str, width: usize) -> String {
    let text_width = UnicodeWidthStr::width(text);
    if text_width >= width {
        text.to_string()
    } else {
        format!("{}{}", text, " ".repeat(width - text_width))
    }
}

/// Align text to center within width
pub fn center_text(text: &str, width: usize) -> String {
    let text_width = UnicodeWidthStr::width(text);
    if text_width >= width {
        text.to_string()
    } else {
        let padding = width - text_width;
        let left_pad = padding / 2;
        let right_pad = padding - left_pad;
        format!("{}{}{}", " ".repeat(left_pad), text, " ".repeat(right_pad))
    }
}

/// Wrap text to specified width
pub fn wrap_text(text: &str, width: usize) -> Vec<String> {
    let mut lines = Vec::new();
    let mut current_line = String::new();
    let mut current_width = 0;

    for word in text.split_whitespace() {
        let word_width = UnicodeWidthStr::width(word);

        if current_width + word_width + 1 > width {
            if !current_line.is_empty() {
                lines.push(current_line);
                current_line = String::new();
                current_width = 0;
            }
        }

        if !current_line.is_empty() {
            current_line.push(' ');
            current_width += 1;
        }

        current_line.push_str(word);
        current_width += word_width;
    }

    if !current_line.is_empty() {
        lines.push(current_line);
    }

    lines
}
