use reqwest::{Client, multipart};
use serde::{Serialize, Deserialize};
use tracing::debug;
use super::Error;
use sqlx::sqlite::SqlitePool;
use crate::models::Param;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Telegram{
    pub token: String,
    pub chat_id: String,
    #[serde(default = "default_thread_id")]
    pub thread_id: String,
    pub template: String,
    pub active: bool,
}

fn default_thread_id() -> String{
    "0".to_string()
}

const URL: &str = "https://api.telegram.org";

impl Telegram{
    pub fn new(active: bool, token: String, chat_id: String, thread_id: String, template: String) -> Self{
        Self{
            active,
            token,
            chat_id,
            thread_id,
            template,
        }
    }

    pub fn is_active(&self) -> bool{
        self.active
    }

    pub async fn get(pool: &SqlitePool) -> Result<Telegram, Error> {
        debug!("get_telegram");
        let active_str = Param::get(pool, "telegram_active")
            .await?;
        let active = active_str == "TRUE";
        let token = Param::get(pool, "telegram_token").await?;
        let chat_id = Param::get(pool, "telegram_chat_id")
            .await?;
        let thread_id = Param::get(pool, "telegram_thread_id")
            .await?;
        let template = Param::get(pool, "telegram_template").await?;
        Ok(Telegram::new(active, token, chat_id, thread_id, template))
    }
    pub async fn set(pool: &SqlitePool, telegram: &Telegram) -> Result<Telegram, Error> {
        debug!("save_telegram, {:?}", telegram);
        Param::set(pool, "telegram_active", &telegram.active.to_string().to_uppercase()).await?;
        Param::set(pool, "telegram_token", &telegram.token).await?;
        Param::set(pool, "telegram_chat_id", &telegram.chat_id).await?;
        Param::set(pool, "telegram_thread_id", &telegram.thread_id).await?;
        Param::set(pool, "telegram_template", &telegram.template).await?;
        Self::get(pool).await
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
        let telegram = Telegram::new(true, token, chat_id, thread_id, "".to_string());
        assert!(telegram.send_message("Prueba").await.is_ok());
        let response = telegram.send_audio(filename, audio, "Esto es una prueba").await;
        println!("{:?}", response);
        assert!(response.is_ok());
    }
}

