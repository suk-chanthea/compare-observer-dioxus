use serde::{Deserialize, Serialize};

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
