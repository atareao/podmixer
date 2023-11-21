use serde::{Deserialize, Serialize};
use rss::{Channel, Item};
use super::Error;
use chrono::{NaiveDateTime, DateTime, Utc, Duration};

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Podcast{
    pub name: String,
    pub url: String,
    pub last_pub_date: NaiveDateTime,
}

#[derive(Debug)]
pub struct CompletePodcast{
    pub podcast: Podcast,
    pub channel: Channel,
}

impl CompletePodcast {
    pub async fn new(podcast: &Podcast) -> Result<Self, Error>{
        let client = reqwest::Client::new();
        let content = client
            .get(&podcast.url)
            .send()
            .await?
            .error_for_status()?
            .bytes()
            .await?;
        let channel = Channel::read_from(&content[..])?;
        Ok(Self {
            podcast: podcast.clone(),
            channel,
        })
    }

    pub fn get_new(&self) -> Result<Vec<Item>, Error>{
        Ok(self.get_older_than(&self.podcast.last_pub_date)?)
    }

    pub fn get_older_than_days(&self, days: i64) -> Result<Vec<Item>, Error>{
        let datetime = (Utc::now() - Duration::days(days)).naive_local();
        Ok(self.get_older_than(&datetime)?)
    }

    pub fn get_older_than(&self, datetime: &NaiveDateTime) -> Result<Vec<Item>, Error>{
        let mut older_than: Vec<Item> = Vec::new();
        for item in self.channel.items.as_slice(){
            if let Some(pub_date_str) = item.pub_date(){
                if let Ok(pub_date) = DateTime::parse_from_rfc2822(pub_date_str){
                    if pub_date.timestamp() > datetime.timestamp(){
                        older_than.push(item.clone());
                    }
                }
            }
        }
        Ok(older_than)
    }
}

#[cfg(test)]
mod tests {
    use crate::models::config::Configuration;
    use super::CompletePodcast;
    use tokio;

    #[tokio::test]
    async fn get_new(){
        let config = Configuration::load().await.unwrap();
        let podcast = CompletePodcast::new(config.get_podcasts().first().unwrap())
            .await
            .unwrap();
        let items = podcast.get_new();
        assert!(items.is_ok());
    }

    #[tokio::test]
    async fn get_older_than_days(){
        let config = Configuration::load().await.unwrap();
        let podcast = CompletePodcast::new(config.get_podcasts().first().unwrap())
            .await
            .unwrap();
        let items = podcast.get_older_than_days(7);
        assert!(items.is_ok());
    }

    #[tokio::test]
    async fn get_older_than_days_zero(){
        let config = Configuration::load().await.unwrap();
        let podcast = CompletePodcast::new(config.get_podcasts().first().unwrap())
            .await
            .unwrap();
        let items = podcast.get_older_than_days(0);
        assert!(items.is_ok());
    }
}
