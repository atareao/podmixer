use std::sync::Arc;

use axum::{
    extract::{Query, State},
    http::StatusCode,
    response::IntoResponse,
    routing, Json, Router,
};
use tracing::{debug, error};

use crate::models::{ApiResponse, AppState, Data, Podcast, NewPodcast, Id};

pub fn podcast_router() -> Router<Arc<AppState>> {
    Router::new()
        .route("/", routing::post(create))
        .route("/", routing::patch(update))
        .route("/", routing::get(read))
        .route("/", routing::delete(delete))
}

pub async fn create(
    State(app_state): State<Arc<AppState>>,
    Json(podcast): Json<NewPodcast>,
) -> impl IntoResponse {
    debug!("Podcast: {:?}", podcast);
    match Podcast::create(&app_state.pool, &podcast.name, &podcast.url, podcast.active, &podcast.last_pub_date).await {
        Ok(podcast) => {
            debug!("Podcast created: {:?}", podcast);
            ApiResponse::new(StatusCode::CREATED, "Podcast created", Data::One(serde_json::to_value(podcast).unwrap()))
        },
        Err(e) => {
            error!("Error creating podcast: {:?}", e);
            ApiResponse::new(StatusCode::BAD_REQUEST, "Error creating podcaste", Data::None)
        }
    }
}

pub async fn update(
    State(app_state): State<Arc<AppState>>,
    Json(podcast): Json<Podcast>,
) -> impl IntoResponse {
    debug!("Update podcast: {:?}", podcast);
    match Podcast::update(&app_state.pool, &podcast).await {
        Ok(podcast) => {
            debug!("Podcast updated: {:?}", podcast);
            ApiResponse::new(StatusCode::OK, "Podcast updated", Data::One(serde_json::to_value(podcast).unwrap()))
        },
        Err(e) => {
            error!("Error updating podcast: {:?}", e);
            ApiResponse::new(StatusCode::BAD_REQUEST, "Error updating podcast", Data::None)
        }
    }
}

pub async fn read(
    State(app_state): State<Arc<AppState>>,
) -> impl IntoResponse {
    match Podcast::get(&app_state.pool).await {
        Ok(podcasts) => {
            debug!("Podcasts: {:?}", podcasts);
            ApiResponse::new(StatusCode::OK, "Podcasts", Data::One(serde_json::to_value(podcasts).unwrap()))
        },
        Err(e) => {
            error!("Error reading podcasts: {:?}", e);
            ApiResponse::new(StatusCode::BAD_REQUEST, "Error reading podcasts", Data::None)
        }
    }
}

pub async fn delete(
    State(app_state): State<Arc<AppState>>,
    id: Query<Id>,
) -> impl IntoResponse {
    debug!("Podcast: {:?}", id);
    match Podcast::delete(&app_state.pool, id.id).await {
        Ok(podcast) => {
            debug!("Podcast deleted: {:?}", podcast);
            ApiResponse::new(StatusCode::OK, "Podcast deleted", Data::One(serde_json::to_value(podcast).unwrap()))
        },
        Err(e) => {
            error!("Error deleting podcast: {:?}", e);
            ApiResponse::new(StatusCode::BAD_REQUEST, "Error deleting podcast", Data::None)
        }
    }
}

