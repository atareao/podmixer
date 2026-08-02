mod user;
mod health;
mod podcast;
mod config;

pub use health::health_router;
pub use user::user_router;
pub use podcast::podcast_router;
pub use config::config_router;

