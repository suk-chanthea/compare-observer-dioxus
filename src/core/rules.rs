/// Default "Without" exclusion rows (mirrors `loadWithoutDefaults`).
pub fn build_default_without(cols: usize) -> Vec<Vec<String>> {
    ["config", "config/include", "config/include/lang", "config/include/title"]
        .iter()
        .map(|&s| vec![s.to_string(); cols])
        .collect()
}

/// Default "Except" exclusion rows (mirrors `loadExceptDefaults`).
pub fn build_default_except(cols: usize) -> Vec<Vec<String>> {
    ["content", ".git", ".idea", "index.php"]
        .iter()
        .map(|&s| vec![s.to_string(); cols])
        .collect()
}

/// Converts a flat list of rule strings into a rows×cols table
/// (mirrors `SettingsDialog::rulesFromJson`).
pub fn rules_from_vec(entries: &[String], cols: usize) -> Vec<Vec<String>> {
    entries
        .iter()
        .map(|entry| vec![entry.clone(); cols.max(1)])
        .collect()
}
