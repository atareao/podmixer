use std::convert::Infallible;
use std::sync::Arc;

use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::{
        sse::{Event, Sse},
        Html, IntoResponse,
    },
    routing, Json, Router,
};
use futures::stream::{Stream, StreamExt};
use serde_json::json;
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

#[derive(Debug, Deserialize)]
pub struct OAuthCallbackQuery {
    pub code: Option<String>,
    pub state: Option<String>,
    pub error: Option<String>,
    pub error_description: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct OAuthCallbackPayload {
    pub code: String,
    pub state: Option<String>,
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
        .route("/{id}/oauth/authorize", routing::post(oauth_authorize))
        .route("/{id}/oauth/callback", routing::post(oauth_callback))
        .route("/oauth/callback", routing::get(oauth_callback_get))
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
    let impl_instance = match create_publisher_impl(&ptype, &publisher.config) {
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

// ---------------------------------------------------------------------------
// OAuth endpoints
// ---------------------------------------------------------------------------

use crate::models::publisher::types::PublisherType;
use crate::models::publisher::x::XPublisher;
use crate::models::publisher::mastodon::MastodonPublisher;

/// POST /api/v1/publishers/oauth/authorize/{id}
pub async fn oauth_authorize(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> (StatusCode, Json<serde_json::Value>) {
    let manager = PublisherManager::new(state.pool.clone());
    let publisher = match manager.get_publisher(&id).await {
        Ok(p) => p,
        Err(e) => return (StatusCode::NOT_FOUND, Json(json!({"ok": false, "error": format!("Publisher not found: {e}")}))),
    };

    let ptype = publisher.publisher_type.clone();
    let config = publisher.config.clone();

    match ptype {
        PublisherType::X => {
            let x_pub = match XPublisher::new(&config) {
                Some(p) => p,
                None => return (StatusCode::BAD_REQUEST, Json(json!({"ok": false, "error": "Invalid X config - need client_id and client_secret"}))),
            };
            let oauth_state = uuid::Uuid::new_v4().to_string();
            let (auth_url, _code_verifier) = x_pub.generate_auth_url(Some(oauth_state.clone()));
            let mut states = state.oauth_states.lock().unwrap();
            states.insert(format!("x:{id}"), (oauth_state, std::time::Instant::now()));
            (StatusCode::OK, Json(json!({"ok": true, "url": auth_url})))
        }
        PublisherType::Mastodon => {
            let m_pub = match MastodonPublisher::new(&config) {
                Some(p) => p,
                None => return (StatusCode::BAD_REQUEST, Json(json!({"ok": false, "error": "Invalid Mastodon config - need server_url"}))),
            };

            // If no client_id, register the app first
            if m_pub.client_id.is_none() || m_pub.client_secret.is_none() {
                match m_pub.register_app().await {
                    Ok((client_id, client_secret)) => {
                        let mut updated_config = config.clone();
                        if let Some(obj) = updated_config.as_object_mut() {
                            obj.insert("client_id".to_string(), json!(client_id));
                            obj.insert("client_secret".to_string(), json!(client_secret));
                            obj.insert("redirect_uri".to_string(), json!(m_pub.redirect_uri));
                        }
                        let updated_pub = crate::models::publisher::types::Publisher {
                            id: publisher.id.clone(),
                            name: publisher.name.clone(),
                            publisher_type: PublisherType::Mastodon,
                            config: updated_config.clone(),
                            template: publisher.template.clone(),
                            active: publisher.active,
                            created_at: publisher.created_at.clone(),
                            updated_at: publisher.updated_at.clone(),
                        };
                        let _ = manager.update_publisher(&id, &updated_pub).await;

                        let m_pub2 = match MastodonPublisher::new(&updated_config) {
                            Some(p) => p,
                            None => return (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"ok": false, "error": "Failed to recreate Mastodon publisher"}))),
                        };
                        let oauth_state = uuid::Uuid::new_v4().to_string();
                        let auth_url = m_pub2.generate_auth_url(Some(oauth_state.clone()));
                        let mut states = state.oauth_states.lock().unwrap();
                        states.insert(format!("mastodon:{id}"), (oauth_state, std::time::Instant::now()));
                        return (StatusCode::OK, Json(json!({"ok": true, "url": auth_url})));
                    }
                    Err(e) => return (StatusCode::BAD_GATEWAY, Json(json!({"ok": false, "error": format!("Failed to register Mastodon app: {e}")}))),
                }
            }

            let oauth_state = uuid::Uuid::new_v4().to_string();
            let auth_url = m_pub.generate_auth_url(Some(oauth_state.clone()));
            let mut states = state.oauth_states.lock().unwrap();
            states.insert(format!("mastodon:{id}"), (oauth_state, std::time::Instant::now()));
            (StatusCode::OK, Json(json!({"ok": true, "url": auth_url})))
        }
        _ => (StatusCode::BAD_REQUEST, Json(json!({"ok": false, "error": "Publisher type does not support OAuth"}))),
    }
}

/// POST /api/v1/publishers/oauth/callback/{id}
pub async fn oauth_callback(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
    Json(payload): Json<OAuthCallbackPayload>,
) -> (StatusCode, Json<serde_json::Value>) {
    let manager = PublisherManager::new(state.pool.clone());
    let publisher = match manager.get_publisher(&id).await {
        Ok(p) => p,
        Err(e) => return (StatusCode::NOT_FOUND, Json(json!({"ok": false, "error": format!("Publisher not found: {e}")}))),
    };

    let ptype = publisher.publisher_type.clone();
    let config = publisher.config.clone();

    match ptype {
        PublisherType::X => {
            if let Some(ref cb_state) = payload.state {
                let mut states = state.oauth_states.lock().unwrap();
                let stored = states.remove(&format!("x:{id}"));
                match stored {
                    Some((ref stored_state, _)) if stored_state == cb_state => {}
                    Some(_) => return (StatusCode::UNAUTHORIZED, Json(json!({"ok": false, "error": "OAuth state mismatch"}))),
                    None => tracing::warn!("No stored OAuth state found for X publisher {id}"),
                }
            }

            let x_pub = match XPublisher::new(&config) {
                Some(p) => p,
                None => return (StatusCode::BAD_REQUEST, Json(json!({"ok": false, "error": "Invalid X config"}))),
            };

            let code_verifier = "challenge";
            match x_pub.exchange_code_for_tokens(&payload.code, code_verifier).await {
                Ok((access_token, refresh_token, _)) => {
                    let mut updated_config = config.clone();
                    if let Some(obj) = updated_config.as_object_mut() {
                        obj.insert("access_token".to_string(), json!(access_token));
                        if let Some(rt) = &refresh_token {
                            obj.insert("refresh_token".to_string(), json!(rt));
                        }
                    }
                    let updated_pub = crate::models::publisher::types::Publisher {
                        id: publisher.id.clone(), name: publisher.name.clone(),
                        publisher_type: PublisherType::X, config: updated_config,
                        template: publisher.template.clone(), active: publisher.active,
                        created_at: publisher.created_at, updated_at: publisher.updated_at,
                    };
                    let _ = manager.update_publisher(&id, &updated_pub).await;
                    (StatusCode::OK, Json(json!({"ok": true, "message": "X connected successfully!"})))
                }
                Err(e) => (StatusCode::BAD_GATEWAY, Json(json!({"ok": false, "error": format!("Token exchange failed: {e}")}))),
            }
        }
        PublisherType::Mastodon => {
            if let Some(ref cb_state) = payload.state {
                let mut states = state.oauth_states.lock().unwrap();
                let stored = states.remove(&format!("mastodon:{id}"));
                match stored {
                    Some((ref stored_state, _)) if stored_state == cb_state => {}
                    Some(_) => return (StatusCode::UNAUTHORIZED, Json(json!({"ok": false, "error": "OAuth state mismatch"}))),
                    None => tracing::warn!("No stored OAuth state found for Mastodon publisher {id}"),
                }
            }

            let m_pub = match MastodonPublisher::new(&config) {
                Some(p) => p,
                None => return (StatusCode::BAD_REQUEST, Json(json!({"ok": false, "error": "Invalid Mastodon config"}))),
            };

            match m_pub.exchange_code_for_tokens(&payload.code).await {
                Ok((access_token, _, _)) => {
                    let mut updated_config = config.clone();
                    if let Some(obj) = updated_config.as_object_mut() {
                        obj.insert("access_token".to_string(), json!(access_token));
                    }
                    let updated_pub = crate::models::publisher::types::Publisher {
                        id: publisher.id.clone(), name: publisher.name.clone(),
                        publisher_type: PublisherType::Mastodon, config: updated_config,
                        template: publisher.template.clone(), active: publisher.active,
                        created_at: publisher.created_at, updated_at: publisher.updated_at,
                    };
                    let _ = manager.update_publisher(&id, &updated_pub).await;
                    (StatusCode::OK, Json(json!({"ok": true, "message": "Mastodon connected successfully!"})))
                }
                Err(e) => (StatusCode::BAD_GATEWAY, Json(json!({"ok": false, "error": format!("Token exchange failed: {e}")}))),
            }
        }
        _ => (StatusCode::BAD_REQUEST, Json(json!({"ok": false, "error": "Publisher type does not support OAuth callback"}))),
    }
}

/// GET /api/v1/publishers/oauth/callback — called by OAuth provider, returns HTML popup
pub async fn oauth_callback_get(
    State(state): State<Arc<AppState>>,
    Query(query): Query<OAuthCallbackQuery>,
) -> axum::response::Response {
    if let Some(_error) = &query.error {
        let desc = query.error_description.as_deref().unwrap_or("OAuth authorization denied");
        return (StatusCode::OK, Html(oauth_result_html(false, desc))).into_response();
    }

    let code = match &query.code {
        Some(c) => c.clone(),
        None => return (StatusCode::BAD_REQUEST, Html(oauth_result_html(false, "Missing authorization code"))).into_response(),
    };
    let state_param = query.state.as_deref().unwrap_or("");

    // Resolve publisher_id from state
    let (publisher_id, stored_state) = {
        let states = state.oauth_states.lock().unwrap();
        let mut found = None;
        for (key, (stored, _)) in states.iter() {
            if stored == state_param {
                if let Some(id) = key.split(':').nth(1) {
                    found = Some((id.to_string(), stored.clone()));
                    break;
                }
            }
        }
        match found {
            Some(s) => s,
            None => return (StatusCode::BAD_REQUEST, Html(oauth_result_html(false, "No matching OAuth state found"))).into_response(),
        }
    };

    if state_param != stored_state {
        return (StatusCode::UNAUTHORIZED, Html(oauth_result_html(false, "OAuth state mismatch"))).into_response();
    }

    let manager = PublisherManager::new(state.pool.clone());
    let publisher = match manager.get_publisher(&publisher_id).await {
        Ok(p) => p,
        Err(_) => return (StatusCode::NOT_FOUND, Html(oauth_result_html(false, "Publisher not found"))).into_response(),
    };

    let ptype = publisher.publisher_type.clone();
    let config = publisher.config.clone();

    match ptype {
        PublisherType::X => {
            let x_pub = match XPublisher::new(&config) {
                Some(p) => p,
                None => return Html(oauth_result_html(false, "Invalid X config")).into_response(),
            };
            let code_verifier = "challenge";
            match x_pub.exchange_code_for_tokens(&code, code_verifier).await {
                Ok((access_token, refresh_token, _)) => {
                    let mut updated_config = config.clone();
                    if let Some(obj) = updated_config.as_object_mut() {
                        obj.insert("access_token".to_string(), json!(access_token));
                        if let Some(rt) = &refresh_token {
                            obj.insert("refresh_token".to_string(), json!(rt));
                        }
                    }
                    let updated_pub = crate::models::publisher::types::Publisher {
                        id: publisher.id.clone(), name: publisher.name.clone(),
                        publisher_type: PublisherType::X, config: updated_config,
                        template: publisher.template.clone(), active: publisher.active,
                        created_at: publisher.created_at, updated_at: publisher.updated_at,
                    };
                    let _ = manager.update_publisher(&publisher_id, &updated_pub).await;
                    Html(oauth_result_html(true, "X connected successfully!")).into_response()
                }
                Err(e) => Html(oauth_result_html(false, &format!("Token exchange failed: {e}"))).into_response(),
            }
        }
        PublisherType::Mastodon => {
            let m_pub = match MastodonPublisher::new(&config) {
                Some(p) => p,
                None => return Html(oauth_result_html(false, "Invalid Mastodon config")).into_response(),
            };
            match m_pub.exchange_code_for_tokens(&code).await {
                Ok((access_token, _, _)) => {
                    let mut updated_config = config.clone();
                    if let Some(obj) = updated_config.as_object_mut() {
                        obj.insert("access_token".to_string(), json!(access_token));
                    }
                    let updated_pub = crate::models::publisher::types::Publisher {
                        id: publisher.id.clone(), name: publisher.name.clone(),
                        publisher_type: PublisherType::Mastodon, config: updated_config,
                        template: publisher.template.clone(), active: publisher.active,
                        created_at: publisher.created_at, updated_at: publisher.updated_at,
                    };
                    let _ = manager.update_publisher(&publisher_id, &updated_pub).await;
                    Html(oauth_result_html(true, "Mastodon connected successfully!")).into_response()
                }
                Err(e) => Html(oauth_result_html(false, &format!("Token exchange failed: {e}"))).into_response(),
            }
        }
        _ => Html(oauth_result_html(false, "Unsupported publisher type")).into_response(),
    }
}

fn oauth_result_html(success: bool, message: &str) -> String {
    let status_str = if success { "success" } else { "error" };
    let title = if success { "\u{2705} Connected!" } else { "\u{274c} Connection failed" };
    let color = if success { "#22c55e" } else { "#ef4444" };
    let icon = if success { "\u{2705}" } else { "\u{274c}" };
    let escaped_msg = message
        .replace('"', "&quot;")
        .replace('<', "&lt;")
        .replace('>', "&gt;");
    format!(
        r#"<!DOCTYPE html>
<html lang="en">
<head><meta charset="UTF-8"><meta name="viewport" content="width=device-width, initial-scale=1.0"><title>OAuth {status_str}</title>
<style>*{{margin:0;padding:0;box-sizing:border-box;}}body{{font-family:-apple-system,BlinkMacSystemFont,'Segoe UI',Roboto,sans-serif;background:#0f0f1a;color:#e0e0e0;display:flex;justify-content:center;align-items:center;min-height:100vh;text-align:center;}}.card{{background:#1a1a2e;padding:2rem;border-radius:12px;max-width:420px;}}.icon{{font-size:3rem;margin-bottom:1rem;}}.title{{font-size:1.25rem;font-weight:600;margin-bottom:0.5rem;}}.message{{color:#a0a0b0;font-size:0.9rem;margin-bottom:1.5rem;}}.btn{{background:{color};color:white;border:none;padding:0.5rem 1.5rem;border-radius:6px;cursor:pointer;font-size:0.9rem;}}.btn:hover{{opacity:0.9;}}</style></head>
<body><div class="card"><div class="icon">{icon}</div><div class="title">{title}</div><div class="message">{escaped_msg}</div><button class="btn" onclick="window.close()">Close window</button></div>
<script>if(window.opener){{window.opener.postMessage({{type:'oauth-{status_str}',publisher:''}},'*');setTimeout(()=>window.close(),1500);}}</script></body></html>"#
    )
}