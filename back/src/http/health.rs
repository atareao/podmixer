use crate::models::{ApiResponse, AppState, Data};
use axum::{http::StatusCode, response::IntoResponse, routing, Router};
use std::sync::Arc;
use tracing::info;

pub fn health_router() -> Router<Arc<AppState>> {
    Router::new().route("/", routing::get(check_health))
}

async fn check_health() -> impl IntoResponse {
    info!("Health check.");
    ApiResponse::create(StatusCode::OK, "Up and running", Data::None)
}
