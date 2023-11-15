use serde::{Serialize, Deserialize};
use reqwest::{Client, Error};
use base64::{
    Engine,
    engine::general_purpose::STANDARD
};
use std::collections::HashMap;

const X_URL: &'static str = "https://twitter.com/api";

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Twitter{
    client_id: String,
    client_secret: String,
    access_token: String,
    refresh_token: String,
}

impl Twitter {
    pub fn new(client_id: String, client_secret: String, access_token: String, refresh_token: String) -> Self{
        Self{
            client_id,
            client_secret,
            access_token,
            refresh_token,
        }
    }

    pub async fn get_access_token(self, code: &str) -> Result<String, Error>{
        let mut basic_auth = String::new();
        STANDARD.encode_string(
            format!("{}:{}", self.client_id, self.client_secret).as_bytes(),
            &mut basic_auth);
        let url = format!("{X_URL}/2/oauth2/token");
        let mut params = HashMap::new();
        params.insert("code", code);
        params.insert("grant_type", "authorization_code");
        params.insert("redirect_uri", "localhost");
        params.insert("code_verifier", "challenge");
        Ok(Client::new()
            .post(url)
            .header("Content-Type", "application/x-www-form-urlencoded")
            .header("Authorization", basic_auth)
            .form(&params)
            .send()
            .await?
            .error_for_status()?
            .json()
            .await?)
    }

    pub async fn update_access_token(self, code: &str) -> Result<String, Error>{
        let mut basic_auth = String::new();
        STANDARD.encode_string(
            format!("{}:{}", self.client_id, self.client_secret).as_bytes(),
            &mut basic_auth);
        let url = format!("{X_URL}/2/oauth2/token");
        let mut params = HashMap::new();
        params.insert("refresh_token", self.refresh_token);
        params.insert("grant_type", "refresh_token".to_string());
        params.insert("client_id", self.client_id);
        Ok(Client::new()
            .post(url)
            .header("Content-Type", "application/x-www-form-urlencoded")
            .header("Authorization", basic_auth)
            .form(&params)
            .send()
            .await?
            .error_for_status()?
            .json()
            .await?)
    }
}

