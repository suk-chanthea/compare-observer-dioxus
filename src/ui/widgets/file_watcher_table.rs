//! FileWatcherTable — Dioxus port of `file_watcher_table.cpp`.
//!
//! Receives the *shared* `all_entries` signal plus `system_index` so that
//! the file-watcher coroutine in App can push events directly into the
//! right system's row list.

use dioxus::prelude::*;

use crate::core::file_entry::{CheckState, FileEntry};

// ── Component ─────────────────────────────────────────────────────────────────

#[component]
pub fn FileWatcherTable(
    all_entries: Signal<Vec<Vec<FileEntry>>>,
    system_index: usize,
    on_view_diff: EventHandler<String>,
) -> Element {
    // Snapshot — avoids holding a read-guard across the RSX tree.
    let entries_snap: Vec<FileEntry> = all_entries
        .read()
        .get(system_index)
        .cloned()
        .unwrap_or_default();

    let header_state = CheckState::from_entries(&entries_snap);

    rsx! {
        div {
            class: "fw-table-wrapper",
            table {
                class: "fw-table",

                colgroup {
                    col { style: "width: 34px;" }
                    col { style: "width: auto;" }
                    col { style: "width: 120px;" }
                    col { style: "width: 160px;" }
                    col { style: "width: 80px;" }
                }

                thead {
                    tr {
                        th {
                            class: "fw-col-check",
                            HeaderCheckbox {
                                state: header_state,
                                on_click: move |_| {
                                    let snap = all_entries.read()
                                        .get(system_index)
                                        .cloned()
                                        .unwrap_or_default();
                                    let next = CheckState::from_entries(&snap) != CheckState::Checked;
                                    if let Some(sys) = all_entries.write().get_mut(system_index) {
                                        sys.iter_mut().for_each(|e| e.checked = next);
                                    }
                                },
                            }
                        }
                        th { "File Path" }
                        th { "Status" }
                        th { "Modified" }
                        th { "Action" }
                    }
                }

                tbody {
                    for (idx, entry) in entries_snap.iter().enumerate() {
                        FileRow {
                            key: "{entry.path}",
                            index: idx,
                            entry: entry.clone(),
                            on_check_change: move |checked: bool| {
                                if let Some(sys) = all_entries.write().get_mut(system_index) {
                                    if let Some(e) = sys.get_mut(idx) {
                                        e.checked = checked;
                                    }
                                }
                            },
                            on_view_diff: move |path: String| on_view_diff.call(path),
                            on_delete: move |_| {
                                let path = all_entries.read()
                                    .get(system_index)
                                    .and_then(|s| s.get(idx))
                                    .map(|e| e.path.clone())
                                    .unwrap_or_default();
                                if let Some(sys) = all_entries.write().get_mut(system_index) {
                                    sys.retain(|e| e.path != path);
                                }
                            },
                        }
                    }
                }
            }
        }
    }
}

// ── HeaderCheckbox ─────────────────────────────────────────────────────────────

#[component]
fn HeaderCheckbox(state: CheckState, on_click: EventHandler<()>) -> Element {
    let (bg, border) = match state {
        CheckState::Checked       => ("#0B57D0", "#0B57D0"),
        CheckState::Indeterminate => ("#2A2A2A", "#0B57D0"),
        CheckState::Unchecked     => ("#2A2A2A", "#555555"),
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
    let path      = entry.path.clone();
    let path_diff = path.clone();
    let path_st   = path.clone();
    let path_mod  = path.clone();

    rsx! {
        tr {
            class: if entry.checked { "selected" } else { "" },

            // ── Checkbox ─────────────────────────────────────────────────────
            td {
                class: "fw-col-check",
                onclick: move |_| on_check_change.call(!entry.checked),
                div {
                    class: if entry.checked { "fw-checkbox fw-checkbox-on" } else { "fw-checkbox" },
                    if entry.checked {
                        div { class: "fw-check-mark" }
                    }
                }
            }

            // ── File path ────────────────────────────────────────────────────
            td {
                class: "fw-col-path clickable",
                onclick: move |_| on_view_diff.call(path_diff.clone()),
                "{entry.path}"
            }

            // ── Status ───────────────────────────────────────────────────────
            td {
                class: "fw-col-status clickable",
                onclick: move |_| on_view_diff.call(path_st.clone()),
                "{entry.status}"
            }

            // ── Modified ─────────────────────────────────────────────────────
            td {
                class: "fw-col-modified clickable",
                onclick: move |_| on_view_diff.call(path_mod.clone()),
                "{entry.modified}"
            }

            // ── Delete ───────────────────────────────────────────────────────
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
