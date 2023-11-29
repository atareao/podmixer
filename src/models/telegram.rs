use reqwest::Client;
use serde_json::json;
use serde::{Serialize, Deserialize};
use tracing::{debug, error};

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

const URL: &'static str = "https://api.telegram.org";

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

    pub async fn send_message(&self, message: &str){
        let url = format!("{URL}/bot{}/sendMessage", self.token);
        let message = json!({
            "chat_id": self.chat_id,
            "message_thread_id": self.thread_id,
            "text": message,
            "parse_mode": "HTML",
        });
        match Client::new()
            .post(url)
            .json(&message)
            .send()
            .await{
                Ok(data) => {
                    match data.error_for_status(){
                        Ok(response) => {
                            match response.text().await {
                                Ok(text) => debug!("{:?}", text),
                                Err(e) => error!("{:?}", e),
                            }

                        },
                        Err(e) => error!("{:?}", e),
                    }
                },
                Err(e) => error!("{:?}", e),
            };
    }

    pub async fn send_audio(&self, audio: &str, caption: &str) {
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
        match Client::new()
            .post(url)
            .json(&message)
            .send()
            .await{
                Ok(data) => {
                    match data.error_for_status(){
                        Ok(response) => {
                            match response.text().await {
                                Ok(text) => debug!("{:?}", text),
                                Err(e) => error!("{:?}", e),
                            }

                        },
                        Err(e) => error!("{:?}", e),
                    }
                },
                Err(e) => error!("{:?}", e),
            };
    }
}
