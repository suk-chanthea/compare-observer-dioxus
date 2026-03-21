use serde::{Deserialize, Serialize};
use std::path::PathBuf;

pub const DEFAULT_SYSTEMS: usize = 3;

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
    pub fn with_default_name(index: usize) -> Self {
        Self {
            name: format!("System {}", index + 1),
            ..Default::default()
        }
    }
}

/// Everything the parent receives when the user clicks Save.
#[derive(Clone, Default, Serialize, Deserialize)]
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
    /// Which system tabs are currently shown (mirrors `selectedSystems` in C++).
    pub selected_systems: Vec<bool>,
}

/// Returns the path to the settings JSON file.
/// Mirrors QSettings("CompareObserver", "FileWatcher") → %APPDATA%\CompareObserver\FileWatcher\settings.json
fn settings_path() -> PathBuf {
    let base = std::env::var("APPDATA")
        .map(PathBuf::from)
        .unwrap_or_else(|_| {
            dirs_next::config_dir()
                .or_else(dirs_next::home_dir)
                .unwrap_or_else(|| PathBuf::from("."))
        });
    base.join("CompareObserver").join("FileWatcher").join("settings.json")
}

/// Load settings from disk. Returns `SettingsData::default()` if no file exists yet.
pub fn load_settings() -> SettingsData {
    let path = settings_path();
    match std::fs::read_to_string(&path) {
        Ok(json) => serde_json::from_str(&json).unwrap_or_default(),
        Err(_) => SettingsData::default(),
    }
}

/// Persist settings to disk, creating directories as needed.
pub fn save_settings(data: &SettingsData) {
    let path = settings_path();
    if let Some(parent) = path.parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    if let Ok(json) = serde_json::to_string_pretty(data) {
        let _ = std::fs::write(&path, json);
    }
}
