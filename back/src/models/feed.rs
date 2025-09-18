use serde::{Deserialize, Serialize};
use rss::{
    ChannelBuilder,
    ImageBuilder,
    CategoryBuilder,
    Item,
    extension::itunes::{
        ITunesChannelExtensionBuilder,
        ITunesCategoryBuilder,
        ITunesOwnerBuilder,
    },
};
use std::collections::BTreeMap;
use super::Error;
use tracing::debug;
use sqlx::sqlite::SqlitePool;
use crate::models::Param;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Feed{
    pub title: String,
    pub subtitle: String,
    pub summary: String,
    pub link: String,
    pub image_url: String,
    pub category: String,
    pub rating: String,
    pub description: String,
    pub author: String,
    pub explicit: bool,
    pub keywords: String,
    pub owner_name: String,
    pub owner_email: String,
}

impl Feed {
    #[allow(clippy::too_many_arguments)]
    pub fn new(title: String, subtitle: String, summary: String, link: String,
               image_url: String, category: String, rating: String,
               description: String, author: String, explicit: bool,
               keywords: String, owner_name: String, owner_email: String) -> Self{
        Self{
            title,
            subtitle,
            summary,
            link,
            image_url,
            category,
            rating,
            description,
            author,
            explicit,
            keywords,
            owner_name,
            owner_email,
        }
    }

    pub async fn get(pool: &SqlitePool) -> Result<Feed, Error> {
        debug!("get_feed");
        let feed_title = Param::get(pool, "feed_title").await?;
        let feed_subtitle = Param::get(pool, "feed_subtitle").await?;
        let feed_summary = Param::get(pool, "feed_summary").await?;
        let feed_link = Param::get(pool, "feed_link").await?;
        let feed_image_url = Param::get(pool, "feed_image_url").await?;
        let feed_category = Param::get(pool, "feed_category").await?;
        let feed_rating = Param::get(pool, "feed_rating").await?;
        let feed_description = Param::get(pool, "feed_description").await?;
        let feed_author = Param::get(pool, "feed_author").await?;
        let feed_explicit_str = Param::get(pool, "feed_explicit").await?;
        let feed_explicit = feed_explicit_str == "TRUE";
        let feed_keywords = Param::get(pool, "feed_keywords").await?;
        let feed_owner_name = Param::get(pool, "feed_owner_name").await?;
        let feed_owner_email = Param::get(pool, "feed_owner_email").await?;
        Ok(Feed::new(feed_title, feed_subtitle, feed_summary, feed_link,
            feed_image_url, feed_category, feed_rating, feed_description,
            feed_author, feed_explicit, feed_keywords, feed_owner_name,
            feed_owner_email))
    }

    pub async fn set(pool: &SqlitePool, feed: &Feed) -> Result<Feed, Error> {
        debug!("save_feed");
        Param::set(pool, "feed_title", &feed.title).await?;
        Param::set(pool, "feed_subtitle", &feed.subtitle).await?;
        Param::set(pool, "feed_summary", &feed.summary).await?;
        Param::set(pool, "feed_link", &feed.link).await?;
        Param::set(pool, "feed_image_url", &feed.image_url).await?;
        Param::set(pool, "feed_category", &feed.category).await?;
        Param::set(pool, "feed_rating", &feed.rating).await?;
        Param::set(pool, "feed_description", &feed.description).await?;
        Param::set(pool, "feed_author", &feed.author).await?;
        Param::set(pool, "feed_explicit", &feed.explicit.to_string().to_uppercase()).await?;
        Param::set(pool, "feed_keywords", &feed.keywords).await?;
        Param::set(pool, "feed_owner_name", &feed.owner_name).await?;
        Param::set(pool, "feed_owner_email", &feed.owner_email).await?;
        Self::get(pool).await
    }

    pub fn rss(&self, episodes: Vec<Item>) -> Result<String, Error>{
        let image = ImageBuilder::default()
            .url(&self.image_url)
            .title(self.title.clone())
            .link(&self.link)
            .build();
        let category = CategoryBuilder::default()
            .name(&self.category)
            .build();
        let itunes_category = ITunesCategoryBuilder::default()
            .text(&self.category)
            .build();
        let itunes_owner = ITunesOwnerBuilder::default()
            .name(self.owner_name.clone())
            .email(self.owner_email.clone())
            .build();
        let itunes = ITunesChannelExtensionBuilder::default()
            .subtitle(Some(self.subtitle.clone()))
            .summary(Some(self.summary.clone()))
            .category(itunes_category)
            .keywords(Some(self.keywords.clone()))
            .explicit(Some(if self.explicit { "yes".to_string() } else { "no".to_string() }))
            .author(Some(self.author.clone()))
            .owner(Some(itunes_owner))
            .build();
        let mut channel = ChannelBuilder::default()
            .title(&self.title)
            .link(&self.link)
            .image(Some(image))
            .category(category)
            .rating(Some(self.rating.clone()))
            .language(Some("es".to_string()))
            .generator(Some("Podmixer".to_string()))
            .description(self.description.clone())
            .build();
        channel.namespaces = BTreeMap::new();
        let mut namespaces = BTreeMap::new();
        namespaces.insert("content".to_string(), "http://purl.org/rss/1.0/modules/content/".to_string());
        namespaces.insert("wfw".to_string(), "http://wellformedweb.org/CommentAPI/".to_string());
        namespaces.insert("itunes".to_string(), "http://www.itunes.com/dtds/podcast-1.0.dtd".to_string());
        namespaces.insert("dc".to_string(), "http://purl.org/dc/elements/1.1/".to_string());
        namespaces.insert("media".to_string(), "http://search.yahoo.com/mrss/".to_string());
        namespaces.insert("atom".to_string(), "http://www.w3.org/2005/Atom".to_string());
        namespaces.insert("feedpress".to_string(), "https://feed.press/xmlns".to_string());
        namespaces.insert("podcast".to_string(), "https://podcastindex.org/namespace/1.0".to_string());
        namespaces.insert("googleplay".to_string(), "http://www.google.com/schemas/play-podcasts/1.0".to_string());
        namespaces.insert("podcast".to_string(), "https://podcastindex.org/namespace/1.0".to_string());
        channel.namespaces = namespaces;
        channel.set_itunes_ext(itunes);
        channel.set_items(episodes);
        channel.pretty_write_to(std::io::sink(), b' ', 4)?;
        Ok(channel.to_string())
    }
}

