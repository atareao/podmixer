use reqwest::Client;
use serde_json::{json, Value};
use serde::{Serialize, Deserialize};
use tracing::debug;
use super::Error;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Telegram{
    active: bool,
    token: String,
    chat_id: i64,
    #[serde(default = "default_thread_id")]
    thread_id: i64,
}

fn default_thread_id() -> i64{
    0
}

const URL: &str = "https://api.telegram.org";

impl Telegram{
    pub fn new(active: bool, token: String, chat_id: i64, thread_id: i64) -> Self{
        Self{
            active,
            token,
            chat_id,
            thread_id,
        }
    }

    pub fn is_active(&self) -> bool{
        self.active
    }

    #[allow(dead_code)]
    pub async fn send_message(&self, message: &str) -> Result<String, Error>{
        let url = format!("{URL}/bot{}/sendMessage", self.token);
        let message = json!({
            "chat_id": self.chat_id,
            "message_thread_id": self.thread_id,
            "text": message,
            "parse_mode": "HTML",
        });
        self._post(&url, &message).await
    }

    pub async fn send_audio(&self, audio: &str, caption: &str) -> Result<String, Error>{
        debug!("send_audio");
        let url = format!("{URL}/bot{}/sendAudio", self.token);
        debug!("url: {url}");
        let message = json!({
            "chat_id": self.chat_id,
            "message_thread_id": self.thread_id,
            "audio": audio,
            "caption": caption,
            "parse_mode": "HTML",
        });
        debug!("message: {:?}", message);
        self._post(&url, &message).await
    }
    async fn _post(&self, url: &str, body: &Value) -> Result<String, Error>{
        debug!("_post");
        Client::new()
            .post(url)
            .json(&body)
            .send()
            .await?
            .error_for_status()?
            .text()
            .await.map_err(|e| e.into())
    }
}
