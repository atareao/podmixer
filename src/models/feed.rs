use serde::{Deserialize, Serialize};
use rss::{
    ChannelBuilder,
    ImageBuilder,
    CategoryBuilder,
    Item,
    extension::itunes::ITunesChannelExtensionBuilder
};
use super::Error;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Feed{
    pub title: String,
    pub link: String,
    pub image_url: String,
    pub category: String,
    pub rating: String,
    pub description: String,
    pub author: String,
    pub explicit: String,
    pub keywords: String,
}

impl Feed {
    pub fn rss(&self, episodes: Vec<Item>) -> Result<String, Error>{
        let image = ImageBuilder::default()
            .url(&self.image_url)
            .build();
        let category = CategoryBuilder::default()
            .name(&self.category)
            .build();
        let itunes = ITunesChannelExtensionBuilder::default()
            .author(Some(self.author.clone()))
            .build();
        let mut channel = ChannelBuilder::default()
            .title(&self.title)
            .link(&self.link)
            .image(Some(image))
            .category(category)
            .rating(Some(self.rating.clone()))
            .description(self.description.clone())
            .build();
        channel.set_itunes_ext(itunes);
        channel.set_items(episodes);
        channel.pretty_write_to(std::io::sink(), b' ', 4)?;
        Ok(channel.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::super::Configuration;
    use chrono::{Utc, Duration};
    use super::CompletePodcast;
    use tokio;

    #[tokio::test]
    async fn test_feed_older_than(){
        let config = Configuration::load().await.unwrap();
        let last_seven_days = (Utc::now() - Duration::days(7)).naive_local();
        let mut episodes = Vec::new();
        for podcast in config.podcasts.as_slice(){
            let complete = CompletePodcast::new(podcast).await.unwrap();
            let mut items = complete.get_older_than(&last_seven_days).unwrap();
            episodes.append(items.as_mut());
        }
        let feed = config.get_feed().rss(episodes).unwrap();
        println!("{}", feed);
        assert!(feed != "")
    }
}
