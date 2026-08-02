use async_trait::async_trait;
use reqwest::Client;
use serde_json::{json, Value};
use url::Url;

use super::super::Error;
use super::types::PublisherImpl;

pub struct MastodonPublisher {
    pub server_url: String,
    pub access_token: Option<String>,
    pub client_id: Option<String>,
    pub client_secret: Option<String>,
    pub redirect_uri: String,
}

impl MastodonPublisher {
    pub fn new(config: &Value) -> Option<Self> {
        Some(Self {
            server_url: config.get("server_url")?.as_str()?.to_string(),
            client_id: config
                .get("client_id")
                .and_then(|v| v.as_str())
                .map(String::from),
            client_secret: config
                .get("client_secret")
                .and_then(|v| v.as_str())
                .map(String::from),
            access_token: config
                .get("access_token")
                .and_then(|v| v.as_str())
                .map(String::from),
            redirect_uri: config
                .get("redirect_uri")
                .and_then(|v| v.as_str())
                .unwrap_or("http://localhost:3000/api/v1/oauth/callback")
                .to_string(),
        })
    }

    pub async fn register_app(&self) -> Result<(String, String), Box<dyn std::error::Error + Send + Sync>> {
        let url = format!("{}/api/v1/apps", self.server_url.trim_end_matches('/'));
        let payload = json!({
            "client_name": "podmixer",
            "redirect_uris": self.redirect_uri,
            "scopes": "read write push"
        });
        let resp = Client::new().post(&url).json(&payload).send().await?;
        if resp.status().is_success() {
            let data: Value = resp.json().await?;
            let client_id = data["client_id"].as_str().unwrap_or("").to_string();
            let client_secret = data["client_secret"].as_str().unwrap_or("").to_string();
            Ok((client_id, client_secret))
        } else {
            Err(format!(
                "Mastodon app registration failed: {}",
                resp.text().await.unwrap_or_default()
            )
            .into())
        }
    }

    pub fn generate_auth_url(&self, state: Option<String>) -> String {
        let state = state.unwrap_or_else(|| uuid::Uuid::new_v4().to_string());
        let scope = "read write push";
        let mut url = Url::parse(&format!(
            "{}/oauth/authorize",
            self.server_url.trim_end_matches('/')
        ))
        .unwrap();
        url.query_pairs_mut()
            .append_pair("response_type", "code")
            .append_pair("client_id", self.client_id.as_deref().unwrap_or_default())
            .append_pair("redirect_uri", &self.redirect_uri)
            .append_pair("scope", scope)
            .append_pair("state", &state);
        url.to_string()
    }

    pub async fn exchange_code_for_tokens(
        &self,
        code: &str,
    ) -> Result<(String, Option<String>, u64), Box<dyn std::error::Error + Send + Sync>> {
        let url = format!("{}/oauth/token", self.server_url.trim_end_matches('/'));
        let client_id = self.client_id.as_deref().ok_or("No client_id")?;
        let client_secret = self.client_secret.as_deref().ok_or("No client_secret")?;
        let params = [
            ("grant_type", "authorization_code"),
            ("code", code),
            ("client_id", client_id),
            ("client_secret", client_secret),
            ("redirect_uri", &self.redirect_uri),
        ];
        let resp = Client::new().post(&url).form(&params).send().await?;
        if resp.status().is_success() {
            let data: Value = resp.json().await?;
            let access_token = data["access_token"].as_str().unwrap_or("").to_string();
            Ok((access_token, None, 0))
        } else {
            Err(format!(
                "Mastodon token exchange failed: {}",
                resp.text().await.unwrap_or_default()
            )
            .into())
        }
    }
}

#[async_trait]
impl PublisherImpl for MastodonPublisher {
    async fn publish(&self, title: &str, description: &str, url: &str) -> Result<String, Error> {
        let token = self
            .access_token
            .as_deref()
            .ok_or_else(|| Error::from("No access token for Mastodon"))?;
        let status = format!("{}\n\n{}\n{}", title, description, url);
        let api_url = format!(
            "{}/api/v1/statuses",
            self.server_url.trim_end_matches('/')
        );
        let params = [("status", status.as_str())];
        let resp = Client::new()
            .post(&api_url)
            .header("Authorization", format!("Bearer {}", token))
            .form(&params)
            .send()
            .await?
            .error_for_status()?
            .text()
            .await?;
        Ok(resp)
    }
}