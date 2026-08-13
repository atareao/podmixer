use std::time::Duration;

use async_trait::async_trait;
use base64::engine::general_purpose;
use base64::Engine as _;
use minijinja::Environment;
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
    pub template: String,
    pub reply_template: String,
}

impl XPublisher {
    pub fn new(config: &Value, template: &str, reply_template: &str) -> Option<Self> {
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
            template: template.to_string(),
            reply_template: reply_template.to_string(),
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
        let auth =
            general_purpose::STANDARD.encode(format!("{}:{}", self.client_id, self.client_secret));
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
    pub async fn refresh_access_token(
        &self,
    ) -> Result<(String, String), Box<dyn std::error::Error + Send + Sync>> {
        let rt = if self.refresh_token.is_empty() {
            return Err("No refresh token".into());
        } else {
            &self.refresh_token
        };
        let auth =
            general_purpose::STANDARD.encode(format!("{}:{}", self.client_id, self.client_secret));
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
        let api_url = "https://api.twitter.com/2/tweets";
        let client = Client::new();

        // Render main tweet text from template
        let main_text = if self.template.is_empty() {
            title.to_string()
        } else {
            let mut env = Environment::new();
            env.add_template("tweet", &self.template)
                .map_err(|e| format!("Invalid X template: {}", e))?;
            let tmpl = env
                .get_template("tweet")
                .map_err(|e| format!("X template not found: {}", e))?;
            tmpl.render(json!({
                "title": title,
                "description": _description,
                "url": url,
            }))
            .map_err(|e| format!("X template render error: {}", e))?
        };
        let main_resp = client
            .post(api_url)
            .header("Content-Type", "application/json")
            .header("Authorization", format!("Bearer {}", self.access_token))
            .json(&json!({ "text": &main_text }))
            .send()
            .await?;

        if !main_resp.status().is_success() {
            let body = main_resp.text().await.unwrap_or_default();
            return Err(format!("X main tweet failed: {}", body).into());
        }

        let main_data: Value = main_resp.json().await?;
        let tweet_id = main_data["data"]["id"]
            .as_str()
            .ok_or_else(|| Error::from("X response missing data.id after posting main tweet"))?
            .to_string();

        // Small delay to ensure sequential execution
        tokio::time::sleep(Duration::from_millis(200)).await;

        // Step 2: Render reply text from reply_template
        let reply_text = if self.reply_template.is_empty() {
            url.to_string()
        } else {
            let mut env = Environment::new();
            env.add_template("reply", &self.reply_template)
                .map_err(|e| format!("Invalid X reply template: {}", e))?;
            let tmpl = env
                .get_template("reply")
                .map_err(|e| format!("X reply template not found: {}", e))?;
            tmpl.render(json!({
                "title": title,
                "description": _description,
                "url": url,
            }))
            .map_err(|e| format!("X reply template render error: {}", e))?
        };

        // Step 3: Post reply with URL
        let reply_body = json!({
            "text": reply_text,
            "reply": {
                "in_reply_to_tweet_id": &tweet_id
            }
        });

        let reply_resp = client
            .post(api_url)
            .header("Content-Type", "application/json")
            .header("Authorization", format!("Bearer {}", self.access_token))
            .json(&reply_body)
            .send()
            .await;

        match reply_resp {
            Ok(resp) if resp.status().is_success() => {
                let reply_data: Value = resp.json().await?;
                let reply_id = reply_data["data"]["id"].as_str().unwrap_or("unknown");
                Ok(json!({
                    "main_tweet_id": tweet_id,
                    "reply_tweet_id": reply_id,
                    "status": "published"
                })
                .to_string())
            }
            Ok(resp) => {
                let body = resp.text().await.unwrap_or_default();
                // Main tweet succeeded but reply failed — log tweet_id for manual retry
                Err(format!("X reply failed for main tweet {}: {}", tweet_id, body).into())
            }
            Err(e) => {
                Err(format!("X reply request failed for main tweet {}: {}", tweet_id, e).into())
            }
        }
    }
}
