use super::super::Error;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};

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
    #[serde(default)]
    pub id: String,
    pub name: String,
    pub publisher_type: PublisherType,
    pub config: serde_json::Value,
    pub template: String,
    pub reply_template: String,
    pub active: bool,
    #[serde(default)]
    pub created_at: String,
    #[serde(default)]
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

#[async_trait]
pub trait PublisherImpl: Send + Sync {
    async fn publish(&self, title: &str, description: &str, url: &str) -> Result<String, Error>;
}
