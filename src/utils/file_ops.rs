use chrono::Local;

use crate::core::file_entry::FileEntry;

/// Add a new entry or refresh the status/timestamp of an existing one.
/// Mirrors `addFileEntry` / `updateFileEntry`.
pub fn upsert_entry(entries: &mut Vec<FileEntry>, path: &str, status: &str) {
    if let Some(e) = entries.iter_mut().find(|e| e.path == path) {
        e.status = status.to_string();
        e.modified = Local::now().format("%Y-%m-%d %H:%M:%S").to_string();
    } else {
        entries.push(FileEntry::new(path, status));
    }
}

/// Remove a file entry by path. Mirrors `removeFileEntry`.
pub fn remove_entry(entries: &mut Vec<FileEntry>, path: &str) {
    entries.retain(|e| e.path != path);
}

/// Return all checked file paths. Mirrors `getCheckedFileKeys`.
pub fn checked_paths(entries: &[FileEntry]) -> Vec<String> {
    entries
        .iter()
        .filter(|e| e.checked)
        .map(|e| e.path.clone())
        .collect()
}

/// Uncheck all entries whose paths are in `paths`. Mirrors `uncheckFileKeys`.
pub fn uncheck_paths(entries: &mut Vec<FileEntry>, paths: &[String]) {
    for e in entries.iter_mut() {
        if paths.contains(&e.path) {
            e.checked = false;
        }
    }
}

/// Remove all unchecked entries and return their paths.
/// Mirrors `removeUncheckedFiles`.
pub fn remove_unchecked(entries: &mut Vec<FileEntry>) -> Vec<String> {
    let mut removed = Vec::new();
    entries.retain(|e| {
        if e.checked {
            true
        } else {
            removed.push(e.path.clone());
            false
        }
    });
    removed
}
