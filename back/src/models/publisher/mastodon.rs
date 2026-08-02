use async_trait::async_trait;
use reqwest::Client;
use serde_json::Value;

use super::super::Error;
use super::types::PublisherImpl;

pub struct MastodonPublisher {
    server_url: String,
    access_token: String,
}

impl MastodonPublisher {
    pub fn new(config: &Value) -> Option<Self> {
        Some(Self {
            server_url: config.get("server_url")?.as_str()?.to_string(),
            access_token: config.get("access_token")?.as_str()?.to_string(),
        })
    }
}

#[async_trait]
impl PublisherImpl for MastodonPublisher {
    async fn publish(&self, title: &str, description: &str, url: &str) -> Result<String, Error> {
        let status = format!("{}\n\n{}\n{}", title, description, url);
        let api_url = format!(
            "{}/api/v1/statuses",
            self.server_url.trim_end_matches('/')
        );
        let params = [("status", status.as_str())];
        let resp = Client::new()
            .post(&api_url)
            .header("Authorization", format!("Bearer {}", self.access_token))
            .form(&params)
            .send()
            .await?
            .error_for_status()?
            .text()
            .await?;
        Ok(resp)
    }
}