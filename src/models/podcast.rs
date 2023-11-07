use serde::{Deserialize, Serialize};
use rss::Channel;
use super::CustomError;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Podcast{
    pub title: String,
    pub author: String,
    pub last_pub_date: String,
    pub url: String,
    pub image_url: String,
}

impl Podcast {
    pub async fn read(self) -> Result<(), CustomError>{
        let client = reqwest::Client::new();
        let response = client
            .get(self.url)
            .send()
            .await
            .map_err(CustomError::Reqwest)?;
        let content = response.error_for_status()
            .map_err(CustomError::Reqwest)?
            .await
            .map_err(CustomError::Reqwest)?;
        Ok(())
    }
}
