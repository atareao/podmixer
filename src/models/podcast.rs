use serde::{Deserialize, Serialize};
use reqwest::Error;
use rss::Channel;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Podcast{
    pub title: String,
    pub author: String,
    pub last_pub_date: String,
    pub url: String,
    pub image_url: String,
}

impl Podcast {
    pub async fn read(self) -> Result<String, Error>{
        let client = reqwest::Client::new();
        Ok(client
            .get(self.url)
            .send()
            .await?
            .error_for_status()?
            .text()
            .await?)
    }
}
