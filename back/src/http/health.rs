use axum::{routing,  Router, response::IntoResponse, http::StatusCode};
use tracing::info;
use crate::models::{Data, ApiResponse, AppState};
use std::sync::Arc;


pub fn health_router() -> Router<Arc<AppState>> {
    Router::new()
        .route("/", routing::get(check_health))
}

async fn check_health() -> impl IntoResponse {
    info!("Health check.");
    ApiResponse::create(StatusCode::OK, "Up and running", Data::None)
}


