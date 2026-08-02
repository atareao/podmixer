mod user;
mod api_response;
mod data;
mod feed;
mod id;
mod podcast;
mod config;
mod telegram;
mod twitter;
pub mod util;

pub use data::Data;
pub use id::Id;
pub use api_response::ApiResponse;
pub use user::{User, TokenClaims, UserSchema, UserRegister};
pub type Error = Box<dyn std::error::Error>;
pub use podcast::{NewPodcast, Podcast, CompletePodcast};
pub use config::Param;
pub use feed::Feed;
pub use telegram::Telegram;
pub use twitter::Twitter;

use sqlx::sqlite::SqlitePool;

#[derive(Clone)]
pub struct AppState {
    pub pool: SqlitePool,
    pub secret: String,
}

