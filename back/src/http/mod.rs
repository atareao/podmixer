mod config;
mod health;
mod podcast;
mod publishers;
mod user;

pub use config::config_router;
pub use health::health_router;
pub use podcast::podcast_router;
pub use publishers::publishers_router;
pub use user::user_router;
