use std::sync::Arc;

use axum::{
    extract::State,
    http::StatusCode,
    response::IntoResponse,
    routing, Json, Router,
};
use tracing::{debug, error};
use crate::models::{ApiResponse, AppState, Data, Feed, Twitter, Telegram};

pub fn config_router() -> Router<Arc<AppState>> {
    Router::new()
        .route("/feed", routing::get(read_feed))
        .route("/feed", routing::post(save_feed))
        .route("/twitter", routing::get(read_twitter))
        .route("/twitter", routing::post(save_twitter))
        .route("/telegram", routing::get(read_telegram))
        .route("/telegram", routing::post(save_telegram))
}

pub async fn read_feed(
    State(app_state): State<Arc<AppState>>,
) -> impl IntoResponse{
    match Feed::get(&app_state.pool).await {
        Ok(feed) => {
            debug!("{:?}", feed);
            ApiResponse::new(StatusCode::OK, "Feed read", Data::One(serde_json::to_value(feed).unwrap()))
        },
        Err(e) => {
            error!("Error reading feed: {:?}", e);
            ApiResponse::new(StatusCode::BAD_REQUEST, "Error reading feed", Data::None)
        }
    }
}

pub async fn save_feed(
    State(app_state): State<Arc<AppState>>,
    Json(feed): Json<Feed>
) -> impl IntoResponse{
    debug!("{:?}", feed);
    match Feed::set(&app_state.pool, &feed).await {
        Ok(feed) => {
            debug!("{:?}", feed);
            ApiResponse::new(StatusCode::OK, "Feed saved", Data::One(serde_json::to_value(feed).unwrap()))
        },
        Err(e) => {
            error!("Error reading feed: {:?}", e);
            ApiResponse::new(StatusCode::BAD_REQUEST, "Error saving feed", Data::None)
        }
    }
}

pub async fn read_twitter(
    State(app_state): State<Arc<AppState>>,
) -> impl IntoResponse{
    match Twitter::get(&app_state.pool).await {
        Ok(feed) => {
            debug!("{:?}", feed);
            ApiResponse::new(StatusCode::OK, "Twitter read", Data::One(serde_json::to_value(feed).unwrap()))
        },
        Err(e) => {
            error!("Error reading feed: {:?}", e);
            ApiResponse::new(StatusCode::BAD_REQUEST, "Error reading twitter", Data::None)
        }
    }
}

pub async fn save_twitter(
    State(app_state): State<Arc<AppState>>,
    Json(twitter): Json<Twitter>
) -> impl IntoResponse{
    match Twitter::set(&app_state.pool, &twitter).await {
        Ok(twitter) => {
            debug!("{:?}", twitter);
            ApiResponse::new(StatusCode::OK, "Twitter saved", Data::One(serde_json::to_value(twitter).unwrap()))
        },
        Err(e) => {
            error!("Error reading twitter: {:?}", e);
            ApiResponse::new(StatusCode::BAD_REQUEST, "Error saving twitter", Data::None)
        }
    }
}

pub async fn read_telegram(
    State(app_state): State<Arc<AppState>>,
) -> impl IntoResponse{
    match Telegram::get(&app_state.pool).await {
        Ok(telegram) => {
            debug!("{:?}", telegram);
            ApiResponse::new(StatusCode::OK, "Telegram read", Data::One(serde_json::to_value(telegram).unwrap()))
        },
        Err(e) => {
            error!("Error reading feed: {:?}", e);
            ApiResponse::new(StatusCode::BAD_REQUEST, "Error reading telegram", Data::None)
        }
    }
}

pub async fn save_telegram(
    State(app_state): State<Arc<AppState>>,
    Json(telegram): Json<Telegram>
) -> impl IntoResponse{
    debug!("{:?}", telegram);
    match Telegram::set(&app_state.pool, &telegram).await {
        Ok(telegram) => {
            debug!("{:?}", telegram);
            ApiResponse::new(StatusCode::OK, "Telegram saved", Data::One(serde_json::to_value(telegram).unwrap()))
        },
        Err(e) => {
            error!("Error reading twitter: {:?}", e);
            ApiResponse::new(StatusCode::BAD_REQUEST, "Error saving telegram", Data::None)
        }
    }
}
