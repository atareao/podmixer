use reqwest::{Client, multipart};
use serde::{Serialize, Deserialize};
use tracing::debug;
use super::Error;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Telegram{
    active: bool,
    token: String,
    chat_id: String,
    #[serde(default = "default_thread_id")]
    thread_id: String,
}

fn default_thread_id() -> String{
    "0".to_string()
}

const URL: &str = "https://api.telegram.org";

impl Telegram{
    pub fn new(active: bool, token: String, chat_id: i64, thread_id: i64) -> Self{
        Self{
            active,
            token,
            chat_id: chat_id.to_string(),
            thread_id: thread_id.to_string(),
        }
    }

    pub fn is_active(&self) -> bool{
        self.active
    }

    #[allow(dead_code)]
    pub async fn send_message(&self, message: &str) -> Result<String, Error>{
        let url = format!("{URL}/bot{}/sendMessage", self.token);
        let params = vec![
            ("chat_id", self.chat_id.as_str()),
            ("message_thread_id", self.thread_id.as_str()),
            ("text", message),
            ("parse_mode", "HTML"),
        ];
        Client::new()
            .post(url)
            .form(&params)
            .send()
            .await?
            .error_for_status()?
            .text()
            .await.map_err(|e| e.into())
    }

    pub async fn send_audio(&self, filename: &str, filepath: &str, caption: &str) -> Result<String, Error>{
        debug!("send_audio");
        let url = format!("{URL}/bot{}/sendAudio", self.token);
        debug!("url: {url}");
        let file = tokio::fs::read(filepath).await?;
        let part = multipart::Part::bytes(file)
            .file_name(filename.to_string());
        let form = multipart::Form::new()
            .text("chat_id", self.chat_id.to_string())
            .text("message_thread_id", self.thread_id.to_string())
            .text("caption", caption.to_string())
            .text("parse_mode", "HTML".to_string())
            .part("audio", part);
        Client::new()
            .post(url)
            .multipart(form)
            .send()
            .await?
            .error_for_status()?
            .text()
            .await.map_err(|e| e.into())
    }
}

#[cfg(test)]
mod test{
    use super::Telegram;
    use dotenv::dotenv;
    use std::{env, str::FromStr};
    use tracing_subscriber::{
        EnvFilter,
        layer::SubscriberExt,
        util::SubscriberInitExt
    };

    #[tokio::test]
    async fn telegram(){
        tracing_subscriber::registry()
            .with(EnvFilter::from_str("debug").unwrap())
            .with(tracing_subscriber::fmt::layer())
        .init();
        dotenv().ok();
        let token = env::var("TELEGRAM_TOKEN")
            .expect("Cant get token");
        let chat_id = env::var("TELEGRAM_CHAT_ID")
            .expect("Cant get chat_id")
            .parse()
            .expect("Cant convert chat_id");
        let thread_id = env::var("TELEGRAM_THREAD_ID")
            .expect("Cant get thread_id")
            .parse()
            .expect("Cant convert thread_id");
        let filename = "example.mp3";
        let audio = "/data/rust/podmixer/5d279930-4426-d35f-7c3b-90314d30595d.mp3";
        let telegram = Telegram::new(true, token, chat_id, thread_id);
        assert!(telegram.send_message("Prueba").await.is_ok());
        let response = telegram.send_audio(filename, audio, "Esto es una prueba").await;
        println!("{:?}", response);
        assert!(response.is_ok());
    }
}
