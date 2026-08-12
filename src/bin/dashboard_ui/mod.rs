// src/bin/dashboard_ui/mod.rs

pub mod layout;
pub mod render_by_file;
pub mod render_charts;
pub mod render_dialogs;
pub mod render_help;
pub mod render_history;
pub mod render_list;
pub mod render_priority;
pub mod render_summary;
pub mod styles;

pub use render_by_file::render_by_file;
pub use render_charts::render_charts;
pub use render_dialogs::{render_confirmation_dialog, render_reason_dialog};
pub use render_help::render_help;
pub use render_history::render_history;
pub use render_list::render_list;
pub use render_priority::render_priority;
pub use render_summary::render_summary;
