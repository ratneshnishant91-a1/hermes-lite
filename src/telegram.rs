//! Telegram bot gateway.

use anyhow::Result;
use serde_json::Value;
use std::sync::{Arc, Mutex};
use tokio::sync::mpsc;

pub struct TelegramBot {
    token: String,
    allowed_users: Vec<i64>,
}

impl TelegramBot {
    pub fn new(token: &str, allowed_users: Vec<i64>) -> Self {
        Self {
            token: token.to_string(),
            allowed_users,
        }
    }

    /// Start bot polling
    pub async fn start(&self, agent_tx: mpsc::UnboundedSender<String>) -> Result<()> {
        let base_url = format!("https://api.telegram.org/bot{}", self.token);

        let mut offset = 0;
        loop {
            // Get updates
            let url = format!("{}/getUpdates?offset={}&timeout=30", base_url, offset);
            let resp = ureq::get(&url).call()?;
            let json: Value = resp.into_json()?;

            if let Some(results) = json.get("result").and_then(|v| v.as_array()) {
                for update in results {
                    offset = update.get("update_id").and_then(|v| v.as_i64()).unwrap_or(0) + 1;

                    if let Some(message) = update.get("message") {
                        let chat_id = message.get("chat").and_then(|v| v.get("id")).and_then(|v| v.as_i64());
                        let from_id = message.get("from").and_then(|v| v.get("id")).and_then(|v| v.as_i64());
                        let text = message.get("text").and_then(|v| v.as_str());

                        if let (Some(chat_id), Some(from_id), Some(text)) = (chat_id, from_id, text) {
                            // Check if user is allowed
                            if self.allowed_users.is_empty() || self.allowed_users.contains(&from_id) {
                                // Send to agent
                                let _ = agent_tx.send(format!("telegram:{}:{}", chat_id, text));
                            }
                        }
                    }
                }
            }

            tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
        }
    }

    /// Send message to Telegram
    pub fn send_message(&self, chat_id: i64, text: &str) -> Result<()> {
        let base_url = format!("https://api.telegram.org/bot{}", self.token);
        let url = format!("{}/sendMessage", base_url);

        let body = serde_json::json!({
            "chat_id": chat_id,
            "text": text
        });

        ureq::post(&url)
            .set("Content-Type", "application/json")
            .send_json(&body)?;

        Ok(())
    }
}
