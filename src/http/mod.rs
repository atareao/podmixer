use std::{sync::Arc, net::{SocketAddr, Ipv4Addr}};
use axum::{
    Router,
    http::{
        header::{
            ACCEPT,
            AUTHORIZATION,
            CONTENT_TYPE
        },
        HeaderValue,
        Method,
    },
};
use crate::models::{
    Configuration,
    Error
};
use tower_http::{
    trace::TraceLayer,
    cors::CorsLayer,
};

pub mod estatic;
pub mod root;
pub mod jwt_auth;
pub mod user;


#[derive(Clone)]
pub struct AppState {
    pub config: Configuration,
}

pub async fn serve(config: Configuration) -> Result<(), Error> {

    let cors = CorsLayer::new()
        .allow_origin(config.get_url().parse::<HeaderValue>().unwrap())
        .allow_methods([Method::GET, Method::POST, Method::PATCH, Method::DELETE])
        .allow_credentials(true)
        .allow_headers([AUTHORIZATION, ACCEPT, CONTENT_TYPE]);

    let app = api_router(
            AppState {
                config: config.clone(),
            })
            .layer(TraceLayer::new_for_http())
            .layer(cors);

    axum::Server::bind(
        &SocketAddr::new(std::net::IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0)), config.get_port()))
        .serve(app.into_make_service())
        .await
        .map_err(Error::from)
}

fn api_router(app_state: AppState) -> Router {
    channel::router()
        .merge(episode::router())
        .merge(rss::router())
        .merge(estatic::router())
        .merge(root::router(Arc::new(app_state.clone())))
        .merge(user::router(Arc::new(app_state.clone())))
        .with_state(Arc::new(app_state.clone()))
}

