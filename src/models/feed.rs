use serde::{Deserialize, Serialize};
use chrono::NaiveDateTime;
use rss::{
    ChannelBuilder,
    ImageBuilder,
    CategoryBuilder,
    Item,
    extension::itunes::ITunesChannelExtensionBuilder
};
use super::{
    CompletePodcast,
    Error,
};

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
    pub fn older_than(&self, datetime: &NaiveDateTime, podcasts: Vec<CompletePodcast>) -> Result<String, Error>{
        let image = ImageBuilder::default()
            .url(&self.image_url)
            .build();
        let category = CategoryBuilder::default()
            .name(&self.category)
            .build();
        let itunes = ITunesChannelExtensionBuilder::default()
            .author(Some(self.author.clone()))
            .build();
        let mut older_than: Vec<Item> = Vec::new();
        for podcast in podcasts.as_slice(){
            let items = podcast.get_older_than(datetime)?;
            for item in items{
                older_than.push(item);
            }
        }
        let mut channel = ChannelBuilder::default()
            .title(&self.title)
            .link(&self.link)
            .image(Some(image))
            .category(category)
            .rating(Some(self.rating.clone()))
            .description(self.description.clone())
            .build();
        channel.set_itunes_ext(itunes);
        channel.set_items(older_than);
        Ok("".to_string())
    }
    
}

