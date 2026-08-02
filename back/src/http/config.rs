use std::sync::Arc;

use crate::models::{ApiResponse, AppState, Data, Feed};
use axum::{extract::State, http::StatusCode, response::IntoResponse, routing, Json, Router};
use tracing::{debug, error};

pub fn config_router() -> Router<Arc<AppState>> {
    Router::new()
        .route("/feed", routing::get(read_feed))
        .route("/feed", routing::post(save_feed))
}

pub async fn read_feed(State(app_state): State<Arc<AppState>>) -> impl IntoResponse {
    match Feed::get(&app_state.pool).await {
        Ok(feed) => {
            debug!("{:?}", feed);
            ApiResponse::new(
                StatusCode::OK,
                "Feed read",
                Data::One(serde_json::to_value(feed).unwrap()),
            )
        }
        Err(e) => {
            error!("Error reading feed: {:?}", e);
            ApiResponse::new(StatusCode::BAD_REQUEST, "Error reading feed", Data::None)
        }
    }
}

pub async fn save_feed(
    State(app_state): State<Arc<AppState>>,
    Json(feed): Json<Feed>,
) -> impl IntoResponse {
    debug!("{:?}", feed);
    match Feed::set(&app_state.pool, &feed).await {
        Ok(feed) => {
            debug!("{:?}", feed);
            ApiResponse::new(
                StatusCode::OK,
                "Feed saved",
                Data::One(serde_json::to_value(feed).unwrap()),
            )
        }
        Err(e) => {
            error!("Error reading feed: {:?}", e);
            ApiResponse::new(StatusCode::BAD_REQUEST, "Error saving feed", Data::None)
        }
    }
}


