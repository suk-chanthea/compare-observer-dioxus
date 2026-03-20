//! Settings dialog — Dioxus port of `settings_dialog.cpp`.
//!
//! Manages: user/Telegram fields, dynamic system rows, and the
//! "Without" / "Except" exclusion-rule tables.

use dioxus::prelude::*;

use crate::core::{
    rules::{build_default_except, build_default_without},
    settings::{SettingsData, SystemConfigData, DEFAULT_SYSTEMS},
};
use crate::services::rules_api;

// ── Component ─────────────────────────────────────────────────────────────────

#[component]
pub fn SettingsDialog(
    username: String,
    api_url: String,
    telegram_token: String,
    telegram_chat_id: String,
    notifications_enabled: bool,
    systems: Vec<SystemConfigData>,
    without_rows: Vec<Vec<String>>,
    except_rows: Vec<Vec<String>>,
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

    // ── Helpers ───────────────────────────────────────────────────────────────

    let sys_count = move || sys_list.read().len();

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
            match rules_api::fetch_rules(&url, count).await {
                Err(msg) => {
                    api_status.set(Some(msg));
                }
                Ok(rules) => {
                    let mut updated = false;
                    if let Some(rows) = rules.without {
                        without.set(rows);
                        updated = true;
                    }
                    if let Some(rows) = rules.except {
                        except.set(rows);
                        updated = true;
                    }
                    api_status.set(Some(if updated {
                        "Rules loaded from API.".into()
                    } else {
                        "No valid rules in API response.".into()
                    }));
                }
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

                div {
                    class: "dialog-body",

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

                    GroupBox {
                        title: "Systems Configuration",
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
                                    }
                                },
                                "Remove Last System"
                            }
                        }

                        div {
                            class: "system-rows",
                            for (idx, sys) in sys_list.read().iter().enumerate() {
                                SystemRow {
                                    key: "{idx}",
                                    index: idx,
                                    data: sys.clone(),
                                    on_change: move |updated: SystemConfigData| {
                                        sys_list.write()[idx] = updated;
                                    },
                                }
                            }
                        }
                    }

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

                    if let Some(msg) = api_status.read().as_deref() {
                        p {
                            style: "color: #80BFFF; font-size: 12px; margin-top: 4px;",
                            "{msg}"
                        }
                    }
                }

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

// ── GroupBox ──────────────────────────────────────────────────────────────────

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

// ── SystemRow ─────────────────────────────────────────────────────────────────
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
                    oninput: {
                        let data = data.clone();
                        move |e: Event<FormData>| on_change.call(SystemConfigData { name: e.value(), ..data.clone() })
                    },
                }
                label { "Source:" }
                input {
                    r#type: "text",
                    value: "{data.source}",
                    oninput: {
                        let data = data.clone();
                        move |e: Event<FormData>| on_change.call(SystemConfigData { source: e.value(), ..data.clone() })
                    },
                }

                label { "Destination:" }
                input {
                    r#type: "text",
                    value: "{data.destination}",
                    oninput: {
                        let data = data.clone();
                        move |e: Event<FormData>| on_change.call(SystemConfigData { destination: e.value(), ..data.clone() })
                    },
                }
                label { "Git:" }
                input {
                    r#type: "text",
                    value: "{data.git}",
                    oninput: {
                        let data = data.clone();
                        move |e: Event<FormData>| on_change.call(SystemConfigData { git: e.value(), ..data.clone() })
                    },
                }

                label { "Backup:" }
                input {
                    r#type: "text",
                    value: "{data.backup}",
                    oninput: {
                        let data = data.clone();
                        move |e: Event<FormData>| on_change.call(SystemConfigData { backup: e.value(), ..data.clone() })
                    },
                }
                label { "Assign:" }
                input {
                    r#type: "text",
                    value: "{data.assign}",
                    oninput: {
                        let data = data.clone();
                        move |e: Event<FormData>| on_change.call(SystemConfigData { assign: e.value(), ..data.clone() })
                    },
                }
            }
        }
    }
}

// ── RuleTable ─────────────────────────────────────────────────────────────────
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
