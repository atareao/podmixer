use serde::{Serialize, Deserialize};

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Telegram{
    token: String,
    group_id: String,
    topic_id: String,
}
