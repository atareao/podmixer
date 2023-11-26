use serde::{Deserialize, Serialize};
use sqlx::{sqlite::{SqlitePool, SqliteRow}, query, Row};
use rss::{Channel, Item};
use super::Error;
use chrono::{NaiveDateTime, DateTime, Utc, Duration};

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct FormPodcast{
    pub name: String,
    pub url: String,
}
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Podcast{
    pub id: i64,
    pub name: String,
    pub url: String,
    pub active: bool,
    pub last_pub_date: NaiveDateTime,
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
        let sql = "SELECT * FROM podcasts";
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

    pub async fn create(pool: &SqlitePool, name: &str, url: &str) -> Result<Podcast, sqlx::error::Error>{
        let sql = "INSERT INTO podcasts (name, url) VALUES ($1, $2) RETURNING *";
        query(sql)
            .bind(name)
            .bind(url)
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
    pub fn get_all(&self) -> Vec<Item>{
        let mut all: Vec<Item> = Vec::new();
        for item in self.channel.items.as_slice(){
            all.push(item.clone());
        }
        all
    }

    pub fn get_new(&self) -> Result<Vec<Item>, Error>{
        Ok(self.get_older_than(&self.podcast.last_pub_date)?)
    }

    pub fn get_older_than_days(&self, days: i32) -> Result<Vec<Item>, Error>{
        let datetime = (Utc::now() - Duration::days(days.into())).naive_local();
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
