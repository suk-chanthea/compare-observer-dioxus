//! Telegram Bot API helper — mirrors `TelegramService` from the C++ project.

use serde::Serialize;

#[derive(Serialize)]
struct SendMessage<'a> {
    chat_id: &'a str,
    text:    &'a str,
}

/// Send a plain-text message to a Telegram chat.
/// Returns `Ok(())` on success, `Err(reason)` on failure.
pub async fn send_message(token: &str, chat_id: &str, text: &str) -> Result<(), String> {
    if token.is_empty() || chat_id.is_empty() {
        return Err("Telegram token or chat ID is empty".into());
    }

    let url = format!("https://api.telegram.org/bot{token}/sendMessage");
    let body = SendMessage { chat_id, text };

    let client = reqwest::Client::new();
    let resp = client
        .post(&url)
        .json(&body)
        .send()
        .await
        .map_err(|e| format!("Network error: {e}"))?;

    if resp.status().is_success() {
        Ok(())
    } else {
        let status = resp.status();
        let detail = resp.text().await.unwrap_or_default();
        Err(format!("Telegram API error {status}: {detail}"))
    }
}

/// Build the notification text for a Copy Send operation.
/// Mirrors `formatFileListForTelegram` in the C++ project.
pub fn build_notification(
    system_name: &str,
    username: &str,
    description: &str,
    files: &[String],
) -> String {
    let file_list = files
        .iter()
        .map(|f| format!("  • {f}"))
        .collect::<Vec<_>>()
        .join("\n");

    format!(
        "🔔 File Update Notification\n\
         📋 Description: {description}\n\
         👤 User: {username}\n\
         📁 System: {system_name}\n\
         📊 Files changed: {count}\n\
         {file_list}",
        description = if description.is_empty() { system_name } else { description },
        count = files.len(),
    )
}
