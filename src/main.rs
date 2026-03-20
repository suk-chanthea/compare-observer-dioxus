mod ui;

use dioxus::prelude::*;
use ui::{
    dialogs::settings_dialog::{SettingsData, SettingsDialog},
    styles,
    widgets::file_watcher_table::{FileEntry, FileWatcherTable},
};

fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "compare_observer=info".into()),
        )
        .init();

    dioxus::launch(App);
}

#[component]
fn App() -> Element {
    let mut show_settings = use_signal(|| false);
    let entries: Signal<Vec<FileEntry>> = use_signal(Vec::new);

    let initial_settings = SettingsData::default();

    rsx! {
        document::Style { {styles::GLOBAL_CSS} }
        div {
            class: "app-root",

            // Top toolbar
            div {
                class: "toolbar",
                span { class: "toolbar-title", "Compare Observer" }
                button {
                    class: "btn",
                    onclick: move |_| show_settings.set(true),
                    "⚙ Settings"
                }
            }

            // Main content
            div {
                class: "main-content",
                FileWatcherTable {
                    entries,
                    on_view_diff: move |path: String| {
                        // handled by parent in full app
                        tracing::info!("View diff: {path}");
                    },
                }
            }

            // Settings dialog overlay
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
