use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Feed{
    pub link: String,
    pub title: String,
    pub author: String,
    pub image_url: String,
    pub category: String,
    pub rating: String,
    pub description: String,
    pub credir: String,
    pub keywords: String,
}

