mod api_response;
mod config;
mod data;
mod feed;
mod id;
mod podcast;
mod telegram;
mod twitter;
mod user;
pub mod publisher;
pub mod util;

pub use api_response::ApiResponse;
pub use data::Data;
pub use id::Id;
pub use user::{TokenClaims, User, UserRegister, UserSchema};
pub type Error = Box<dyn std::error::Error + Send + Sync>;
pub use config::Param;
pub use feed::Feed;
pub use podcast::{CompletePodcast, NewPodcast, Podcast};
pub use publisher::sse::SseBroadcaster;
pub use publisher::types::Publisher;
pub use telegram::Telegram;
pub use twitter::Twitter;

use sqlx::sqlite::SqlitePool;

#[derive(Clone)]
pub struct AppState {
    pub pool: SqlitePool,
    pub secret: String,
    pub sse_broadcaster: SseBroadcaster,
}
