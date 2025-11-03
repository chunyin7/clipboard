use chrono::{DateTime, Local};

#[derive(Clone)]
pub struct ClipboardEntry {
    pub content: String,
    pub timestamp: DateTime<Local>,
}
