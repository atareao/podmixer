use async_trait::async_trait;
use reqwest::Client;
use serde_json::{json, Value};

use super::super::Error;
use super::types::{PublisherImpl, PublisherType};

pub struct MatrixPublisher {
    id: String,
    homeserver_url: String,
    room_id: String,
    access_token: String,
}

impl MatrixPublisher {
    pub fn new(id: String, config: &Value) -> Option<Self> {
        Some(Self {
            id,
            homeserver_url: config.get("homeserver_url")?.as_str()?.to_string(),
            room_id: config.get("room_id")?.as_str()?.to_string(),
            access_token: config.get("access_token")?.as_str()?.to_string(),
        })
    }
}

#[async_trait]
impl PublisherImpl for MatrixPublisher {
    async fn publish(&self, title: &str, description: &str, url: &str) -> Result<String, Error> {
        let html = format!(
            "<b>{}</b><br/>{}<br/><a href=\"{}\">{}</a>",
            title, description, url, url
        );
        let body = json!({
            "msgtype": "m.text",
            "format": "org.matrix.custom.html",
            "body": format!("{}\n{}\n{}", title, description, url),
            "formatted_body": html,
        });
        let txn_id = uuid::Uuid::new_v4().to_string();
        let api_url = format!(
            "{}/_matrix/client/v3/rooms/{}/send/m.room.message/{}",
            self.homeserver_url.trim_end_matches('/'),
            self.room_id,
            txn_id
        );
        let resp = Client::new()
            .put(&api_url)
            .header("Authorization", format!("Bearer {}", self.access_token))
            .json(&body)
            .send()
            .await?
            .error_for_status()?
            .text()
            .await?;
        Ok(resp)
    }

    fn publisher_type(&self) -> PublisherType {
        PublisherType::Matrix
    }

    fn id(&self) -> &str {
        &self.id
    }
}