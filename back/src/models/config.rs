use super::Error;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::{
    query,
    sqlite::{SqlitePool, SqliteRow},
    Row,
};
use tracing::debug;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Param {
    id: i64,
    key: String,
    value: String,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

impl Param {
    fn from_row(row: SqliteRow) -> Self {
        Self {
            id: row.get("id"),
            key: row.get("key"),
            value: row.get("value"),
            created_at: row.get("created_at"),
            updated_at: row.get("updated_at"),
        }
    }

    pub async fn get(pool: &SqlitePool, key: &str) -> Result<String, Error> {
        debug!("get {key}");
        let sql = "SELECT value FROM config WHERE key = $1";
        query(sql)
            .bind(key)
            .map(|row: SqliteRow| -> String { row.get(0) })
            .fetch_one(pool)
            .await
            .map_err(|e| e.into())
    }

    pub async fn set(pool: &SqlitePool, key: &str, value: &str) -> Result<Param, Error> {
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
