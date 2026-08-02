use async_trait::async_trait;
use reqwest::Client;
use serde_json::Value;

use super::super::Error;
use super::types::PublisherImpl;

pub struct TelegramPublisher {
    bot_token: String,
    chat_id: String,
    message_thread_id: String,
}

impl TelegramPublisher {
    pub fn new(config: &Value) -> Option<Self> {
        Some(Self {
            bot_token: config.get("bot_token")?.as_str()?.to_string(),
            chat_id: config.get("chat_id")?.as_str()?.to_string(),
            message_thread_id: config
                .get("message_thread_id")
                .and_then(|v| v.as_str())
                .unwrap_or("0")
                .to_string(),
        })
    }
}

#[async_trait]
impl PublisherImpl for TelegramPublisher {
    async fn publish(&self, title: &str, description: &str, url: &str) -> Result<String, Error> {
        let message = format!(
            "<b>{}</b>\n\n{}\n\n<a href=\"{}\">{}</a>",
            title, description, url, url
        );
        let api_url = format!("https://api.telegram.org/bot{}/sendMessage", self.bot_token);
        let params = vec![
            ("chat_id", self.chat_id.as_str()),
            ("message_thread_id", self.message_thread_id.as_str()),
            ("text", &message),
            ("parse_mode", "HTML"),
        ];
        let resp = Client::new()
            .post(&api_url)
            .form(&params)
            .send()
            .await?
            .error_for_status()?
            .text()
            .await?;
        Ok(resp)
    }
}