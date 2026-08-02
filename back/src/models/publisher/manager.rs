use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use sqlx::sqlite::SqlitePool;

use super::super::Error;
use super::types::{Publisher, PublisherImpl, PublisherType, PublishLog};

use super::telegram::TelegramPublisher;
use super::x::XPublisher;
use super::mastodon::MastodonPublisher;
use super::matrix::MatrixPublisher;

pub struct PublisherManager {
    pool: SqlitePool,
    publishers: RwLock<HashMap<String, Arc<dyn PublisherImpl>>>,
}

impl PublisherManager {
    pub fn new(pool: SqlitePool) -> Self {
        Self {
            pool,
            publishers: RwLock::new(HashMap::new()),
        }
    }

    pub async fn load_publishers(&self) -> Result<(), Error> {
        let rows = sqlx::query_as::<_, (String, String, String, String, String, bool, String, String)>(
            "SELECT id, name, publisher_type, config, template, active, created_at, updated_at FROM publishers WHERE active = 1"
        )
        .fetch_all(&self.pool)
        .await?;

        let mut publishers = self.publishers.write().await;
        publishers.clear();

        for (id, name, ptype, config_json, template, active, created_at, updated_at) in rows {
            let ptype = PublisherType::from_str(&ptype);
            if ptype.is_none() { continue; }
            let config: serde_json::Value = serde_json::from_str(&config_json).unwrap_or_default();
            let _publisher = Publisher {
                id, name,
                publisher_type: ptype.unwrap(),
                config, template,
                active, created_at, updated_at,
            };
        }
        Ok(())
    }

    pub async fn get_publishers_db(&self) -> Result<Vec<Publisher>, Error> {
        let rows = sqlx::query_as::<_, (String, String, String, String, String, bool, String, String)>(
            "SELECT id, name, publisher_type, config, template, active, created_at, updated_at FROM publishers ORDER BY created_at DESC"
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(rows.into_iter().map(|(id, name, ptype, config, template, active, created_at, updated_at)| {
            Publisher {
                id, name,
                publisher_type: PublisherType::from_str(&ptype).unwrap_or(PublisherType::Telegram),
                config: serde_json::from_str(&config).unwrap_or_default(),
                template, active, created_at, updated_at,
            }
        }).collect())
    }

    pub async fn create_publisher(&self, publisher: &Publisher) -> Result<Publisher, Error> {
        let id = uuid::Uuid::new_v4().to_string();
        let config_str = serde_json::to_string(&publisher.config)?;
        sqlx::query(
            "INSERT INTO publishers (id, name, publisher_type, config, template, active) VALUES (?, ?, ?, ?, ?, ?)"
        )
        .bind(&id)
        .bind(&publisher.name)
        .bind(publisher.publisher_type.as_str())
        .bind(&config_str)
        .bind(&publisher.template)
        .bind(publisher.active)
        .execute(&self.pool)
        .await?;

        self.get_publisher(&id).await
    }

    pub async fn get_publisher(&self, id: &str) -> Result<Publisher, Error> {
        let row = sqlx::query_as::<_, (String, String, String, String, String, bool, String, String)>(
            "SELECT id, name, publisher_type, config, template, active, created_at, updated_at FROM publishers WHERE id = ?"
        )
        .bind(id)
        .fetch_one(&self.pool)
        .await?;

        Ok(Publisher {
            id: row.0, name: row.1,
            publisher_type: PublisherType::from_str(&row.2).unwrap_or(PublisherType::Telegram),
            config: serde_json::from_str(&row.3).unwrap_or_default(),
            template: row.4, active: row.5,
            created_at: row.6, updated_at: row.7,
        })
    }

    pub async fn update_publisher(&self, id: &str, publisher: &Publisher) -> Result<Publisher, Error> {
        let config_str = serde_json::to_string(&publisher.config)?;
        sqlx::query(
            "UPDATE publishers SET name = ?, publisher_type = ?, config = ?, template = ?, active = ?, updated_at = datetime('now') WHERE id = ?"
        )
        .bind(&publisher.name)
        .bind(publisher.publisher_type.as_str())
        .bind(&config_str)
        .bind(&publisher.template)
        .bind(publisher.active)
        .bind(id)
        .execute(&self.pool)
        .await?;

        self.get_publisher(id).await
    }

    pub async fn delete_publisher(&self, id: &str) -> Result<(), Error> {
        sqlx::query("DELETE FROM publishers WHERE id = ?")
            .bind(id)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    pub async fn toggle_publisher(&self, id: &str) -> Result<Publisher, Error> {
        sqlx::query("UPDATE publishers SET active = NOT active, updated_at = datetime('now') WHERE id = ?")
            .bind(id)
            .execute(&self.pool)
            .await?;
        self.get_publisher(id).await
    }

    pub async fn add_log(&self, log: &PublishLog) -> Result<(), Error> {
        sqlx::query(
            "INSERT INTO publish_logs (id, publisher_id, publisher_name, publisher_type, episode_title, status, message) VALUES (?, ?, ?, ?, ?, ?, ?)"
        )
        .bind(&log.id)
        .bind(&log.publisher_id)
        .bind(&log.publisher_name)
        .bind(&log.publisher_type)
        .bind(&log.episode_title)
        .bind(&log.status)
        .bind(&log.message)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    pub async fn get_logs(&self, limit: i64, offset: i64) -> Result<Vec<PublishLog>, Error> {
        let rows = sqlx::query_as::<_, (String, String, String, String, String, String, String, String)>(
            "SELECT id, publisher_id, publisher_name, publisher_type, episode_title, status, message, created_at FROM publish_logs ORDER BY created_at DESC LIMIT ? OFFSET ?"
        )
        .bind(limit)
        .bind(offset)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows.into_iter().map(|r| PublishLog {
            id: r.0, publisher_id: r.1, publisher_name: r.2,
            publisher_type: r.3, episode_title: r.4,
            status: r.5, message: r.6, created_at: r.7,
        }).collect())
    }

    pub async fn get_logs_by_publisher(&self, publisher_id: &str, limit: i64) -> Result<Vec<PublishLog>, Error> {
        let rows = sqlx::query_as::<_, (String, String, String, String, String, String, String, String)>(
            "SELECT id, publisher_id, publisher_name, publisher_type, episode_title, status, message, created_at FROM publish_logs WHERE publisher_id = ? ORDER BY created_at DESC LIMIT ?"
        )
        .bind(publisher_id)
        .bind(limit)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows.into_iter().map(|r| PublishLog {
            id: r.0, publisher_id: r.1, publisher_name: r.2,
            publisher_type: r.3, episode_title: r.4,
            status: r.5, message: r.6, created_at: r.7,
        }).collect())
    }
}

pub fn create_publisher_impl(
    id: &str,
    publisher_type: &PublisherType,
    config: &serde_json::Value,
) -> Option<Arc<dyn PublisherImpl>> {
    match publisher_type {
        PublisherType::Telegram => {
            TelegramPublisher::new(id.to_string(), config)
                .map(|p| Arc::new(p) as Arc<dyn PublisherImpl>)
        }
        PublisherType::X => {
            XPublisher::new(id.to_string(), config)
                .map(|p| Arc::new(p) as Arc<dyn PublisherImpl>)
        }
        PublisherType::Mastodon => {
            MastodonPublisher::new(id.to_string(), config)
                .map(|p| Arc::new(p) as Arc<dyn PublisherImpl>)
        }
        PublisherType::Matrix => {
            MatrixPublisher::new(id.to_string(), config)
                .map(|p| Arc::new(p) as Arc<dyn PublisherImpl>)
        }
    }
}