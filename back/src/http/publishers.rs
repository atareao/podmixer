use std::convert::Infallible;
use std::sync::Arc;

use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::{
        sse::{Event, Sse},
        IntoResponse,
    },
    routing, Json, Router,
};
use futures::stream::{Stream, StreamExt};
use serde::Deserialize;
use tokio_stream::wrappers::BroadcastStream;
use tracing::error;

use crate::models::{
    publisher::{
        manager::{create_publisher_impl, PublisherManager},
        template::TemplateContext,
        types::PublishLog,
    },
    ApiResponse, AppState, Data,
};

#[derive(Deserialize)]
pub struct LogsQuery {
    limit: Option<i64>,
    offset: Option<i64>,
}

pub fn publishers_router() -> Router<Arc<AppState>> {
    Router::new()
        .route("/", routing::get(list_publishers))
        .route("/", routing::post(create_publisher))
        .route("/{id}", routing::get(get_publisher))
        .route("/{id}", routing::patch(update_publisher))
        .route("/{id}", routing::delete(delete_publisher))
        .route("/{id}/toggle", routing::post(toggle_publisher))
        .route("/{id}/test", routing::post(test_publisher))
        .route("/logs", routing::get(get_logs))
        .route("/logs/stream", routing::get(stream_logs))
}

pub async fn list_publishers(State(app_state): State<Arc<AppState>>) -> impl IntoResponse {
    let manager = PublisherManager::new(app_state.pool.clone());
    match manager.get_publishers_db().await {
        Ok(publishers) => ApiResponse::new(
            StatusCode::OK,
            "Publishers list",
            Data::One(serde_json::to_value(publishers).unwrap()),
        ),
        Err(e) => {
            error!("Error listing publishers: {:?}", e);
            ApiResponse::new(
                StatusCode::INTERNAL_SERVER_ERROR,
                "Error listing publishers",
                Data::None,
            )
        }
    }
}

pub async fn create_publisher(
    State(app_state): State<Arc<AppState>>,
    Json(publisher): Json<crate::models::Publisher>,
) -> impl IntoResponse {
    let manager = PublisherManager::new(app_state.pool.clone());
    match manager.create_publisher(&publisher).await {
        Ok(p) => ApiResponse::new(
            StatusCode::CREATED,
            "Publisher created",
            Data::One(serde_json::to_value(p).unwrap()),
        ),
        Err(e) => {
            error!("Error creating publisher: {:?}", e);
            ApiResponse::new(
                StatusCode::BAD_REQUEST,
                "Error creating publisher",
                Data::None,
            )
        }
    }
}

pub async fn get_publisher(
    State(app_state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    let manager = PublisherManager::new(app_state.pool.clone());
    match manager.get_publisher(&id).await {
        Ok(p) => ApiResponse::new(
            StatusCode::OK,
            "Publisher found",
            Data::One(serde_json::to_value(p).unwrap()),
        ),
        Err(e) => {
            error!("Error getting publisher: {:?}", e);
            ApiResponse::new(StatusCode::NOT_FOUND, "Publisher not found", Data::None)
        }
    }
}

pub async fn update_publisher(
    State(app_state): State<Arc<AppState>>,
    Path(id): Path<String>,
    Json(publisher): Json<crate::models::Publisher>,
) -> impl IntoResponse {
    let manager = PublisherManager::new(app_state.pool.clone());
    match manager.update_publisher(&id, &publisher).await {
        Ok(p) => ApiResponse::new(
            StatusCode::OK,
            "Publisher updated",
            Data::One(serde_json::to_value(p).unwrap()),
        ),
        Err(e) => {
            error!("Error updating publisher: {:?}", e);
            ApiResponse::new(
                StatusCode::BAD_REQUEST,
                "Error updating publisher",
                Data::None,
            )
        }
    }
}

pub async fn delete_publisher(
    State(app_state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    let manager = PublisherManager::new(app_state.pool.clone());
    match manager.delete_publisher(&id).await {
        Ok(_) => ApiResponse::new(StatusCode::OK, "Publisher deleted", Data::None),
        Err(e) => {
            error!("Error deleting publisher: {:?}", e);
            ApiResponse::new(
                StatusCode::BAD_REQUEST,
                "Error deleting publisher",
                Data::None,
            )
        }
    }
}

pub async fn toggle_publisher(
    State(app_state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    let manager = PublisherManager::new(app_state.pool.clone());
    match manager.toggle_publisher(&id).await {
        Ok(p) => ApiResponse::new(
            StatusCode::OK,
            "Publisher toggled",
            Data::One(serde_json::to_value(p).unwrap()),
        ),
        Err(e) => {
            error!("Error toggling publisher: {:?}", e);
            ApiResponse::new(
                StatusCode::BAD_REQUEST,
                "Error toggling publisher",
                Data::None,
            )
        }
    }
}

pub async fn test_publisher(
    State(app_state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    let manager = PublisherManager::new(app_state.pool.clone());

    let publisher = match manager.get_publisher(&id).await {
        Ok(p) => p,
        Err(e) => {
            error!("Publisher not found: {:?}", e);
            return ApiResponse::new(StatusCode::NOT_FOUND, "Publisher not found", Data::None);
        }
    };

    let ptype = publisher.publisher_type.clone();
    let impl_instance = match create_publisher_impl(&id, &ptype, &publisher.config) {
        Some(instance) => instance,
        None => {
            return ApiResponse::new(
                StatusCode::BAD_REQUEST,
                "Invalid publisher config",
                Data::None,
            );
        }
    };

    let ctx = TemplateContext {
        title: "Test publication".to_string(),
        description: "This is a test message from Podmixer".to_string(),
        url: "https://podmixer.example.com".to_string(),
    };

    let log_id = uuid::Uuid::new_v4().to_string();
    let log = PublishLog {
        id: log_id.clone(),
        publisher_id: publisher.id.clone(),
        publisher_name: publisher.name.clone(),
        publisher_type: publisher.publisher_type.as_str().to_string(),
        episode_title: "Test".to_string(),
        status: "sending".to_string(),
        message: String::new(),
        created_at: String::new(),
    };
    let _ = manager.add_log(&log).await;

    let (log, status_code, message, data) = {
        let publish_result = impl_instance
            .publish(&ctx.title, &ctx.description, &ctx.url)
            .await;
        match publish_result {
            Ok(response) => {
                let log = PublishLog {
                    id: log_id,
                    publisher_id: publisher.id,
                    publisher_name: publisher.name,
                    publisher_type: publisher.publisher_type.as_str().to_string(),
                    episode_title: "Test".to_string(),
                    status: "success".to_string(),
                    message: "Test published successfully".to_string(),
                    created_at: String::new(),
                };
                let data = Data::One(serde_json::json!({"response": response}));
                (log, StatusCode::OK, "Test published".to_string(), data)
            }
            Err(e) => {
                let msg = e.to_string();
                let log = PublishLog {
                    id: log_id,
                    publisher_id: publisher.id,
                    publisher_name: publisher.name,
                    publisher_type: publisher.publisher_type.as_str().to_string(),
                    episode_title: "Test".to_string(),
                    status: "error".to_string(),
                    message: msg.clone(),
                    created_at: String::new(),
                };
                let data = Data::None;
                (log, StatusCode::INTERNAL_SERVER_ERROR, msg, data)
            }
        }
    };

    let _ = manager.add_log(&log).await;
    app_state.sse_broadcaster.broadcast(&log);
    ApiResponse::new(status_code, &message, data)
}

pub async fn get_logs(
    State(app_state): State<Arc<AppState>>,
    Query(query): Query<LogsQuery>,
) -> impl IntoResponse {
    let manager = PublisherManager::new(app_state.pool.clone());
    let limit = query.limit.unwrap_or(50);
    let offset = query.offset.unwrap_or(0);
    match manager.get_logs(limit, offset).await {
        Ok(logs) => ApiResponse::new(
            StatusCode::OK,
            "Logs retrieved",
            Data::One(serde_json::to_value(logs).unwrap()),
        ),
        Err(e) => {
            error!("Error getting logs: {:?}", e);
            ApiResponse::new(
                StatusCode::INTERNAL_SERVER_ERROR,
                "Error getting logs",
                Data::None,
            )
        }
    }
}

pub async fn stream_logs(
    State(app_state): State<Arc<AppState>>,
) -> Sse<impl Stream<Item = Result<Event, Infallible>>> {
    let rx = app_state.sse_broadcaster.subscribe();
    let stream = BroadcastStream::new(rx)
        .filter(|result| futures::future::ready(result.is_ok()))
        .map(|result| Ok(Event::default().data(result.unwrap())));
    Sse::new(stream).keep_alive(axum::response::sse::KeepAlive::new())
}