use dioxus::prelude::*;

use crate::core::{
    file_entry::FileEntry,
    settings::{SettingsData, DEFAULT_SYSTEMS},
};
use crate::ui::{
    dialogs::settings_dialog::SettingsDialog,
    styles,
    widgets::file_watcher_table::FileWatcherTable,
};

// ── Per-system metadata (entries live inside SystemPanel) ─────────────────────

#[derive(Clone)]
struct SystemMeta {
    name: String,
    description: String,
    status: String,
    selected: bool,
}

impl SystemMeta {
    fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            description: String::new(),
            status: "Idle".to_string(),
            selected: true,
        }
    }
}

// ── App ───────────────────────────────────────────────────────────────────────

#[component]
pub fn App() -> Element {
    let mut show_settings = use_signal(|| false);
    let mut settings = use_signal(SettingsData::default);
    let mut is_watching = use_signal(|| false);

    let mut systems: Signal<Vec<SystemMeta>> = use_signal(|| {
        (0..DEFAULT_SYSTEMS)
            .map(|i| SystemMeta::new(format!("System {}", i + 1)))
            .collect()
    });

    // Snapshot for iteration — avoids holding the read lock inside closures
    let sys_info: Vec<(usize, String, bool)> = systems
        .read()
        .iter()
        .enumerate()
        .map(|(i, s)| (i, s.name.clone(), s.selected))
        .collect();

    let statuses: Vec<String> = systems
        .read()
        .iter()
        .map(|s| format!("• {}: {}", s.name, s.status))
        .collect();

    let panels: Vec<(usize, String, String)> = systems
        .read()
        .iter()
        .enumerate()
        .filter(|(_, s)| s.selected)
        .map(|(i, s)| (i, s.name.clone(), s.description.clone()))
        .collect();

    rsx! {
        document::Style { {styles::GLOBAL_CSS} }
        div {
            class: "app-root",

            // ── Menu bar ─────────────────────────────────────────────────────
            div {
                class: "menu-bar",
                button { class: "menu-item", "File Monitoring" }
                button { class: "menu-item", "Help" }
            }

            // ── Toolbar ──────────────────────────────────────────────────────
            div {
                class: "toolbar",

                // Left: system selector tabs
                div {
                    class: "toolbar-left",
                    span { class: "label-text", "Select Systems:" }
                    for (i, nm, sel) in sys_info {
                        button {
                            key: "{i}",
                            class: if sel { "sys-btn sys-btn-on" } else { "sys-btn" },
                            onclick: move |_| {
                                systems.write()[i].selected ^= true;
                            },
                            if sel { "✓ " } else { "" }
                            "{nm}"
                        }
                    }
                }

                // Right: status dots + action buttons
                div {
                    class: "toolbar-right",
                    for s in &statuses {
                        span { class: "status-dot-label", "{s}" }
                    }
                    button {
                        class: if *is_watching.read() { "btn btn-stop" } else { "btn btn-start" },
                        onclick: move |_| {
                            let v = !*is_watching.read();
                            is_watching.set(v);
                        },
                        if *is_watching.read() { "Stop Watching" } else { "Start Watching" }
                    }
                    button { class: "btn", "View Logs" }
                    button {
                        class: "btn",
                        onclick: move |_| show_settings.set(true),
                        "⚙"
                    }
                }
            }

            // ── Main content — one panel per selected system ──────────────────
            div {
                class: "main-content",
                for (i, nm, desc) in panels {
                    SystemPanel {
                        key: "{i}",
                        index: i,
                        name: nm,
                        description: desc,
                        on_description_change: move |val: String| {
                            systems.write()[i].description = val;
                        },
                    }
                }
            }

            // ── Settings dialog ───────────────────────────────────────────────
            if *show_settings.read() {
                SettingsDialog {
                    username: settings.read().username.clone(),
                    api_url: settings.read().api_url.clone(),
                    telegram_token: settings.read().telegram_token.clone(),
                    telegram_chat_id: settings.read().telegram_chat_id.clone(),
                    notifications_enabled: settings.read().notifications_enabled,
                    systems: settings.read().systems.clone(),
                    without_rows: settings.read().without_rows.clone(),
                    except_rows: settings.read().except_rows.clone(),
                    on_save: move |data: SettingsData| {
                        // Sync system names/count from saved settings
                        let new_count = data.systems.len();
                        let mut metas = systems.write();
                        metas.resize_with(new_count, || SystemMeta::new(""));
                        for (meta, cfg) in metas.iter_mut().zip(data.systems.iter()) {
                            if !cfg.name.trim().is_empty() {
                                meta.name = cfg.name.clone();
                            }
                            if meta.status.is_empty() {
                                meta.status = "Idle".to_string();
                                meta.selected = true;
                            }
                        }
                        drop(metas);
                        settings.set(data);
                        show_settings.set(false);
                    },
                    on_cancel: move |_| show_settings.set(false),
                }
            }
        }
    }
}

// ── SystemPanel ───────────────────────────────────────────────────────────────

#[component]
fn SystemPanel(
    index: usize,
    name: String,
    description: String,
    on_description_change: EventHandler<String>,
) -> Element {
    let entries: Signal<Vec<FileEntry>> = use_signal(Vec::new);

    rsx! {
        div {
            class: "system-panel",

            // Description row
            div {
                class: "system-desc-row",
                label { class: "system-desc-label", "Description for {name}:" }
                input {
                    r#type: "text",
                    class: "system-desc-input",
                    value: "{description}",
                    placeholder: "Enter description here...",
                    oninput: move |e| on_description_change.call(e.value()),
                }
            }

            // Table + action buttons
            div {
                class: "system-body",
                div {
                    class: "system-table-area",
                    FileWatcherTable {
                        entries,
                        on_view_diff: move |path: String| {
                            tracing::info!("View diff [system {}]: {path}", index);
                        },
                    }
                }
                div {
                    class: "system-actions",
                    button {
                        class: "btn-action btn-copy",
                        onclick: move |_| {
                            let paths = crate::utils::file_ops::checked_paths(&entries.read());
                            tracing::info!("Copy [system {}]: {:?}", index, paths);
                        },
                        "Copy"
                    }
                    button {
                        class: "btn-action btn-copy-send",
                        onclick: move |_| {
                            let paths = crate::utils::file_ops::checked_paths(&entries.read());
                            tracing::info!("Copy Send [system {}]: {:?}", index, paths);
                        },
                        "Copy Send"
                    }
                    button {
                        class: "btn-action btn-assign",
                        onclick: move |_| {
                            let paths = crate::utils::file_ops::checked_paths(&entries.read());
                            tracing::info!("Assign To [system {}]: {:?}", index, paths);
                        },
                        "Assign To"
                    }
                }
            }
        }
    }
}
