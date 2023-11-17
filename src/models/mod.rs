mod config;
mod feed;
mod podcast;
mod telegram;
mod twitter;

pub use config::Configuration;
pub use feed::Feed;
pub use podcast::Podcast;
pub use telegram::Telegram;
pub use twitter::Twitter;

pub type Error = Box<dyn std::error::Error>;
