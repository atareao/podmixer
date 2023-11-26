use tokio;
use futures;
use sqlx::{
    sqlite::{
        SqlitePoolOptions,
        SqlitePool,
    },
    migrate::{
        Migrator,
        MigrateDatabase
    },
};
use tracing_subscriber::{
    EnvFilter,
    layer::SubscriberExt,
    util::SubscriberInitExt
};
use core::time;
use std::{
    str::FromStr,
    env::var,
    path::Path,
};
use tracing::{info, debug, error};
use models::{
    Param,
    Podcast,
    CompletePodcast,
};
use chrono::DateTime;
use rss::Item;
use minijinja::{Environment, context};

mod models;
mod http;



#[tokio::main]
async fn main(){
    let log_level = var("RUST_LOG").unwrap_or("debug".to_string());
    tracing_subscriber::registry()
        .with(EnvFilter::from_str(&log_level).unwrap())
        .with(tracing_subscriber::fmt::layer())
        .init();
    info!("Log level: {log_level}");

    let db_url = var("DB_URL").unwrap_or("podmixer.db".to_string());
    info!("DB url: {}", db_url);

    if !sqlx::Sqlite::database_exists(&db_url).await.unwrap(){
        sqlx::Sqlite::create_database(&db_url).await.unwrap();
    }

    let migrations = if var("RUST_ENV") == Ok("production".to_string()){
        std::env::current_exe().unwrap().parent().unwrap().join("migrations")
    }else{
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

    let sleep_time = Param::get_sleep_time(&pool).await;
    let older_than = Param::get_older_than(&pool).await;

    let pool2 = pool.clone();
    tokio::spawn(async move {
        loop {
            do_the_work(&pool2, older_than).await;
            tokio::time::sleep(
                time::Duration::from_secs(sleep_time)
            ).await;
        }
    });
    tracing::info!("🚀 Server started successfully");
    http::serve(&pool)
        .await
        .unwrap();
}

async fn do_the_work(pool: &SqlitePool, older_than: i32) {
    debug!("Init telegram");
    let telegram = Param::get_telegram(pool).await.unwrap();
    debug!("Init twitter");
    let mut twitter = Param::get_twitter(pool).await.unwrap();
    debug!("Update twitter");
    if twitter.update_access_token().await.is_ok(){
        let access_token = twitter.get_access_token();
        match Param::set(pool, "access_token", access_token).await{
            Ok(response) => debug!("{:?}", response),
            Err(e) => error!("{:?}", e),
        };
        let refresh_token = twitter.get_refresh_token();
        match Param::set(pool, "refresh_token", refresh_token).await{
            Ok(response) => debug!("{:?}", response),
            Err(e) => error!("{:?}", e),
        };
    }
    debug!("Init feed");
    let feed = Param::get_feed(pool).await.unwrap();
    let mut new_episodes: Vec<Item> = Vec::new();
    let mut older_than_episodes: Vec<Item> = Vec::new();
    let mut all_episodes: Vec<Item> = Vec::new();
    let mut podcasts = Podcast::get(pool).await.unwrap();
    let mut generate = false;
    for podcast in podcasts.as_mut_slice(){
        debug!("Get episodes for {}", &podcast.name);
        match CompletePodcast::new(&podcast).await{
            Ok(complete) => {
                match complete.get_new(){
                    Ok(news) => {
                        debug!("Podcast: {}. News: {}", &podcast.name, news.len());
                        new_episodes.extend_from_slice(news.as_slice());
                        if news.len() > 0{
                            generate = true;
                            let first = news.first().clone();
                            debug!("{}", first.unwrap().pub_date().unwrap());
                            let pub_date = DateTime::parse_from_rfc2822(first.unwrap().pub_date().unwrap())
                                .unwrap()
                                .naive_local();
                            podcast.last_pub_date = pub_date;
                            match futures::executor::block_on(Podcast::update(pool, podcast)){
                                Ok(response) => debug!("{:?}", response),
                                Err(e) => error!("{:?}", e),
                            };
                        }
                    },
                    Err(e) => error!("Error doing the work: {}", e),
                };
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
    if generate {
        new_episodes.sort_by(|a, b| a.pub_date.cmp(&b.pub_date));
        for episode in new_episodes.as_slice(){
            if telegram.is_active(){
                let template = Param::get(pool, "telegram_template")
                    .await
                    .unwrap();
                let mut env = Environment::new();
                env.add_template("telegram", &template).unwrap();
                let tmpl = env.get_template("telegram").unwrap();
                let ctx = context!(
                    title => episode.title().unwrap(),
                    description => episode.description().unwrap(),
                );
                let audio = &episode.enclosure().unwrap().url();
                let message = tmpl.render(ctx).unwrap();
                telegram.send_audio(&audio, &message).await;
            }
            if twitter.is_active(){
                let template = Param::get(pool, "twitter_template")
                    .await
                    .unwrap();
                let mut env = Environment::new();
                env.add_template("telegram", &template).unwrap();
                let tmpl = env.get_template("telegram").unwrap();
                let ctx = context!(
                    title => episode.title().unwrap(),
                    description => episode.description().unwrap(),
                );
                let message = tmpl.render(ctx).unwrap();
                match twitter.post(&message).await{
                    Ok(response) => debug!("{:?}", response),
                    Err(e) => error!("{:?}", e),
                }
            }
            tokio::time::sleep(time::Duration::from_secs(1)).await;
        }
        //Make short feed
        match feed.rss(older_than_episodes){
            Ok(short_feed) => {
                //debug!("{}", &short_feed);
                match std::fs::write("rss/long.xml", &short_feed.as_bytes()){
                    Ok(response) => debug!("{:?}", response),
                    Err(e) => error!("{:?}", e),
                };
            },
            Err(e) => error!("{:?}", e),
        };
        //Make long feed
        match feed.rss(all_episodes){
            Ok(long_feed) => {
                //debug!("{}", &long_feed);
                match std::fs::write("rss/long.xml", &long_feed.as_bytes()){
                    Ok(response) => debug!("{:?}", response),
                    Err(e) => error!("{:?}", e),
                };
            },
            Err(e) => error!("{:?}", e),
        };
    }
}
