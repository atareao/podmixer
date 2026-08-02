use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct Id {
    pub id: i64,
}
