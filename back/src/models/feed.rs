use serde::{Deserialize, Serialize};
use rss::{
    ChannelBuilder,
    ImageBuilder,
    CategoryBuilder,
    Item,
    extension::itunes::ITunesChannelExtensionBuilder
};
use super::Error;
use tracing::debug;
use sqlx::sqlite::SqlitePool;
use crate::models::Param;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Feed{
    pub title: String,
    pub link: String,
    pub image_url: String,
    pub category: String,
    pub rating: String,
    pub description: String,
    pub author: String,
    pub explicit: bool,
    pub keywords: String,
}

impl Feed {
    #[allow(clippy::too_many_arguments)]
    pub fn new(title: String, link: String, image_url: String,
               category: String, rating: String, description: String,
               author: String, explicit: bool, keywords: String) -> Self{
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

    pub async fn get(pool: &SqlitePool) -> Result<Feed, Error> {
        debug!("get_feed");
        let feed_title = Param::get(pool, "feed_title").await?;
        let feed_link = Param::get(pool, "feed_link").await?;
        let feed_image_url = Param::get(pool, "feed_image_url").await?;
        let feed_category = Param::get(pool, "feed_category").await?;
        let feed_rating = Param::get(pool, "feed_rating").await?;
        let feed_description = Param::get(pool, "feed_description").await?;
        let feed_author = Param::get(pool, "feed_author").await?;
        let feed_explicit_str = Param::get(pool, "feed_explicit").await?;
        let feed_explicit = feed_explicit_str == "TRUE";
        let feed_keywords = Param::get(pool, "feed_keywords").await?;
        Ok(Feed::new(feed_title, feed_link, feed_image_url, feed_category,
            feed_rating, feed_description, feed_author, feed_explicit,
            feed_keywords))
    }

    pub async fn set(pool: &SqlitePool, feed: &Feed) -> Result<Feed, Error> {
        debug!("save_feed");
        Param::set(pool, "feed_title", &feed.title).await?;
        Param::set(pool, "feed_link", &feed.link).await?;
        Param::set(pool, "feed_image_url", &feed.image_url).await?;
        Param::set(pool, "feed_category", &feed.category).await?;
        Param::set(pool, "feed_rating", &feed.rating).await?;
        Param::set(pool, "feed_description", &feed.description).await?;
        Param::set(pool, "feed_author", &feed.author).await?;
        Param::set(pool, "feed_explicit", &feed.explicit.to_string().to_uppercase()).await?;
        Param::set(pool, "feed_keywords", &feed.keywords).await?;
        Self::get(pool).await
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

