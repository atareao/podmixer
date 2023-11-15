use reqwest::{Client, Error};
use serde_json::json;
use serde::{Serialize, Deserialize};

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

    pub async fn send_message(&self, message: &str) -> Result<String, Error>{
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

    pub async fn send_audio(&self, audio: &str, caption: &str) -> Result<String, Error>{
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
    use dotenv::dotenv;
    use std::env;
    use crate::models::telegram::Telegram;
    use tokio;

    #[tokio::test]
    async fn send_audio_test(){
        dotenv().ok();
        let token = env::var("TELEGRAM_TOKEN").unwrap();
        let chat_id = env::var("TELEGRAM_CHAT_ID").unwrap().parse::<i64>().unwrap();
        let thread_id = env::var("TELEGRAM_THREAD_ID").unwrap().parse::<i64>().unwrap();
        let message = r#"Este es un "audio" de prueba"#;
        let audio = "https://anchor.fm/s/5a5b39c/podcast/play/78552354/https%3A%2F%2Fd3ctxlq1ktw2nl.cloudfront.net%2Fstaging%2F2023-10-13%2Ff899af71-0889-b3fe-07e1-2e6d935da0b1.mp3";
        println!("{}, {}, {}", token, chat_id, message);
        let telegram = Telegram::new(token, chat_id, thread_id);
        let result = telegram.send_audio(audio, message).await;
        println!("{:?}", result);
        assert!(result.is_ok())
    }

    #[tokio::test]
    async fn send_message_test(){
        dotenv().ok();
        let token = env::var("TELEGRAM_TOKEN").unwrap();
        let chat_id = env::var("TELEGRAM_CHAT_ID").unwrap().parse::<i64>().unwrap();
        let thread_id = env::var("TELEGRAM_THREAD_ID").unwrap().parse::<i64>().unwrap();
        let message = r#"Este es un "título" de prueba"#;
        println!("{}, {}, {}", token, chat_id, message);
        
        let telegram = Telegram::new(token, chat_id, thread_id);
        let result = telegram.send_message(&message).await;
        println!("{:?}", result);
        assert!(result.is_ok())
    }
}

