use serde::{Deserialize, Serialize};
use sqlx::{sqlite::{SqlitePool, SqliteRow}, query, Row};
use rss::{Channel, Item};
use super::Error;
use chrono::{DateTime, Utc, Duration};
use tracing::debug;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct NewPodcast{
    pub name: String,
    pub url: String,
    pub active: bool,
    pub last_pub_date: DateTime<Utc>,
}
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Podcast{
    pub id: i64,
    pub name: String,
    pub url: String,
    pub active: bool,
    pub last_pub_date: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug)]
pub struct CompletePodcast{
    pub podcast: Podcast,
    pub channel: Channel,
}

impl Podcast{
    fn from_row(row: SqliteRow) -> Self{
        Self{
            id: row.get("id"),
            name: row.get("name"),
            url: row.get("url"),
            active: row.get("active"),
            last_pub_date: row.get("last_pub_date"),
            created_at: row.get("created_at"),
            updated_at: row.get("updated_at"),
        }
    }
    pub async fn get(pool: &SqlitePool) -> Result<Vec<Podcast>, sqlx::error::Error>{
        let sql = "SELECT * FROM podcasts ORDER BY last_pub_date DESC";
        query(sql)
            .map(Self::from_row)
            .fetch_all(pool)
            .await
    }

    pub async fn update(pool: &SqlitePool, podcast: &Podcast) -> Result<Podcast, sqlx::error::Error>{
        let sql = "UPDATE podcasts SET name=$1, url=$2, active=$3,
                   last_pub_date=$4, updated_at=$5 WHERE id=$6 RETURNING *";
        query(sql)
            .bind(podcast.name.to_owned())
            .bind(podcast.url.to_owned())
            .bind(podcast.active)
            .bind(podcast.last_pub_date)
            .bind(Utc::now())
            .bind(podcast.id)
            .map(Self::from_row)
            .fetch_one(pool)
            .await
    }

    #[allow(unused)]
    pub async fn create_or_update(pool: &SqlitePool, name: &str, url: &str, active: bool) -> Result<Podcast, sqlx::error::Error>{
        let current_ts = Utc::now();
        let sql = "INSERT INTO podcasts (name, url, active, updated_at) 
            VALUES ($1, $2, $3, $4)
            ON CONFLICT(name) DO UPDATE SET
            url=excluded.url,
            active=excluded.active,
            updated_at=excluded.updated_at
            RETURNING *";
        query(sql)
            .bind(name)
            .bind(url)
            .bind(active)
            .bind(current_ts)
            .map(Self::from_row)
            .fetch_one(pool)
            .await
    }

    pub async fn create(pool: &SqlitePool, name: &str, url: &str, active: bool, last_pub_date: &DateTime<Utc>) -> Result<Podcast, sqlx::error::Error>{
        let sql = "INSERT INTO podcasts (name, url, active, last_pub_date) VALUES ($1, $2, $3, $4) RETURNING *";
        query(sql)
            .bind(name)
            .bind(url)
            .bind(active)
            .bind(last_pub_date)
            .map(Self::from_row)
            .fetch_one(pool)
            .await
    }

    pub async fn delete(pool: &SqlitePool, id: i64) -> Result<Podcast, sqlx::error::Error>{
        let sql = "DELETE FROM podcasts WHERE id = $1 RETURNING *";
        query(sql)
            .bind(id)
            .map(Self::from_row)
            .fetch_one(pool)
            .await
    }

}

impl CompletePodcast {
    pub async fn new(podcast: &Podcast) -> Result<Self, Error>{
        debug!("Url: {}", &podcast.url);
        let content = reqwest::Client::builder()
            .user_agent("Mozilla/5.0 (Macintosh; Intel Mac OS X 10_10_1) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/39.0.2171.95 Safari/537.36")
            .build()?
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
    pub fn get_all(&self) -> Vec<Item>{
        let mut all: Vec<Item> = Vec::new();
        for item in self.channel.items.as_slice(){
            all.push(item.clone());
        }
        all
    }

    pub fn get_new(&self) -> Result<Vec<Item>, Error>{
        self.get_older_than(&self.podcast.last_pub_date)
    }

    pub fn get_older_than_days(&self, days: i32) -> Result<Vec<Item>, Error>{
        let datetime = Utc::now() - Duration::days(days.into());
        self.get_older_than(&datetime)
    }

    pub fn get_older_than(&self, datetime: &DateTime<Utc>) -> Result<Vec<Item>, Error>{
        debug!("get_older_than: {datetime}");
        let mut older_than: Vec<Item> = Vec::new();
        for item in self.channel.items.as_slice(){
            if let Some(pub_date_str) = item.pub_date(){
                if let Ok(pub_date) = DateTime::parse_from_rfc2822(pub_date_str){
                    if pub_date.timestamp() > datetime.timestamp(){
                        older_than.push(item.clone());
                    }
                }else if let Ok(pub_date) = DateTime::parse_from_str(pub_date_str, "%a, %d %b %Y %H:%M:%S") {
                    if pub_date.timestamp() > datetime.timestamp(){
                        older_than.push(item.clone());
                    }
                }
            }
        }
        Ok(older_than)
    }
}

