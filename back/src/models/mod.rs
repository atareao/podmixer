mod api_response;
mod config;
mod data;
mod feed;
mod id;
mod podcast;
pub mod publisher;
pub mod util;
mod user;

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

use sqlx::sqlite::SqlitePool;
use std::sync::Arc;
use std::collections::HashMap;
use std::time::Instant;
use std::sync::Mutex;

#[derive(Clone)]
pub struct AppState {
    pub pool: SqlitePool,
    pub secret: String,
    pub sse_broadcaster: SseBroadcaster,
    pub oauth_states: Arc<Mutex<HashMap<String, (String, Instant)>>>,
}
