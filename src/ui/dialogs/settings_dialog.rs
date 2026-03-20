//! Settings dialog — Dioxus port of `settings_dialog.cpp`.
//!
//! Manages: user/Telegram fields, dynamic system rows, and the
//! "Without" / "Except" exclusion-rule tables.

use dioxus::prelude::*;
use serde::{Deserialize, Serialize};

const DEFAULT_SYSTEMS: usize = 3;

// ── Data types ────────────────────────────────────────────────────────────────

/// Per-system configuration row (mirrors `SettingsDialog::SystemConfigData`).
#[derive(Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct SystemConfigData {
    pub name: String,
    pub source: String,
    pub destination: String,
    pub git: String,
    pub backup: String,
    pub assign: String,
}

impl SystemConfigData {
    fn with_default_name(index: usize) -> Self {
        Self {
            name: format!("System {}", index + 1),
            ..Default::default()
        }
    }
}

/// Everything the parent receives when the user clicks Save.
#[derive(Clone, Default)]
pub struct SettingsData {
    pub username: String,
    pub api_url: String,
    pub telegram_token: String,
    pub telegram_chat_id: String,
    pub notifications_enabled: bool,
    pub systems: Vec<SystemConfigData>,
    /// Rows × columns (one column per system).
    pub without_rows: Vec<Vec<String>>,
    pub except_rows: Vec<Vec<String>>,
}

// ── Remote-API response shape ─────────────────────────────────────────────────

#[derive(Deserialize)]
struct RulesApiResponse {
    without: Option<Vec<String>>,
    except: Option<Vec<String>>,
}

// ── Component ─────────────────────────────────────────────────────────────────

#[component]
pub fn SettingsDialog(
    // Initial values passed from the parent
    username: String,
    api_url: String,
    telegram_token: String,
    telegram_chat_id: String,
    notifications_enabled: bool,
    systems: Vec<SystemConfigData>,
    without_rows: Vec<Vec<String>>,
    except_rows: Vec<Vec<String>>,
    // Callbacks
    on_save: EventHandler<SettingsData>,
    on_cancel: EventHandler<()>,
) -> Element {
    // ── Form-field signals ───────────────────────────────────────────────────
    let mut username_input = use_signal(|| username.clone());
    let mut api_url_input = use_signal(|| api_url.clone());
    let mut token_input = use_signal(|| telegram_token.clone());
    let mut chat_id_input = use_signal(|| telegram_chat_id.clone());
    let mut notifications = use_signal(|| notifications_enabled);

    // ── Systems list ─────────────────────────────────────────────────────────
    let initial_systems = if systems.is_empty() {
        (0..DEFAULT_SYSTEMS)
            .map(SystemConfigData::with_default_name)
            .collect::<Vec<_>>()
    } else {
        systems.clone()
    };
    let mut sys_list: Signal<Vec<SystemConfigData>> = use_signal(|| initial_systems);

    // ── Table data: Vec<rows> where each row is Vec<col per system> ──────────
    let initial_without = if without_rows.is_empty() {
        build_default_without(systems.len().max(DEFAULT_SYSTEMS))
    } else {
        without_rows.clone()
    };
    let initial_except = if except_rows.is_empty() {
        build_default_except(systems.len().max(DEFAULT_SYSTEMS))
    } else {
        except_rows.clone()
    };
    let mut without: Signal<Vec<Vec<String>>> = use_signal(|| initial_without);
    let mut except: Signal<Vec<Vec<String>>> = use_signal(|| initial_except);

    // ── "Add System" mini-dialog state ───────────────────────────────────────
    let mut show_add_dialog = use_signal(|| false);
    let mut new_sys_name = use_signal(String::new);

    // ── API error/status message ──────────────────────────────────────────────
    let mut api_status: Signal<Option<String>> = use_signal(|| None);

    // ── Helpers (closures) ────────────────────────────────────────────────────

    // Return system count at call time
    let sys_count = move || sys_list.read().len();

    // Build a settings snapshot for the save callback
    let build_settings = move || SettingsData {
        username: username_input.read().clone(),
        api_url: api_url_input.read().clone(),
        telegram_token: token_input.read().clone(),
        telegram_chat_id: chat_id_input.read().clone(),
        notifications_enabled: *notifications.read(),
        systems: sys_list.read().clone(),
        without_rows: without.read().clone(),
        except_rows: except.read().clone(),
    };

    // ── Remote defaults loader (mirrors loadRemoteRuleDefaults) ───────────────
    let load_remote = move |_| {
        let url = api_url_input.read().clone();
        let count = sys_count();

        spawn(async move {
            match reqwest::get(&url).await {
                Err(e) => {
                    api_status.set(Some(format!("Network error: {e}")));
                }
                Ok(resp) => match resp.json::<RulesApiResponse>().await {
                    Err(e) => {
                        api_status.set(Some(format!("JSON parse error: {e}")));
                    }
                    Ok(data) => {
                        let mut updated = false;
                        if let Some(arr) = data.without {
                            if !arr.is_empty() {
                                without.set(rules_from_vec(&arr, count));
                                updated = true;
                            }
                        }
                        if let Some(arr) = data.except {
                            if !arr.is_empty() {
                                except.set(rules_from_vec(&arr, count));
                                updated = true;
                            }
                        }
                        if updated {
                            api_status.set(Some("Rules loaded from API.".into()));
                        } else {
                            api_status.set(Some("No valid rules in API response.".into()));
                        }
                    }
                },
            }
        });
    };

    // ── Render ────────────────────────────────────────────────────────────────
    rsx! {
        div {
            class: "dialog-overlay",

            div {
                class: "dialog",
                style: "min-width: 1000px; max-height: 90vh;",

                // ── Scrollable body ──────────────────────────────────────────
                div {
                    class: "dialog-body",

                    // ── User & Telegram group ────────────────────────────────
                    GroupBox {
                        title: "User & Telegram",
                        div {
                            class: "grid-4col",
                            label { "Username:" }
                            input {
                                r#type: "text",
                                value: "{username_input}",
                                oninput: move |e| username_input.set(e.value()),
                                style: "grid-column: span 3",
                            }
                            label { "API URL:" }
                            input {
                                r#type: "text",
                                value: "{api_url_input}",
                                placeholder: "http://khmergaming.436bet.com/app/log_sys.php",
                                oninput: move |e| api_url_input.set(e.value()),
                                style: "grid-column: span 3",
                            }
                            label { "Telegram Token:" }
                            input {
                                r#type: "password",
                                value: "{token_input}",
                                oninput: move |e| token_input.set(e.value()),
                            }
                            label { "Telegram Group ID:" }
                            input {
                                r#type: "text",
                                value: "{chat_id_input}",
                                oninput: move |e| chat_id_input.set(e.value()),
                            }
                            // Notifications checkbox spans full width
                            div {
                                style: "grid-column: span 4; margin-top: 4px;",
                                label {
                                    class: "checkbox-label",
                                    input {
                                        r#type: "checkbox",
                                        checked: *notifications.read(),
                                        onchange: move |e| notifications.set(e.checked()),
                                    }
                                    "Enable Telegram Notifications"
                                }
                            }
                        }
                    }

                    // ── Systems Configuration group ──────────────────────────
                    GroupBox {
                        title: "Systems Configuration",
                        // Add / Remove buttons
                        div {
                            class: "system-row-buttons",
                            button {
                                class: "btn",
                                onclick: move |_| {
                                    new_sys_name.set(
                                        format!("System {}", sys_list.read().len() + 1)
                                    );
                                    show_add_dialog.set(true);
                                },
                                "Add System"
                            }
                            button {
                                class: "btn",
                                onclick: move |_| {
                                    if sys_list.read().len() > 1 {
                                        sys_list.write().pop();
                                        without.write().iter_mut().for_each(|r| { r.pop(); });
                                        except.write().iter_mut().for_each(|r| { r.pop(); });
                                        // Keep at least 1 column in every row
                                    }
                                },
                                "Remove Last System"
                            }
                        }

                        // System rows
                        div {
                            class: "system-rows",
                            for (idx, sys) in sys_list.read().iter().enumerate() {
                                SystemRow {
                                    key: "{idx}",
                                    index: idx,
                                    data: sys.clone(),
                                    on_change: move |updated: SystemConfigData| {
                                        // Sync group-box title live as name changes
                                        sys_list.write()[idx] = updated;
                                        // Rebuild table column headers by re-rendering (reactive)
                                    },
                                }
                            }
                        }
                    }

                    // ── Without group ────────────────────────────────────────
                    GroupBox {
                        title: "Without",
                        RuleTable {
                            rows: without,
                            headers: sys_list.read().iter()
                                .enumerate()
                                .map(|(i, s)| {
                                    let n = s.name.trim();
                                    if n.is_empty() { format!("Sys{}", i + 1) } else { n.to_string() }
                                })
                                .collect(),
                        }
                        div {
                            class: "table-buttons",
                            button {
                                class: "btn",
                                onclick: load_remote.clone(),
                                "Default"
                            }
                            button {
                                class: "btn",
                                onclick: move |_| {
                                    let cols = sys_count();
                                    without.write().push(vec![String::new(); cols]);
                                },
                                "Add"
                            }
                            button {
                                class: "btn",
                                onclick: move |_| { without.write().pop(); },
                                "Delete"
                            }
                        }
                    }

                    // ── Except group ─────────────────────────────────────────
                    GroupBox {
                        title: "Except",
                        RuleTable {
                            rows: except,
                            headers: sys_list.read().iter()
                                .enumerate()
                                .map(|(i, s)| {
                                    let n = s.name.trim();
                                    if n.is_empty() { format!("Sys{}", i + 1) } else { n.to_string() }
                                })
                                .collect(),
                        }
                        div {
                            class: "table-buttons",
                            button {
                                class: "btn",
                                onclick: load_remote.clone(),
                                "Default"
                            }
                            button {
                                class: "btn",
                                onclick: move |_| {
                                    let cols = sys_count();
                                    except.write().push(vec![String::new(); cols]);
                                },
                                "Add"
                            }
                            button {
                                class: "btn",
                                onclick: move |_| { except.write().pop(); },
                                "Delete"
                            }
                        }
                    }

                    // API status message
                    if let Some(msg) = api_status.read().as_deref() {
                        p {
                            style: "color: #80BFFF; font-size: 12px; margin-top: 4px;",
                            "{msg}"
                        }
                    }
                }

                // ── Footer buttons ───────────────────────────────────────────
                div {
                    class: "dialog-footer",
                    button {
                        class: "btn",
                        onclick: move |_| on_save.call(build_settings()),
                        "Save"
                    }
                    button {
                        class: "btn",
                        onclick: move |_| on_cancel.call(()),
                        "Cancel"
                    }
                }
            }

            // ── "Add System" mini-dialog ─────────────────────────────────────
            if *show_add_dialog.read() {
                div {
                    class: "mini-dialog-overlay",
                    div {
                        class: "mini-dialog",
                        h3 { "Add System" }
                        label { "System name:" }
                        input {
                            r#type: "text",
                            value: "{new_sys_name}",
                            oninput: move |e| new_sys_name.set(e.value()),
                            autofocus: true,
                        }
                        div {
                            class: "mini-dialog-buttons",
                            button {
                                class: "btn",
                                onclick: move |_| {
                                    let name = new_sys_name.read().trim().to_string();
                                    if !name.is_empty() {
                                        sys_list.write().push(SystemConfigData {
                                            name: name.clone(),
                                            ..Default::default()
                                        });
                                        // Append an empty column to every existing row
                                        without.write().iter_mut().for_each(|r| r.push(String::new()));
                                        except.write().iter_mut().for_each(|r| r.push(String::new()));
                                    }
                                    show_add_dialog.set(false);
                                },
                                "OK"
                            }
                            button {
                                class: "btn",
                                onclick: move |_| show_add_dialog.set(false),
                                "Cancel"
                            }
                        }
                    }
                }
            }
        }
    }
}

// ── GroupBox helper component ─────────────────────────────────────────────────

#[component]
fn GroupBox(title: String, children: Element) -> Element {
    rsx! {
        div {
            class: "group-box",
            span { class: "group-box-title", "{title}" }
            {children}
        }
    }
}

// ── SystemRow component ───────────────────────────────────────────────────────
/// Renders one system's configuration fields (mirrors `createSystemRowWidget`).

#[component]
fn SystemRow(
    index: usize,
    data: SystemConfigData,
    on_change: EventHandler<SystemConfigData>,
) -> Element {
    let title = if data.name.trim().is_empty() {
        format!("System {}", index + 1)
    } else {
        data.name.clone()
    };

    rsx! {
        div {
            class: "group-box",
            span { class: "group-box-title", "{title}" }

            div {
                class: "grid-4col",

                label { "Name:" }
                input {
                    r#type: "text",
                    value: "{data.name}",
                    oninput: move |e| on_change.call(SystemConfigData { name: e.value(), ..data.clone() }),
                }
                label { "Source:" }
                input {
                    r#type: "text",
                    value: "{data.source}",
                    oninput: move |e| on_change.call(SystemConfigData { source: e.value(), ..data.clone() }),
                }

                label { "Destination:" }
                input {
                    r#type: "text",
                    value: "{data.destination}",
                    oninput: move |e| on_change.call(SystemConfigData { destination: e.value(), ..data.clone() }),
                }
                label { "Git:" }
                input {
                    r#type: "text",
                    value: "{data.git}",
                    oninput: move |e| on_change.call(SystemConfigData { git: e.value(), ..data.clone() }),
                }

                label { "Backup:" }
                input {
                    r#type: "text",
                    value: "{data.backup}",
                    oninput: move |e| on_change.call(SystemConfigData { backup: e.value(), ..data.clone() }),
                }
                label { "Assign:" }
                input {
                    r#type: "text",
                    value: "{data.assign}",
                    oninput: move |e| on_change.call(SystemConfigData { assign: e.value(), ..data.clone() }),
                }
            }
        }
    }
}

// ── RuleTable component ───────────────────────────────────────────────────────
/// Editable table whose columns map to systems (mirrors the Without / Except tables).

#[component]
fn RuleTable(rows: Signal<Vec<Vec<String>>>, headers: Vec<String>) -> Element {
    rsx! {
        div {
            class: "settings-table-wrapper",
            table {
                class: "settings-table",
                thead {
                    tr {
                        for h in &headers {
                            th { "{h}" }
                        }
                    }
                }
                tbody {
                    for (row_idx, row_data) in rows.read().iter().enumerate() {
                        tr {
                            key: "{row_idx}",
                            for (col_idx, cell_val) in row_data.iter().enumerate() {
                                td {
                                    key: "{col_idx}",
                                    input {
                                        r#type: "text",
                                        value: "{cell_val}",
                                        oninput: move |e| {
                                            rows.write()[row_idx][col_idx] = e.value();
                                        },
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

// ── Default data helpers (mirrors loadWithoutDefaults / loadExceptDefaults) ───

fn build_default_without(cols: usize) -> Vec<Vec<String>> {
    ["config", "config/include", "config/include/lang", "config/include/title"]
        .iter()
        .map(|&s| vec![s.to_string(); cols])
        .collect()
}

fn build_default_except(cols: usize) -> Vec<Vec<String>> {
    ["content", ".git", ".idea", "index.php"]
        .iter()
        .map(|&s| vec![s.to_string(); cols])
        .collect()
}

/// Converts a flat list of rule strings into a rows×cols table
/// (mirrors `SettingsDialog::rulesFromJson`).
fn rules_from_vec(entries: &[String], cols: usize) -> Vec<Vec<String>> {
    entries
        .iter()
        .map(|entry| vec![entry.clone(); cols.max(1)])
        .collect()
}
