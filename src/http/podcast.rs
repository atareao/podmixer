use serde::Deserialize;
use std::sync::Arc;
use serde_json::json;

use axum::{
    extract::{
        State,
        Query,
    },
    Router,
    routing,
    response::IntoResponse,
    Json,
    middleware::from_fn_with_state
};

#[derive(Deserialize)]
struct Params {
    id: i64,
}

use crate::{
    models::{
        Podcast,
        FormPodcast,
    },
    http::{
        AppState,
        jwt_auth::auth
    },
};
use tracing::debug;

pub fn router(app_state: Arc<AppState>) -> Router<Arc<AppState>> {
    Router::new()
        .route("/podcasts",
            routing::get(get_podcasts)
            .route_layer(from_fn_with_state(app_state.clone(), auth))
        )
        .route("/podcasts",
            routing::post(create)
            .route_layer(from_fn_with_state(app_state.clone(), auth))
        )
        .route("/podcasts",
            routing::delete(delete)
            .route_layer(from_fn_with_state(app_state.clone(), auth))
        )
}

pub async fn get_podcasts(
    State(app_state): State<Arc<AppState>>,
) -> impl IntoResponse{
    Podcast::get(&app_state.pool)
        .await
        .map(|podcasts| Json(json!({
            "result": "ok",
            "content": podcasts
        })))
        .map_err(|e| Json(json!({
            "result": "ko",
            "content": e.to_string(),
        })))
}

async fn create(
    State(app_state): State<Arc<AppState>>,
    Json(podcast): Json<FormPodcast>,
) -> impl IntoResponse{
    Podcast::create(&app_state.pool, &podcast.name, &podcast.url)
        .await
        .map(|podcasts| Json(json!({
            "result": "ok",
            "content": podcasts
        })))
        .map_err(|e| Json(json!({
            "result": "ko",
            "content": e.to_string(),

        })))
}
async fn delete(
    State(app_state): State<Arc<AppState>>,
    Query(params): Query<Params>,
) -> impl IntoResponse{
    debug!("podcast: {}", params.id);
    Podcast::delete(&app_state.pool, params.id)
        .await
        .map(|podcasts| Json(json!({
            "result": "ok",
            "content": podcasts
        })))
        .map_err(|e| Json(json!({
            "result": "ko",
            "content": e.to_string(),

        })))
}
