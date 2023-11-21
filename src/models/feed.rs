use serde::{Deserialize, Serialize};
use rss::{ChannelBuilder, ImageBuilder, CategoryBuilder, extension::itunes::ITunesChannelExtensionBuilder};
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
    pub fn older_than(&self, podcasts: Vec<CompletePodcast>) -> Result<String, Error>{
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
        Ok("".to_string())
    }
    
}

