use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use super::super::Error;

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum PublisherType {
    Telegram,
    X,
    Mastodon,
    Matrix,
}

impl PublisherType {
    pub fn as_str(&self) -> &'static str {
        match self {
            PublisherType::Telegram => "telegram",
            PublisherType::X => "x",
            PublisherType::Mastodon => "mastodon",
            PublisherType::Matrix => "matrix",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "telegram" => Some(PublisherType::Telegram),
            "x" => Some(PublisherType::X),
            "mastodon" => Some(PublisherType::Mastodon),
            "matrix" => Some(PublisherType::Matrix),
            _ => None,
        }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Publisher {
    pub id: String,
    pub name: String,
    pub publisher_type: PublisherType,
    pub config: serde_json::Value,
    pub template: String,
    pub active: bool,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct PublishLog {
    pub id: String,
    pub publisher_id: String,
    pub publisher_name: String,
    pub publisher_type: String,
    pub episode_title: String,
    pub status: String,
    pub message: String,
    pub created_at: String,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct PublisherConfig {
    pub bot_token: Option<String>,
    pub chat_id: Option<String>,
    pub message_thread_id: Option<String>,
    pub client_id: Option<String>,
    pub client_secret: Option<String>,
    pub access_token: Option<String>,
    pub refresh_token: Option<String>,
    pub server_url: Option<String>,
    pub access_token_mastodon: Option<String>,
    pub homeserver_url: Option<String>,
    pub room_id: Option<String>,
    pub access_token_matrix: Option<String>,
}

#[async_trait]
pub trait PublisherImpl: Send + Sync {
    async fn publish(&self, title: &str, description: &str, url: &str) -> Result<String, Error>;
    fn publisher_type(&self) -> PublisherType;
    fn id(&self) -> &str;
}