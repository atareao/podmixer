use serde::Deserialize;
use std::sync::Arc;
use serde_json::{json, Value};

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

use crate::{
    models::{
        Param,
        Podcast,
        CompletePodcast,
        FormPodcast,
    },
    http::{
        AppState,
        jwt_auth::auth
    },
};
use tracing::{error, debug, info};
use rss::Item;

#[derive(Deserialize)]
struct Params {
    id: i64,
}


pub fn router(app_state: Arc<AppState>) -> Router<Arc<AppState>> {
    Router::new()
        .route("/podcasts",
            routing::get(get_podcasts)
            .route_layer(from_fn_with_state(app_state.clone(), auth))
        )
        .route("/podcasts",
            routing::post(create_or_update)
            .route_layer(from_fn_with_state(app_state.clone(), auth))
        )
        .route("/podcasts",
            routing::delete(delete)
            .route_layer(from_fn_with_state(app_state.clone(), auth))
        )
        .route("/force_rss_generation",
            routing::get(generate_rss)
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

async fn create_or_update(
    State(app_state): State<Arc<AppState>>,
    Json(podcast): Json<FormPodcast>,
) -> impl IntoResponse{
    Podcast::create_or_update(&app_state.pool, &podcast.name, &podcast.url, podcast.active)
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

async fn generate_rss(
    State(app_state): State<Arc<AppState>>,
) -> impl IntoResponse{
    info!("generate_rss");
    let older_than = Param::get_older_than(&app_state.pool).await;
    let feed = Param::get_feed(&app_state.pool).await.unwrap();
    let mut older_than_episodes: Vec<Item> = Vec::new();
    let mut all_episodes: Vec<Item> = Vec::new();
    let mut podcasts = Podcast::get(&app_state.pool).await.unwrap();
    for podcast in podcasts.as_mut_slice(){
        debug!("Get episodes for {}", &podcast.name);
        match CompletePodcast::new(podcast).await{
            Ok(complete) => {
                match complete.get_older_than_days(older_than){
                    Ok(older) => older_than_episodes.extend_from_slice(older.as_slice()),
                    Err(e) => error!("Error doing the work: {}", e),
                };
                let all = complete.get_all();
                all_episodes.extend_from_slice(all.as_slice());
            },
            Err(e) => error!("Error doing the work: {}", e),
        }
    }
    // Sort episodes
    all_episodes.sort_by(|a, b| a.pub_date.cmp(&b.pub_date));
    older_than_episodes.sort_by(|a, b| a.pub_date.cmp(&b.pub_date));
    // Make short feed
    let shorter_rss = match feed.rss(older_than_episodes){
        Ok(short_feed) => {
            //debug!("{}", &short_feed);
            match std::fs::write("rss/short.xml", short_feed.as_bytes()){
                Ok(response) => {
                    debug!("{:?}", response);
                    json!({
                        "result": "ok",
                        "content": "complete",
                    })
                },
                Err(e) => {
                    error!("{:?}", e);
                    json!({
                        "result": "ko",
                        "content": e.to_string(),
                    })
                },
            }
        },
        Err(e) => {
            error!("{:?}", e);
            json!({
                "result": "ko",
                "content": e.to_string(),
            })
        },
    };
    //Make long feed
    let longer_rss = match feed.rss(all_episodes){
        Ok(long_feed) => {
            //debug!("{}", &long_feed);
            match std::fs::write("rss/long.xml", long_feed.as_bytes()){
                Ok(response) => {
                    debug!("{:?}", response);
                    json!({
                        "result": "ok",
                        "content": "complete",
                    })
                },
                Err(e) => {
                    error!("{:?}", e);
                    json!({
                        "result": "ko",
                        "content": e.to_string(),
                    })
                },
            }
        },
        Err(e) => {
            error!("{:?}", e);
            json!({
                "result": "ko",
                "content": e.to_string(),
            })
        },
    };
    if longer_rss.get("result") == Some(&Value::from("ok")) && 
            shorter_rss.get("result") == Some(&Value::from("ok")){
        Json(json!({
            "result": "ok",
            "shorter": shorter_rss,
            "longer": longer_rss,
        }))
    }else{
        Json(json!({
            "result": "ko",
            "shorter": shorter_rss,
            "longer": longer_rss,
        }))
    }
}

