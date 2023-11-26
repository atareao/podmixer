use serde::{Serialize, Deserialize};
use reqwest::Client;
use base64::{
    Engine,
    engine::general_purpose::STANDARD
};
use serde_json::{Value, json};
use std::collections::HashMap;
use super::Error;

const X_URL: &'static str = "https://api.twitter.com";

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

    async fn update_access_token(&mut self) -> Result<(), Error>{
        let mut basic_auth = String::new();
        STANDARD.encode_string(
            format!("{}:{}", self.client_id, self.client_secret).as_bytes(),
            &mut basic_auth);
        let basic_auth_header = format!("Basic {basic_auth}");
        println!("Authorization: {basic_auth_header}");
        let url = format!("{X_URL}/2/oauth2/token");
        println!("Url: {url}");
        let mut params = HashMap::new();
        let rt = String::from("refresh_token");
        params.insert("refresh_token", &self.refresh_token);
        params.insert("grant_type", &rt);
        params.insert("client_id", &self.client_id);
        println!("Params: {:?}", params);
        let data: Value = Client::new()
            .post(url)
            .header("Content-Type", "application/x-www-form-urlencoded")
            .header("Authorization", basic_auth_header)
            .form(&params)
            .send()
            .await?
            .error_for_status()?
            .json()
            .await?;
        println!("Data: {:?}", data);
        self.access_token = data.get("access_token").unwrap().as_str().unwrap().to_string();
        self.refresh_token = data.get("refresh_token").unwrap().as_str().unwrap().to_string();
        Ok(())
    }

    pub async fn post(&mut self, message: &str) -> Result<String, Error>{
        println!("post");
        self.update_access_token().await?;
        let url = format!("{X_URL}/2/tweets");
        let message = json!({
            "text": message
        });
        Ok(Client::new()
            .post(url)
            .header("Content-Type", "application/json")
            .header("Authorization", format!("Bearer {}", self.access_token))
            .json(&message)
            .send()
            .await?
            .error_for_status()?
            .text()
            .await?)
    }
}
