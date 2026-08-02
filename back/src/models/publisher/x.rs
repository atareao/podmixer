use async_trait::async_trait;
use base64::engine::general_purpose;
use base64::Engine as _;
use reqwest::Client;
use serde_json::{json, Value};
use url::Url;

use super::super::Error;
use super::types::PublisherImpl;

pub struct XPublisher {
    pub access_token: String,
    pub client_id: String,
    pub client_secret: String,
    #[allow(dead_code)]
    pub refresh_token: String,
    pub redirect_uri: String,
}

impl XPublisher {
    pub fn new(config: &Value) -> Option<Self> {
        Some(Self {
            client_id: config.get("client_id")?.as_str()?.to_string(),
            client_secret: config.get("client_secret")?.as_str()?.to_string(),
            access_token: config
                .get("access_token")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string(),
            refresh_token: config
                .get("refresh_token")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string(),
            redirect_uri: config
                .get("redirect_uri")
                .and_then(|v| v.as_str())
                .unwrap_or("http://localhost:3000/api/v1/oauth/callback")
                .to_string(),
        })
    }

    pub fn generate_auth_url(&self, state: Option<String>) -> (String, String) {
        let state = state.unwrap_or_else(|| uuid::Uuid::new_v4().to_string());
        let code_verifier = "challenge";
        let code_challenge = code_verifier;
        let scope = "tweet.read tweet.write users.read offline.access";
        let mut url = Url::parse("https://twitter.com/i/oauth2/authorize").unwrap();
        url.query_pairs_mut()
            .append_pair("response_type", "code")
            .append_pair("client_id", &self.client_id)
            .append_pair("redirect_uri", &self.redirect_uri)
            .append_pair("scope", scope)
            .append_pair("state", &state)
            .append_pair("code_challenge", code_challenge)
            .append_pair("code_challenge_method", "plain");
        (url.to_string(), code_verifier.to_string())
    }

    pub async fn exchange_code_for_tokens(
        &self,
        code: &str,
        code_verifier: &str,
    ) -> Result<(String, Option<String>, u64), Box<dyn std::error::Error + Send + Sync>> {
        let auth = general_purpose::STANDARD.encode(format!("{}:{}", self.client_id, self.client_secret));
        let params = [
            ("grant_type", "authorization_code"),
            ("code", code),
            ("redirect_uri", &self.redirect_uri),
            ("code_verifier", code_verifier),
        ];
        let resp = Client::new()
            .post("https://api.twitter.com/2/oauth2/token")
            .header("Authorization", format!("Basic {}", auth))
            .header("Content-Type", "application/x-www-form-urlencoded")
            .form(&params)
            .send()
            .await?;
        if resp.status().is_success() {
            let data: Value = resp.json().await?;
            let access_token = data["access_token"].as_str().unwrap_or("").to_string();
            let expires_in = data["expires_in"].as_u64().unwrap_or(7200);
            let refresh_token = data
                .get("refresh_token")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string());
            Ok((access_token, refresh_token, expires_in))
        } else {
            let body = resp.text().await.unwrap_or_default();
            Err(format!("X token exchange failed: {}", body).into())
        }
    }

    #[allow(dead_code)]
    pub async fn refresh_access_token(&self) -> Result<(String, String), Box<dyn std::error::Error + Send + Sync>> {
        let rt = if self.refresh_token.is_empty() {
            return Err("No refresh token".into());
        } else {
            &self.refresh_token
        };
        let auth = general_purpose::STANDARD.encode(format!("{}:{}", self.client_id, self.client_secret));
        let params = [("grant_type", "refresh_token"), ("refresh_token", rt)];
        let resp = Client::new()
            .post("https://api.twitter.com/2/oauth2/token")
            .header("Authorization", format!("Basic {}", auth))
            .header("Content-Type", "application/x-www-form-urlencoded")
            .form(&params)
            .send()
            .await?;
        if resp.status().is_success() {
            let data: Value = resp.json().await?;
            let new_at = data["access_token"].as_str().unwrap_or("").to_string();
            let new_rt = data
                .get("refresh_token")
                .and_then(|v| v.as_str())
                .unwrap_or(&self.refresh_token)
                .to_string();
            Ok((new_at, new_rt))
        } else {
            let body = resp.text().await.unwrap_or_default();
            Err(format!("X token refresh failed: {}", body).into())
        }
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