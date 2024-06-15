use serde::{Serialize, Deserialize};
use reqwest::Client;
use serde_json::{Value, json};
use tracing::debug;
use super::Error;

const X_URL: &str = "https://api.twitter.com";

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Twitter{
    active: bool,
    client_id: String,
    client_secret: String,
    access_token: String,
    refresh_token: String,
}

impl Twitter {
    pub fn new(active: bool, client_id: String, client_secret: String, access_token: String, refresh_token: String) -> Self{
        Self{
            active,
            client_id,
            client_secret,
            access_token,
            refresh_token,
        }
    }

    pub fn is_active(&self) -> bool{
        self.active
    }

    pub fn get_access_token(&self) -> &str{
        &self.access_token
    }

    pub fn get_refresh_token(&self) -> &str{
        &self.refresh_token
    }

    pub async fn update_access_token(&mut self) -> Result<(), Error>{
        debug!("Update access token");
        let url = format!("{X_URL}/2/oauth2/token");
        debug!("Url: {url}");
        let params = [
            ("refresh_token", &self.refresh_token),
            ("grant_type", &"refresh_token".to_string()),
            ("client_id", &self.client_id)
        ];
        // .header(header::CONTENT_TYPE, "application/x-www-form-urlencoded")
        debug!("Params: {:?}", params);
        debug!("Before access_token: {}", &self.access_token);
        debug!("Before refresh_token: {}", &self.refresh_token);
        let data: Value = Client::new()
            .post(url)
            .basic_auth(&self.client_id, Some(&self.client_secret))
            .form(&params)
            .send()
            .await?
            .error_for_status()?
            .json()
            .await?;
        debug!("Data: {:?}", data);
        self.access_token = data.get("access_token").unwrap().as_str().unwrap().to_string();
        self.refresh_token = data.get("refresh_token").unwrap().as_str().unwrap().to_string();
        debug!("New access_token: {}", &self.access_token);
        debug!("New refresh_token: {}", &self.refresh_token);
        Ok(())
    }

    pub async fn post(&self, message: &str) -> Result<String, Error>{
        debug!("post");
        let url = format!("{X_URL}/2/tweets");
        debug!("url: {url}. message: {message}");
        let message = json!({
            "text": message
        });
        debug!("access_token: {}", self.access_token);
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

#[cfg(test)]
mod test{
    use super::Twitter;
    use dotenv::dotenv;
    use std::{env, str::FromStr};
    use tracing_subscriber::{
        EnvFilter,
        layer::SubscriberExt,
        util::SubscriberInitExt
    };

    #[tokio::test]
    async fn twitter(){
        tracing_subscriber::registry()
            .with(EnvFilter::from_str("debug").unwrap())
            .with(tracing_subscriber::fmt::layer())
        .init();
        dotenv().ok();
        let active = true;
        let client_id = env::var("X_CLIENT_ID").expect("X_CLIENT_ID");
        let client_secret = env::var("X_CLIENT_SECRET").expect("X_CLIENT_SECRET");
        let access_token = env::var("X_ACCESS_TOKEN").expect("X_ACCESS_TOKEN");
        let refresh_token = env::var("X_REFRESH_TOKEN").expect("X_REFRESH_TOKEN");
        let mut twitter = Twitter::new(active, client_id, client_secret, access_token, refresh_token);
        assert!(twitter.update_access_token().await.is_ok());
        let response = twitter.post("Prueba").await;
        match response{
            Ok(_) => println!("Populated in Twitter"),
            Err(ref error) => {
                println!("Could NOT populate in Twitter: {error}");
                let mut next_error = error.source();
                // render causes as well
                while next_error.is_some(){
                    println!("caused by: {:#}", next_error.unwrap());
                    next_error = next_error.unwrap().source();
                }
            },
        }
        println!("{:?}", response);
    }
}
