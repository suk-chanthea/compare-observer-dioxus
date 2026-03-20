use chrono::Local;

/// A single entry in the watched-files table.
#[derive(Clone, PartialEq)]
pub struct FileEntry {
    pub path: String,
    pub status: String,
    pub modified: String,
    pub checked: bool,
    /// Baseline content for diff viewing (`None` = no baseline captured yet).
    pub content: Option<String>,
}

impl FileEntry {
    pub fn new(path: impl Into<String>, status: impl Into<String>) -> Self {
        Self {
            path: path.into(),
            status: status.into(),
            modified: Local::now().format("%Y-%m-%d %H:%M:%S").to_string(),
            checked: true,
            content: None,
        }
    }
}

/// Tri-state for the header checkbox in the file-watcher table.
#[derive(Clone, Copy, PartialEq)]
pub enum CheckState {
    Unchecked,
    Checked,
    Indeterminate,
}

impl CheckState {
    pub fn from_entries(entries: &[FileEntry]) -> Self {
        if entries.is_empty() {
            return Self::Unchecked;
        }
        let checked_count = entries.iter().filter(|e| e.checked).count();
        match checked_count {
            0 => Self::Unchecked,
            n if n == entries.len() => Self::Checked,
            _ => Self::Indeterminate,
        }
    }
}
