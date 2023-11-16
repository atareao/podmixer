use reqwest::Client;
use serde_json::json;
use serde::{Serialize, Deserialize};
use super::CustomError;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Telegram{
    token: String,
    chat_id: i64,
    #[serde(default = "default_thread_id")]
    thread_id: i64,
}

fn default_thread_id() -> i64{
    0
}

const URL: &'static str = "https://api.telegram.org";

impl Telegram{
    pub fn new(token: String, chat_id: i64, thread_id: i64) -> Self{
        Self{
            token,
            chat_id,
            thread_id,
        }
    }

    pub async fn send_message(&self, message: &str) -> Result<String, CustomError>{
        let url = format!("{URL}/bot{}/sendMessage", self.token);
        let message = json!({
            "chat_id": self.chat_id,
            "message_thread_id": self.thread_id,
            "text": message,
            "parse_mode": "HTML",
        });
        Ok(Client::new()
            .post(url)
            .json(&message)
            .send()
            .await?
            .error_for_status()?
            .text()
            .await?)
    }

    pub async fn send_audio(&self, audio: &str, caption: &str) -> Result<String, CustomError>{
        let url = format!("{URL}/bot{}/sendAudio", self.token);
        let message = json!({
            "chat_id": self.chat_id,
            "message_thread_id": self.thread_id,
            "audio": audio,
            "caption": caption,
            "parse_mode": "HTML",
        });
        Ok(Client::new()
            .post(url)
            .json(&message)
            .send()
            .await?
            .error_for_status()?
            .text()
            .await?)
    }
}

#[cfg(test)]
mod tests {
    use crate::models::config::Configuration;
    use tokio;

    #[tokio::test]
    async fn send_audio_test(){
        let config = Configuration::load().await.unwrap();
        let telegram = config.get_telegram();
        let message = r#"Este es un "audio" de prueba"#;
        let audio = "https://anchor.fm/s/5a5b39c/podcast/play/78552354/https%3A%2F%2Fd3ctxlq1ktw2nl.cloudfront.net%2Fstaging%2F2023-10-13%2Ff899af71-0889-b3fe-07e1-2e6d935da0b1.mp3";
        let result = telegram.send_audio(audio, message).await;
        println!("{:?}", result);
        assert!(result.is_ok())
    }

    #[tokio::test]
    async fn send_message_test(){
        let config = Configuration::load().await.unwrap();
        let telegram = config.get_telegram();
        let message = r#"Este es un "título" de prueba"#;
        let result = telegram.send_message(&message).await;
        println!("{:?}", result);
        assert!(result.is_ok())
    }
}

