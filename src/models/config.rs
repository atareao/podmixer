use serde::{Serialize, Deserialize};
use sqlx::{sqlite::{SqlitePool, SqliteRow}, query, Row};
use chrono::{DateTime, Utc};
use tracing::{info, debug};
use std::collections::HashMap;
use super::{
    Error,
    Feed,
    Telegram,
    Twitter,
};


#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Param{
    id: i64,
    key: String,
    value: String,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}


impl Param{
    pub fn get_id(&self) -> i64{
        self.id
    }

    pub fn get_key(&self) -> &str{
        &self.key
    }

    pub async fn get_url(pool: &SqlitePool) -> String{
        Self::get(pool, "url")
            .await
            .unwrap()
    }

    pub async fn get_port(pool: &SqlitePool) -> u16{
        Self::get(pool, "port")
            .await
            .unwrap()
            .parse::<u16>()
            .unwrap()
    }

    pub async fn get_secret(pool: &SqlitePool) -> String{
        Self::get(pool, "jwt_secret")
            .await
            .unwrap()
    }

    pub async fn get_sleep_time(pool: &SqlitePool) -> u64{
        Self::get(pool, "sleep_time")
            .await
            .unwrap()
            .parse::<u64>()
            .unwrap()
    }

    pub async fn get_older_than(pool: &SqlitePool) -> i32{
        Self::get(pool, "older_than")
            .await
            .unwrap()
            .parse::<i32>()
            .unwrap()
    }

    pub async fn get_telegram(pool: &SqlitePool) -> Result<Telegram, Error> {
        let active: bool = Self::get(pool, "telegram_active")
            .await?
            .parse()?;
        let token = Self::get(pool, "telegram_token").await?;
        let chat_id: i64 = Self::get(pool, "telegram_chat_id")
            .await?
            .parse()?;
        let thread_id = Self::get(pool, "telegram_thread_id")
            .await?
            .parse()?;
        Ok(Telegram::new(active, token, chat_id, thread_id))
    }

    pub async fn get_feed(pool: &SqlitePool) -> Result<Feed, Error> {
        let feed_title = Self::get(pool, "feed_title").await?;
        let feed_link = Self::get(pool, "feed_link").await?;
        let feed_image_url = Self::get(pool, "feed_image_url").await?;
        let feed_category = Self::get(pool, "feed_category").await?;
        let feed_rating = Self::get(pool, "feed_rating").await?;
        let feed_description = Self::get(pool, "feed_description").await?;
        let feed_author = Self::get(pool, "feed_author").await?;
        let feed_explicit = Self::get(pool, "feed_explicit").await?;
        let feed_keywords = Self::get(pool, "feed_keywords").await?;
        Ok(Feed::new(feed_title, feed_link, feed_image_url, feed_category, feed_rating, feed_description, feed_author, feed_explicit, feed_keywords
        ))
    }

    pub async fn get_twitter(pool: &SqlitePool) -> Result<Twitter, Error> {
        let active: bool = Self::get(pool, "telegram_active")
            .await?
            .parse()?;
        let client_id = Self::get(pool, "client_id").await?;
        let client_secret = Self::get(pool, "client_secret").await?;
        let access_token = Self::get(pool, "access_token").await?;
        let refresh_token = Self::get(pool, "refresh_token").await?;
        Ok(Twitter::new(active, client_id, client_secret, access_token, refresh_token))
    }

    pub fn get_value(&self) -> &str{
        &self.value
    }

    pub fn get_created_at(&self) -> &DateTime<Utc>{
        &self.created_at
    }

    pub fn get_updated_at(&self) -> &DateTime<Utc>{
        &self.updated_at
    }

    fn from_row(row: SqliteRow) -> Self{
        Self{
            id: row.get("id"),
            key: row.get("key"),
            value: row.get("value"),
            created_at: row.get("created_at"),
            updated_at: row.get("updated_at"),
        }
    }

    pub async fn get(pool: &SqlitePool, key: &str) -> Result<String, Error>{
        debug!("get {key}");
        let sql = "SELECT value FROM config WHERE key = $1";
        query(sql)
            .bind(key)
            .map(|row: SqliteRow| -> String {row.get(0)})
            .fetch_one(pool)
            .await
            .map_err(|e| e.into())
    }

    pub async fn get_all(pool: &SqlitePool) -> Result<HashMap<String, String>, Error>{
        info!("get_all");
        let sql = "SELECT * FROM config";
        let params = query(sql)
            .map(Self::from_row)
            .fetch_all(pool)
            .await?;
        let mut kv = HashMap::new();
        for param in params{
            debug!("{:?}", param);
            kv.insert(param.key, param.value);
        }
        Ok(kv)
    }


    pub async fn exists(pool: &SqlitePool, key: &str) -> Result<bool, Error>{
        debug!("exists {key}");
        let sql = "SELECT count(key) FROM config WHERE key = $1";
        Ok(query(sql)
            .bind(key)
            .map(|row: SqliteRow| -> i64 {row.get(0)})
            .fetch_one(pool)
            .await? > 0)
    }

    pub async fn set(pool: &SqlitePool, key: &str, value: &str) -> Result<Param, Error>{
        debug!("set {key}={value}");
        let current_ts = Utc::now();
        let sql = "INSERT INTO config(key, value, updated_at) \
            VALUES($1, $2, $3)
            ON CONFLICT(key) DO UPDATE SET
            value=excluded.value,
            updated_at=excluded.updated_at
            RETURNING *";
        query(sql)
            .bind(key)
            .bind(value)
            .bind(current_ts)
            .map(Self::from_row)
            .fetch_one(pool)
            .await
            .map_err(|e| e.into())
    }
}

