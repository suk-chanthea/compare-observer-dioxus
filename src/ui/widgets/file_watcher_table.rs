//! FileWatcherTable — Dioxus port of `file_watcher_table.cpp`.
//!
//! Renders a table of watched files with:
//!  - per-row checkbox  (col 0)
//!  - file path         (col 1)  — click → emits `on_view_diff`
//!  - status            (col 2)
//!  - last modified     (col 3)
//!  - delete button     (col 4)
//!
//! The header checkbox mirrors `CheckBoxHeader`: Checked when all rows are
//! checked, Unchecked when none, visually indeterminate when some are checked.

use chrono::Local;
use dioxus::prelude::*;

// ── Data model ────────────────────────────────────────────────────────────────

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

// ── Header checkbox state ─────────────────────────────────────────────────────

#[derive(Clone, Copy, PartialEq)]
enum CheckState {
    Unchecked,
    Checked,
    Indeterminate,
}

impl CheckState {
    fn from_entries(entries: &[FileEntry]) -> Self {
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

// ── Public API helpers (usable by parent component) ───────────────────────────

/// Add or update a file entry. Mirrors `addFileEntry` / `updateFileEntry`.
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

/// Remove all unchecked entries, returning the removed paths.
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

// ── Component ─────────────────────────────────────────────────────────────────

#[component]
pub fn FileWatcherTable(
    entries: Signal<Vec<FileEntry>>,
    on_view_diff: EventHandler<String>,
) -> Element {
    // Derived header checkbox state
    let header_state = use_memo(move || CheckState::from_entries(&entries.read()));

    rsx! {
        div {
            class: "fw-table-wrapper",
            table {
                class: "fw-table",

                // ── Column widths ─────────────────────────────────────────────
                colgroup {
                    col { style: "width: 34px;" }
                    col { style: "width: auto;" }
                    col { style: "width: 120px;" }
                    col { style: "width: 160px;" }
                    col { style: "width: 80px;" }
                }

                // ── Header ────────────────────────────────────────────────────
                thead {
                    tr {
                        th {
                            class: "fw-col-check",
                            // Header checkbox — toggles all rows
                            HeaderCheckbox {
                                state: *header_state.read(),
                                on_click: move |_| {
                                    let next_checked = *header_state.read() != CheckState::Checked;
                                    entries.write().iter_mut().for_each(|e| e.checked = next_checked);
                                },
                            }
                        }
                        th { "File Path" }
                        th { "Status" }
                        th { "Modified" }
                        th { "Action" }
                    }
                }

                // ── Body ──────────────────────────────────────────────────────
                tbody {
                    for (idx, entry) in entries.read().iter().enumerate() {
                        FileRow {
                            key: "{entry.path}",
                            index: idx,
                            entry: entry.clone(),
                            on_check_change: move |checked: bool| {
                                entries.write()[idx].checked = checked;
                            },
                            on_view_diff: move |path: String| on_view_diff.call(path),
                            on_delete: move |_| {
                                let path = entries.read()[idx].path.clone();
                                entries.write().retain(|e| e.path != path);
                            },
                        }
                    }
                }
            }
        }
    }
}

// ── HeaderCheckbox ─────────────────────────────────────────────────────────────
/// Custom header checkbox rendered as a styled div.
/// Mirrors `CheckBoxHeader::paintSection`: blue-fill when checked,
/// dark-fill + blue-border + dash when indeterminate, dark-fill when unchecked.

#[component]
fn HeaderCheckbox(state: CheckState, on_click: EventHandler<()>) -> Element {
    let (bg, border) = match state {
        CheckState::Checked      => ("#0B57D0", "#0B57D0"),
        CheckState::Indeterminate => ("#2A2A2A", "#0B57D0"),
        CheckState::Unchecked    => ("#2A2A2A", "#555555"),
    };

    rsx! {
        div {
            style: "
                width: 16px; height: 16px;
                border-radius: 3px;
                background-color: {bg};
                border: 1px solid {border};
                cursor: pointer;
                display: flex;
                align-items: center;
                justify-content: center;
                margin: 0 auto;
                flex-shrink: 0;
            ",
            onclick: move |_| on_click.call(()),

            // Checkmark (white tick) — visible only when Checked
            if state == CheckState::Checked {
                div {
                    style: "
                        width: 9px; height: 5px;
                        border-left: 2px solid #fff;
                        border-bottom: 2px solid #fff;
                        transform: rotate(-45deg) translate(0px, -1px);
                    "
                }
            }

            // Horizontal dash — visible only when Indeterminate
            if state == CheckState::Indeterminate {
                div {
                    style: "
                        width: 8px; height: 2px;
                        background-color: #0B57D0;
                        border-radius: 1px;
                    "
                }
            }
        }
    }
}

// ── FileRow ───────────────────────────────────────────────────────────────────

#[component]
fn FileRow(
    index: usize,
    entry: FileEntry,
    on_check_change: EventHandler<bool>,
    on_view_diff: EventHandler<String>,
    on_delete: EventHandler<()>,
) -> Element {
    let path = entry.path.clone();
    let path_for_diff = path.clone();

    rsx! {
        tr {
            class: if entry.checked { "selected" } else { "" },

            // Col 0 — checkbox
            td {
                class: "fw-col-check",
                style: "text-align: center;",
                {
                    let (border_color, bg_color) = if entry.checked {
                        ("#0B57D0", "#0B57D0")
                    } else {
                        ("#555555", "#2A2A2A")
                    };
                    rsx! {
                        div {
                            style: "
                                width: 12px; height: 12px;
                                border: 1px solid {border_color};
                                border-radius: 4px;
                                background-color: {bg_color};
                                cursor: pointer;
                                margin: 0 auto;
                                display: flex;
                                align-items: center;
                                justify-content: center;
                            ",
                            onclick: move |_| on_check_change.call(!entry.checked),
                            if entry.checked {
                                div {
                                    style: "
                                        width: 6px; height: 4px;
                                        border-left: 1.5px solid #fff;
                                        border-bottom: 1.5px solid #fff;
                                        transform: rotate(-45deg) translate(0px, -1px);
                                    "
                                }
                            }
                        }
                    }
                }
            }

            // Col 1 — file path (clickable → view diff)
            td {
                class: "fw-col-path clickable",
                onclick: move |_| on_view_diff.call(path_for_diff.clone()),
                "{entry.path}"
            }

            // Col 2 — status
            td {
                class: "fw-col-status clickable",
                onclick: {
                    let p = path.clone();
                    move |_| on_view_diff.call(p.clone())
                },
                "{entry.status}"
            }

            // Col 3 — last modified
            td {
                class: "fw-col-modified clickable",
                onclick: {
                    let p = path.clone();
                    move |_| on_view_diff.call(p.clone())
                },
                "{entry.modified}"
            }

            // Col 4 — delete button
            td {
                class: "fw-col-action",
                style: "text-align: center;",
                button {
                    class: "btn-danger",
                    title: "Remove from list",
                    onclick: move |_| on_delete.call(()),
                    "🗑"
                }
            }
        }
    }
}
