use serde::{Serialize, Deserialize};
use chrono::{DateTime, Utc};
use sqlx::{sqlite::{SqlitePool, SqliteRow}, query, Row, Error};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct User{
    id: i64,
    pub name: String,
    pub password: String,
    pub active: bool,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TokenClaims {
    pub sub: String,
    pub iat: usize,
    pub exp: usize,
}

#[derive(Debug, Deserialize)]
pub struct UserSchema {
    pub name: String,
    pub password: String,
}

#[derive(Debug, Serialize)]
pub struct FilteredUser {
    pub id: i64,
    pub name: String,
    pub role: String,
    pub verified: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}


impl User{
    fn from_row(row: SqliteRow) -> Self{
        Self{
            id: row.get("id"),
            name: row.get("name"),
            password: row.get("password"),
            active: row.get("active"),
            created_at: row.get("created_at"),
            updated_at: row.get("updated_at"),
        }
    }

    pub async fn get_by_name(pool: &SqlitePool, name: &str) -> Result<User, Error>{
        let sql = "SELECT * FROM users WHERE name = $1";
        query(sql)
            .bind(name)
            .map(Self::from_row)
            .fetch_one(pool)
            .await
    }

}
