use std::sync::Arc;
use axum::{
    Router,
    Extension,
    routing,
    response::{
        IntoResponse,
        Html,
        Json
    },
    middleware,
    http::header::{self, HeaderValue},
    extract::{Path, Query, State},
};
use base64::{engine::general_purpose, Engine as _};

use crate::{
    http::{
        AppState,
        jwt_auth::auth,
        user::do_login,
    },
    models::{
        episode::Episode,
        channel::Channel,
        parameters::Parameters,
        user::UserSchema,
    }
};

pub fn router(app_state: Arc<AppState>) -> Router<Arc<AppState>> {
    Router::new()
        .route("/favicon.ico",
            routing::get(favicon)
        )
        .route("/healthcheck",
            routing::get(healthcheck)
        )
        .route("/status",
            routing::get(get_root)
        )
        .route("/",
            routing::get(get_channels)
                .route_layer(middleware::from_fn_with_state(app_state.clone(), auth))
        )
        .route("/login",
            routing::get(login).post(do_login)
        )
        .route("/:path",
            routing::get(get_podcast)
        )
}

async fn favicon() -> impl IntoResponse {
    let one_pixel_favicon = "";
    let pixel_favicon= general_purpose::STANDARD.decode(one_pixel_favicon).unwrap();
    ([(header::CONTENT_TYPE, HeaderValue::from_static("image/png"))], pixel_favicon)
}

async fn get_root(
    t: Extension<Tera>,
) -> impl IntoResponse{
    let mut context = Context::new();
    context.insert("title", "Title");
    Html(t.render("index.html", &context).unwrap())
}

async fn get_channels(
    State(app_state): State<Arc<AppState>>,
    t: Extension<Tera>,
) -> impl IntoResponse{
    let mut context = Context::new();
    context.insert("title", "Channels");
    let channels = match Channel::read_all(&app_state.pool).await{
        Ok(channels) => channels,
        Err(_) => Vec::new(),
    };
    tracing::debug!("Channels: {:?}", &channels);
    context.insert("channels", &channels);
    Html(t.render("channels.html", &context).unwrap())
}

async fn login(
    t: Extension<Tera>,
) -> impl IntoResponse{
    let context = Context::new();
    Html(t.render("login.html", &context).unwrap())
}

// async fn login_post(
//     State(app_state): State<Arc<AppState>>,
//     Form(user_data): Form<UserSchema>
// ) -> impl IntoResponse{
//     tracing::debug!("{:?}", user_data);
//     //do_login(app_state, user_data).await
//     //Html("esto funciona aparentemente")
// }

async fn healthcheck() -> impl IntoResponse{
    Json(serde_json::json!({
        "status": "success",
        "message": "Up and running"
    }))
}


async fn get_podcast(
    State(app_state): State<Arc<AppState>>,
    t: Extension<Tera>,
    Path(channel_id): Path<i64>,
    parameters: Query<Parameters>
) -> impl IntoResponse{
    let per_page = app_state.config.get_page();
    let total = Episode::number_of_episodes(&app_state.pool, channel_id).await / per_page + 1;
    let page = match parameters.page {
        Some(value) =>  {
            tracing::debug!("Página solicitada: {}", value);
            if value > 0 {
                value
            }else{
                1
            }
        },
        None => 1,
    };
    tracing::debug!("A leer la página: {}", page);
    let mut context = Context::new();
    match Episode::read_with_pagination_in_channel(&app_state.pool, channel_id, page, per_page).await{
        Ok(episodes) => {
            let base_url = app_state.config.get_url();
            context.insert("title", &channel_id);
            context.insert("base_url", base_url);
            context.insert("episodes", &episodes);
            context.insert("page", &page);
            context.insert("per_page", &per_page);
            context.insert("total", &total);
            Html(t.render("podcast.html", &context).unwrap())
        },
        Err(_) => {
            context.insert("title", "Channels");
            let channels: Vec<Channel> = match Channel::read_all(&app_state.pool).await{
                Ok(channels) => channels,
                Err(_) => Vec::new(),
            };
            context.insert("channels", &channels);
            Html(t.render("podcast.html", &context).unwrap())

        }
    }
}

