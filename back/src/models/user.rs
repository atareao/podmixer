use serde::{Serialize, Deserialize};
use chrono::{DateTime, Utc};
use sqlx::{sqlite::{SqlitePool, SqliteRow}, query, Row, Error};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct User{
    id: i64,
    pub username: String,
    pub email: String,
    pub hashed_password: String,
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
    pub email: String,
    pub password: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UserRegister {
    pub username: String,
    pub email: String,
    pub password: String,
}

#[derive(Debug, Serialize)]
pub struct FilteredUser {
    pub id: i64,
    pub username: String,
    pub role: String,
    pub verified: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}


impl User{
    fn from_row(row: SqliteRow) -> Self{
        Self{
            id: row.get("id"),
            username: row.get("username"),
            email: row.get("email"),
            hashed_password: row.get("hashed_password"),
            active: row.get("active"),
            created_at: row.get("created_at"),
            updated_at: row.get("updated_at"),
        }
    }

    pub async fn create(pool: &SqlitePool, username: &str, email: &str, password: &str) -> Result<User, Error> {
        let hashed_password = bcrypt::hash(password, bcrypt::DEFAULT_COST).unwrap();

        let sql = "INSERT INTO users (username, email, hashed_password) VALUES ($1, $2, $3) RETURNING *";
        query(sql)
            .bind(username)
            .bind(email)
            .bind(hashed_password)
            .map(Self::from_row)
            .fetch_one(pool)
            .await
    }

    pub async fn get_by_email(pool: &SqlitePool, email: &str) -> Result<User, Error>{
        let sql = "SELECT * FROM users WHERE email = $1";
        query(sql)
            .bind(email)
            .map(Self::from_row)
            .fetch_one(pool)
            .await
    }
}

