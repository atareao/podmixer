use std::{
    sync::Arc,
    env::var,
};

use axum::{
    extract::{Query, State},
    http::StatusCode,
    response::IntoResponse,
    routing, Json, Router,
};
use tracing::{debug, error};
use rss::Item;
use chrono::DateTime;

use crate::models::{
    CompletePodcast,
    ApiResponse,
    AppState,
    Data,
    Podcast,
    Feed,
    NewPodcast,
    Id
};

pub fn podcast_router() -> Router<Arc<AppState>> {
    Router::new()
        .route("/", routing::post(create))
        .route("/", routing::patch(update))
        .route("/", routing::get(read))
        .route("/", routing::delete(delete))
        .route("/generate", routing::post(regenerate_feed))
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

pub fn get_pub_date_timestamp(item: &Item) -> i64{
    if let Ok(pub_date) = DateTime::parse_from_rfc2822(item.pub_date().unwrap()){
        pub_date.timestamp()
    }else if let Ok(pub_date) = DateTime::parse_from_str(item.pub_date().unwrap(), "%a, %d %b %Y %H:%M:%S") {
        pub_date.timestamp()
    }else {
        0
    }

}
pub fn item_comparator(a: &Item, b: &Item) -> std::cmp::Ordering {
    let date_a = get_pub_date_timestamp(a);
    let date_b = get_pub_date_timestamp(b);
    date_b.cmp(&date_a)
}

pub async fn regenerate_feed(
    State(app_state): State<Arc<AppState>>,
) -> impl IntoResponse {
    debug!("Regenerate feed");
    
    // 1. Get all podcasts from the database once.
    let mut podcasts = Podcast::get(&app_state.pool).await.unwrap();

    // 2. Initialize vectors to hold all episodes outside the loop.
    let mut all_episodes: Vec<Item> = Vec::new();
    let mut older_than_episodes: Vec<Item> = Vec::new();
    let older_than: i32 = var("OLDER_THAN").unwrap_or("30".to_string()).parse().unwrap();

    for podcast in podcasts.as_mut_slice() {
        match CompletePodcast::new(podcast).await {
            Ok(complete) => {
                // Collect episodes from each podcast.
                match complete.get_older_than_days(older_than) {
                    Ok(older) => older_than_episodes.extend(older),
                    Err(e) => error!("Error collecting older episodes: {}", e),
                };

                let all = complete.get_all();
                all_episodes.extend(all);
            },
            Err(e) => error!("Error creating CompletePodcast: {}", e),
        }
    }

    // 3. Sort the collected episodes once, outside the loop.
    all_episodes.sort_by(item_comparator);
    older_than_episodes.sort_by(item_comparator);

    // 4. Get the Feed instance once.
    let feed = Feed::get(&app_state.pool).await.unwrap();

    // 5. Generate and write the short feed.
    debug!("Making short feed");
    match feed.rss(older_than_episodes) {
        Ok(short_feed) => {
            match std::fs::write("rss/short.xml", short_feed.as_bytes()) {
                Ok(_) => debug!("Short feed written successfully"),
                Err(e) => error!("Error writing short feed: {:?}", e),
            };
        },
        Err(e) => error!("Error generating short feed: {:?}", e),
    };

    // 6. Generate and write the long feed.
    debug!("Making long feed");
    match feed.rss(all_episodes) {
        Ok(long_feed) => {
            match std::fs::write("rss/long.xml", long_feed.as_bytes()) {
                Ok(_) => debug!("Long feed written successfully"),
                Err(e) => error!("Error writing long feed: {:?}", e),
            };
        },
        Err(e) => error!("Error generating long feed: {:?}", e),
    };

    StatusCode::OK
}
