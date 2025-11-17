//! Terminal UI Components
//!
//! This module provides beautiful, informative terminal output components
//! with responsive layouts, semantic colors, and graceful degradation.

pub mod theme;
pub mod terminal;
pub mod card;
pub mod badge;
pub mod progress;
pub mod format;
pub mod dashboard;

// Re-exports for convenience
pub use theme::{Theme, OutputMode};
pub use terminal::Terminal;
pub use card::Card;
pub use badge::Badge;
pub use progress::Progress;
pub use format::{truncate_text, format_time_ago, format_file_size};
pub use dashboard::Dashboard;
