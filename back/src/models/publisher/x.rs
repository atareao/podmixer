use async_trait::async_trait;
use reqwest::Client;
use serde_json::{json, Value};

use super::super::Error;
use super::types::{PublisherImpl, PublisherType};

pub struct XPublisher {
    id: String,
    client_id: String,
    client_secret: String,
    access_token: String,
    refresh_token: String,
}

impl XPublisher {
    pub fn new(id: String, config: &Value) -> Option<Self> {
        Some(Self {
            id,
            client_id: config.get("client_id")?.as_str()?.to_string(),
            client_secret: config.get("client_secret")?.as_str()?.to_string(),
            access_token: config.get("access_token")?.as_str()?.to_string(),
            refresh_token: config.get("refresh_token")?.as_str()?.to_string(),
        })
    }

    pub async fn refresh_access_token(&mut self) -> Result<(), Error> {
        let url = "https://api.twitter.com/2/oauth2/token";
        let params = [
            ("refresh_token", self.refresh_token.as_str()),
            ("grant_type", "refresh_token"),
            ("client_id", self.client_id.as_str()),
        ];
        let data: Value = Client::new()
            .post(url)
            .basic_auth(&self.client_id, Some(&self.client_secret))
            .form(&params)
            .send()
            .await?
            .error_for_status()?
            .json()
            .await?;
        self.access_token = data["access_token"].as_str().unwrap_or("").to_string();
        self.refresh_token = data["refresh_token"].as_str().unwrap_or("").to_string();
        Ok(())
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

    fn publisher_type(&self) -> PublisherType {
        PublisherType::X
    }

    fn id(&self) -> &str {
        &self.id
    }
}