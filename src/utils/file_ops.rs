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

/// Like `upsert_entry` but also sets the baseline `content` on NEW entries only.
/// On subsequent updates the original baseline is preserved so diffs always
/// compare against the content captured at watch-start — exactly like C++.
pub fn upsert_watch_event(
    entries: &mut Vec<FileEntry>,
    path: &str,
    status: &str,
    baseline_content: Option<String>,
) {
    if let Some(e) = entries.iter_mut().find(|e| e.path == path) {
        // Entry already in table: never downgrade "Created" to "Modified".
        // A file that was newly created and then immediately written to should
        // still show as "Created" so the user knows it didn't exist before.
        if e.status != "Created" {
            e.status = status.to_string();
        }
        e.modified = Local::now().format("%Y-%m-%d %H:%M:%S").to_string();
    } else {
        // New entry: attach baseline so the diff dialog can show old vs new
        let mut e = FileEntry::new(path, status);
        e.content = baseline_content;
        entries.push(e);
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
