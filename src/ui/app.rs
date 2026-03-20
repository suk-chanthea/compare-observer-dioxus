use dioxus::prelude::*;

use crate::core::{file_entry::FileEntry, settings::SettingsData};
use crate::ui::{
    dialogs::settings_dialog::SettingsDialog,
    styles,
    widgets::file_watcher_table::FileWatcherTable,
};

#[component]
pub fn App() -> Element {
    let mut show_settings = use_signal(|| false);
    let entries: Signal<Vec<FileEntry>> = use_signal(Vec::new);

    let initial_settings = SettingsData::default();

    rsx! {
        document::Style { {styles::GLOBAL_CSS} }
        div {
            class: "app-root",

            div {
                class: "toolbar",
                span { class: "toolbar-title", "Compare Observer" }
                button {
                    class: "btn",
                    onclick: move |_| show_settings.set(true),
                    "⚙ Settings"
                }
            }

            div {
                class: "main-content",
                FileWatcherTable {
                    entries,
                    on_view_diff: move |path: String| {
                        tracing::info!("View diff: {path}");
                    },
                }
            }

            if *show_settings.read() {
                SettingsDialog {
                    username: initial_settings.username.clone(),
                    api_url: initial_settings.api_url.clone(),
                    telegram_token: initial_settings.telegram_token.clone(),
                    telegram_chat_id: initial_settings.telegram_chat_id.clone(),
                    notifications_enabled: initial_settings.notifications_enabled,
                    systems: initial_settings.systems.clone(),
                    without_rows: initial_settings.without_rows.clone(),
                    except_rows: initial_settings.except_rows.clone(),
                    on_save: move |data: SettingsData| {
                        tracing::info!("Settings saved: {} systems", data.systems.len());
                        show_settings.set(false);
                    },
                    on_cancel: move |_| show_settings.set(false),
                }
            }
        }
    }
}
