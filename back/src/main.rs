mod http;
mod models;

use axum::{
    http::{
        header::{ACCEPT, AUTHORIZATION, CONTENT_TYPE},
        Method,
    },
    Router,
};
use chrono::DateTime;
use html2text::from_read;
use http::{config_router, health_router, podcast_router, publishers_router, user_router};
use models::{
    publisher::{
        manager::{create_publisher_impl, PublisherManager},
        template::TemplateContext,
        types::PublishLog,
    },
    AppState, CompletePodcast, Error, Feed, Podcast, SseBroadcaster,
};
use rss::Item;
use sqlx::{
    migrate::{MigrateDatabase, Migrator},
    sqlite::{SqlitePool, SqlitePoolOptions},
};
use std::{env::var, path::Path, str::FromStr, sync::Arc, time::Duration};
use tower_http::services::{ServeDir, ServeFile};
use tower_http::{
    cors::{Any, CorsLayer},
    trace::TraceLayer,
};
use tracing::{debug, error, info};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

#[tokio::main]
async fn main() -> Result<(), Error> {
    let log_level = var("RUST_LOG").unwrap_or("debug".to_string());
    tracing_subscriber::registry()
        .with(EnvFilter::from_str(&log_level).unwrap())
        .with(tracing_subscriber::fmt::layer())
        .init();
    info!("Log level: {log_level}");
    let db_url = var("DB_URL").unwrap_or("podmixer.db".to_string());
    info!("DB url: {}", db_url);
    let port = var("PORT").unwrap_or("3000".to_string());
    info!("Port: {}", port);
    let secret = var("SECRET").unwrap_or("esto-es-un-secreto".to_string());
    let sleep_time: u64 = var("SLEEP_TIME")
        .unwrap_or("900".to_string())
        .parse()
        .unwrap();
    info!("Sleep time: {}", sleep_time);
    let older_than: i32 = var("OLDER_THAN")
        .unwrap_or("30".to_string())
        .parse()
        .unwrap();
    info!("Older than: {}", older_than);

    if !sqlx::Sqlite::database_exists(&db_url).await.unwrap() {
        sqlx::Sqlite::create_database(&db_url).await.unwrap();
    }

    let migrations = if var("RUST_ENV") == Ok("production".to_string()) {
        info!("Working on production");
        std::env::current_exe()
            .unwrap()
            .parent()
            .unwrap()
            .join("migrations")
    } else {
        info!("Working on development");
        let crate_dir = std::env::var("CARGO_MANIFEST_DIR").unwrap();
        Path::new(&crate_dir).join("migrations")
    };
    info!("{}", &migrations.display());

    let pool = SqlitePoolOptions::new()
        .max_connections(2)
        .connect(&db_url)
        .await
        .expect("Pool failed");

    Migrator::new(migrations)
        .await
        .unwrap()
        .run(&pool)
        .await
        .unwrap();

    let sse_broadcaster = SseBroadcaster::new();

    let api_routes = Router::new()
        .nest("/health", health_router())
        .nest("/auth", user_router())
        .nest("/podcasts", podcast_router())
        .nest("/config", config_router())
        .nest("/publishers", publishers_router())
        .with_state(Arc::new(AppState {
            pool: pool.clone(),
            secret,
            sse_broadcaster: sse_broadcaster.clone(),
            oauth_states: Arc::new(std::sync::Mutex::new(std::collections::HashMap::new())),
        }));

    let cors = CorsLayer::new()
        //.allow_origin(url.parse::<HeaderValue>().unwrap())
        .allow_origin(Any)
        .allow_methods([
            Method::GET,
            Method::POST,
            Method::PUT,
            Method::PATCH,
            Method::DELETE,
        ])
        //.allow_credentials(true)
        .allow_headers([AUTHORIZATION, ACCEPT, CONTENT_TYPE]);

    let app = Router::new()
        .nest_service("/rss", ServeDir::new("./rss"))
        .nest("/api/v1", api_routes)
        .fallback_service(ServeDir::new("static").fallback(ServeFile::new("static/index.html")))
        .layer(TraceLayer::new_for_http())
        .layer(cors);

    let pool2 = pool.clone();
    let sse2 = sse_broadcaster.clone();
    let dry_run = std::env::var("PUBLISHER_DRY_RUN")
        .ok()
        .map(|v| v == "1" || v.to_lowercase() == "true")
        .unwrap_or(false);
    tokio::spawn(async move {
        loop {
            match do_the_work(&pool2, older_than, &sse2, dry_run).await {
                Ok(_) => {}
                Err(error) => {
                    error!("do_the_work error: {error}");
                    let mut next_err = error.source();
                    while next_err.is_some() {
                        error!("caused by: {:#}", next_err.unwrap());
                        next_err = next_err.unwrap().source();
                    }
                }
            }
            tokio::time::sleep(Duration::from_secs(sleep_time)).await;
        }
    });
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await?;
    tracing::info!("🚀 Server started successfully");
    axum::serve(listener, app).await?;

    Ok(())
}

async fn do_the_work(pool: &SqlitePool, older_than: i32, sse_broadcaster: &SseBroadcaster, dry_run: bool) -> Result<(), Error> {
    debug!("Init feed");
    let feed = Feed::get(pool).await?;
    let mut new_episodes: Vec<Item> = Vec::new();
    let mut older_than_episodes: Vec<Item> = Vec::new();
    let mut all_episodes: Vec<Item> = Vec::new();
    let mut podcasts = Podcast::get(pool).await?;
    let mut generate = false;
    for podcast in podcasts.as_mut_slice() {
        match CompletePodcast::new(podcast).await {
            Ok(complete) => {
                match complete.get_new() {
                    Ok(news) => {
                        info!("Get episodes for: {}. News: {}", &podcast.name, news.len());
                        new_episodes.extend_from_slice(news.as_slice());
                        if !news.is_empty() {
                            generate = true;
                            let first = news.first().unwrap();
                            info!("{}", first.pub_date().unwrap());
                            if let Ok(pub_date) =
                                DateTime::parse_from_rfc2822(first.pub_date().unwrap())
                            {
                                podcast.last_pub_date = pub_date.to_utc();
                            } else if let Ok(pub_date) = DateTime::parse_from_str(
                                first.pub_date().unwrap(),
                                "%a, %d %b %Y %H:%M:%S",
                            ) {
                                podcast.last_pub_date = pub_date.to_utc();
                            }
                            match futures::executor::block_on(Podcast::update(pool, podcast)) {
                                Ok(response) => debug!("{:?}", response),
                                Err(e) => error!("{:?}", e),
                            };
                        }
                    }
                    Err(e) => error!("Error doing the work: {}", e),
                };
                match complete.get_older_than_days(older_than) {
                    Ok(older) => older_than_episodes.extend_from_slice(older.as_slice()),
                    Err(e) => error!("Error doing the work: {}", e),
                };
                let all = complete.get_all();
                all_episodes.extend_from_slice(all.as_slice());
            }
            Err(e) => error!("Error doing the work: {}", e),
        }
    }
    if generate {
        new_episodes.sort_by(|a, b| a.pub_date.cmp(&b.pub_date));
        for episode in new_episodes.as_slice() {
            let title = episode.title().unwrap_or("");
            let description = from_read(
                episode.description().unwrap_or("").as_bytes(),
                5000,
            )
            .unwrap_or_else(|_| "".to_string());
            let url = episode.link().unwrap_or("");
            info!(
                "Publishing episode: {}",
                title
            );
            publish_episode(pool, sse_broadcaster, title, &description, url, dry_run).await;
            tokio::time::sleep(Duration::from_secs(1)).await;
        }
        // Sort episodes
        all_episodes.sort_by(item_comparator);
        older_than_episodes.sort_by(item_comparator);
        //Make short feed
        debug!("Make short feed");
        match feed.rss(older_than_episodes) {
            Ok(short_feed) => {
                //debug!("{}", &short_feed);
                match std::fs::write("rss/short.xml", short_feed.as_bytes()) {
                    Ok(response) => debug!("{:?}", response),
                    Err(e) => error!("{:?}", e),
                };
            }
            Err(e) => error!("{:?}", e),
        };
        //Make long feed
        debug!("Make long feed");
        match feed.rss(all_episodes) {
            Ok(long_feed) => {
                //debug!("{}", &long_feed);
                match std::fs::write("rss/long.xml", long_feed.as_bytes()) {
                    Ok(response) => debug!("{:?}", response),
                    Err(e) => error!("{:?}", e),
                };
            }
            Err(e) => error!("{:?}", e),
        };
    }
    Ok(())
}

async fn publish_episode(
    pool: &SqlitePool,
    sse_broadcaster: &SseBroadcaster,
    title: &str,
    description: &str,
    url: &str,
    dry_run: bool,
) {
    let manager = PublisherManager::new(pool.clone());
    let publishers = match manager.get_publishers_db().await {
        Ok(p) => p.into_iter().filter(|p| p.active).collect::<Vec<_>>(),
        Err(e) => {
            error!("Error loading publishers: {:?}", e);
            return;
        }
    };

    for publisher in publishers {
        let ptype = publisher.publisher_type.clone();
        let ctx = TemplateContext {
            title: title.to_string(),
            description: description.to_string(),
            url: url.to_string(),
        };

        let log_id = uuid::Uuid::new_v4().to_string();
        let log = PublishLog {
            id: log_id.clone(),
            publisher_id: publisher.id.clone(),
            publisher_name: publisher.name.clone(),
            publisher_type: publisher.publisher_type.as_str().to_string(),
            episode_title: title.to_string(),
            status: "sending".to_string(),
            message: String::new(),
            created_at: String::new(),
        };
        let _ = manager.add_log(&log).await;

        if dry_run {
            info!("[DRY-RUN] Would publish to {}: {}", publisher.name, title);
            let dry_log = PublishLog {
                id: log_id,
                publisher_id: publisher.id,
                publisher_name: publisher.name,
                publisher_type: publisher.publisher_type.as_str().to_string(),
                episode_title: title.to_string(),
                status: "dry-run".to_string(),
                message: "Dry-run: publicación simulada".to_string(),
                created_at: String::new(),
            };
            let _ = manager.add_log(&dry_log).await;
            sse_broadcaster.broadcast(&dry_log);
            continue;
        }

        let impl_instance = create_publisher_impl(&ptype, &publisher.config);
        let impl_instance = match impl_instance {
            Some(instance) => instance,
            None => {
                error!("Invalid config for publisher {}", publisher.name);
                continue;
            }
        };

        match impl_instance.publish(&ctx.title, &ctx.description, &ctx.url).await {
            Ok(response) => {
                info!("Published to {}: {}", publisher.name, response);
                let success_log = PublishLog {
                    id: log_id,
                    publisher_id: publisher.id,
                    publisher_name: publisher.name,
                    publisher_type: publisher.publisher_type.as_str().to_string(),
                    episode_title: title.to_string(),
                    status: "success".to_string(),
                    message: "Published successfully".to_string(),
                    created_at: String::new(),
                };
                let _ = manager.add_log(&success_log).await;
                sse_broadcaster.broadcast(&success_log);
            }
            Err(e) => {
                let err_msg = format!("{}", e);
                error!("Error publishing to {}: {}", publisher.name, err_msg);
                drop(e);
                let error_log = PublishLog {
                    id: log_id,
                    publisher_id: publisher.id,
                    publisher_name: publisher.name,
                    publisher_type: publisher.publisher_type.as_str().to_string(),
                    episode_title: title.to_string(),
                    status: "error".to_string(),
                    message: err_msg,
                    created_at: String::new(),
                };
                let _ = manager.add_log(&error_log).await;
                sse_broadcaster.broadcast(&error_log);
            }
        }
    }
}

pub fn item_comparator(a: &Item, b: &Item) -> std::cmp::Ordering {
    let date_a = get_pub_date_timestamp(a);
    let date_b = get_pub_date_timestamp(b);
    date_b.cmp(&date_a)
}

pub fn get_pub_date_timestamp(item: &Item) -> i64 {
    if let Ok(pub_date) = DateTime::parse_from_rfc2822(item.pub_date().unwrap()) {
        pub_date.timestamp()
    } else if let Ok(pub_date) =
        DateTime::parse_from_str(item.pub_date().unwrap(), "%a, %d %b %Y %H:%M:%S")
    {
        pub_date.timestamp()
    } else {
        0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn truncate(value: String, length: usize) -> String {
        debug!("truncate");
        let mut cloned = value.clone();
        cloned.truncate(length);
        cloned
    }

    #[test]
    fn truncate_test_0() {
        let prueba = "1234567890".to_string();
        let result = truncate(prueba.clone(), 100);
        assert_eq!(prueba, result);
    }
    #[test]
    fn truncate_test_1() {
        let prueba = "1234567890".to_string();
        let result = truncate(prueba.clone(), 1);
        assert_eq!("1".to_string(), result);
    }
    #[test]
    fn truncate_test_2() {
        let prueba = "".to_string();
        let result = truncate(prueba.clone(), 10);
        assert_eq!(prueba, result);
    }
    #[test]
    fn truncate_test_3() {
        let prueba = "".to_string();
        let result = truncate(prueba.clone(), 0);
        assert_eq!(prueba, result);
    }

    #[test]
    fn truncate_test_4() {
        let prueba = "1234567890".to_string();
        let result = truncate(prueba.clone(), 100);
        assert_eq!(prueba, result);
    }
    #[test]
    fn truncate_test_5() {
        let prueba = "1234567890".to_string();
        let result = truncate(prueba.clone(), 1);
        assert_eq!("1".to_string(), result);
    }
    #[test]
    fn truncate_test_6() {
        let prueba = "".to_string();
        let result = truncate(prueba.clone(), 10);
        assert_eq!(prueba, result);
    }
    #[test]
    fn truncate_test_7() {
        let prueba = "".to_string();
        let result = truncate(prueba.clone(), 0);
        assert_eq!(prueba, result);
    }
    #[test]
    fn convert_1() {
        let date1 = "Fri, 28 Feb 2025 16:08:58 +0100";
        let value = DateTime::parse_from_rfc2822(date1);
        debug!("{:?}", value);
        assert!(value.is_ok());
    }
    #[test]
    fn convert_2() {
        let date1 = "Fri, 28 Feb 2025 16:08:58 GMT";
        let value = DateTime::parse_from_rfc2822(date1);
        debug!("{:?}", value);
        assert!(value.is_ok());
    }
    #[test]
    fn convert_3() {
        let date1 = "Fri, 28 Feb 2025 16:08:58";
        let value = DateTime::parse_from_rfc2822(date1);
        debug!("{:?}", value);
        assert!(value.is_ok());
    }
    #[test]
    fn convert_4() {
        let date1 = "Fri, 28 Feb 2025 16:08:58 GMT";
        let value = DateTime::parse_from_rfc2822(date1);
        debug!("{:?}", value);
        assert!(value.is_ok());
    }
}
