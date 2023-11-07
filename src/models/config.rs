use serde::{Serialize, Deserialize};
use tokio::fs::read_to_string;
use std::process;

use super::{
    Podcast,
    Feed,
    Telegram,
    Twitter,
};

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Configuration{
    log_level: String,
    frequency: i32,
    long_in_days: i32,
    feed: Feed,
    podcasts: Vec<Podcast>,
    telegram: Telegram,
    twitter: Twitter,
}

impl Configuration{
    pub fn get_log_level(&self) -> &str{
        &self.log_level
    }
    pub fn get_frequency(&self) -> i32{
        self.frequency
    }
    pub fn get_long_in_days(&self) -> i32{
        self.long_in_days
    }
    pub fn get_feed(&self) -> &Feed{
        &self.feed
    }
    pub fn get_podcasts(&self) -> &Vec<Podcast>{
        &self.podcasts
    }
    pub fn get_telegram(&self) -> &Telegram{
        &self.telegram
    }
    pub fn get_twitter(&self) -> &Twitter{
        &self.twitter
    }

    pub async fn read_configuration() -> Configuration{
        let content = match read_to_string("config.yml")
            .await {
                Ok(value) => value,
                Err(e) => {
                    println!("Error with config file `config.yml`: {}",
                        e.to_string());
                    process::exit(0);
                }
            };
        match serde_yaml::from_str(&content){
            Ok(configuration) => configuration,
            Err(e) => {
                println!("Error with config file `config.yml`: {}",
                    e.to_string());
                process::exit(0);
            }
        }
    }
}

