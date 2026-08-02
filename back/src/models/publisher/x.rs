use async_trait::async_trait;
use reqwest::Client;
use serde_json::{json, Value};

use super::super::Error;
use super::types::PublisherImpl;

pub struct XPublisher {
    access_token: String,
}

impl XPublisher {
    pub fn new(config: &Value) -> Option<Self> {
        Some(Self {
            access_token: config.get("access_token")?.as_str()?.to_string(),
        })
    }
}

#[async_trait]
impl PublisherImpl for XPublisher {
    async fn publish(&self, title: &str, _description: &str, url: &str) -> Result<String, Error> {
        let message = format!("{} {}", title, url);
        let api_url = "https://api.twitter.com/2/tweets";
        let body = json!({ "text": message });
        let resp = Client::new()
            .post(api_url)
            .header("Content-Type", "application/json")
            .header("Authorization", format!("Bearer {}", self.access_token))
            .json(&body)
            .send()
            .await?
            .error_for_status()?
            .text()
            .await?;
        Ok(resp)
    }
}