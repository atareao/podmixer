use serde::{Serialize, Deserialize};
use reqwest::{Client, Error};
use std::collections::HashMap;

const X_URL: &'static str= "https://twitter.com/api";

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
        let url = format!("{X_URL}/2/oauth2/token");
        let mut params = HashMap::new();
        params.insert("code", code);
        params.insert("grant_type", "authorization_code");
        params.insert("redirect_uri", "localhost");
        params.insert("code_verifier", "challenge");
        Ok(Client::new()
            .post(url)
            .header("Content-Type", "application/x-www-form-urlencoded")
            .header("Authorization", "")
            .form(&params)
            .send()
            .await?
            .error_for_status()?
            .json()
            .await?)
    }

}

