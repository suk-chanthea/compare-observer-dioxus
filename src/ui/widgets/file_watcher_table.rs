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

use dioxus::prelude::*;

use crate::core::file_entry::{CheckState, FileEntry};

// ── Component ─────────────────────────────────────────────────────────────────

#[component]
pub fn FileWatcherTable(
    entries: Signal<Vec<FileEntry>>,
    on_view_diff: EventHandler<String>,
) -> Element {
    let header_state = use_memo(move || CheckState::from_entries(&entries.read()));

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
/// Mirrors `CheckBoxHeader::paintSection`.

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
    let path = entry.path.clone();
    let path_for_diff = path.clone();

    rsx! {
        tr {
            class: if entry.checked { "selected" } else { "" },

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

            td {
                class: "fw-col-path clickable",
                onclick: move |_| on_view_diff.call(path_for_diff.clone()),
                "{entry.path}"
            }

            td {
                class: "fw-col-status clickable",
                onclick: {
                    let p = path.clone();
                    move |_| on_view_diff.call(p.clone())
                },
                "{entry.status}"
            }

            td {
                class: "fw-col-modified clickable",
                onclick: {
                    let p = path.clone();
                    move |_| on_view_diff.call(p.clone())
                },
                "{entry.modified}"
            }

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
