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
    #[allow(clippy::too_many_arguments)]
    pub fn new(title: String, link: String, image_url: String,
               category: String, rating: String, description: String,
               author: String, explicit: String, keywords: String) -> Self{
        Self{
            title,
            link,
            image_url,
            category,
            rating,
            description,
            author,
            explicit,
            keywords,
        }
    }
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
