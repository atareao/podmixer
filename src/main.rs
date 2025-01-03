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
    Error,
    util,
    Telegram,
    Twitter,
};
use chrono::{DateTime, NaiveDateTime};
use minijinja::Value;
use rss::Item;
use minijinja::{Environment, context};
use html2text::from_read;

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
            match do_the_work(&pool2, older_than).await{
                Ok(_) => {},
                Err(error) => {
                    error!("do_the_work error: {error}");
                    let mut next_err = error.source();
                    while next_err.is_some(){
                        error!("caused by: {:#}", next_err.unwrap());
                        next_err = next_err.unwrap().source();
                    }
                },
            }
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

async fn do_the_work(pool: &SqlitePool, older_than: i32) -> Result<(), Error>{
    debug!("Init feed");
    let feed = Param::get_feed(pool).await?;
    let mut new_episodes: Vec<Item> = Vec::new();
    let mut older_than_episodes: Vec<Item> = Vec::new();
    let mut all_episodes: Vec<Item> = Vec::new();
    let mut podcasts = Podcast::get(pool).await?;
    let mut generate = false;
    for podcast in podcasts.as_mut_slice(){
        info!("Get episodes for {}", &podcast.name);
        match CompletePodcast::new(podcast).await{
            Ok(complete) => {
                match complete.get_new(){
                    Ok(news) => {
                        info!("Podcast: {}. News: {}", &podcast.name, news.len());
                        new_episodes.extend_from_slice(news.as_slice());
                        if !news.is_empty(){
                            generate = true;
                            let first = news.first().unwrap();
                            info!("{}", first.pub_date().unwrap());
                            if let Ok(pub_date) = DateTime::parse_from_rfc2822(first.pub_date().unwrap()){
                                podcast.last_pub_date = pub_date.naive_local();
                            }else if let Ok(pub_date) = NaiveDateTime::parse_from_str(first.pub_date().unwrap(), "%a, %d %b %Y %H:%M:%S") {
                                podcast.last_pub_date = pub_date;
                            }
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
        info!("Init telegram");
        let telegram = Param::get_telegram(pool).await?;
        info!("Init twitter");
        let mut twitter = Param::get_twitter(pool).await?;
        if twitter.is_active(){
            debug!("What before access_token: {}", twitter.get_access_token());
            debug!("What before refresh_token: {}", twitter.get_refresh_token());
            debug!("Update twitter");
            if twitter.update_access_token().await.is_ok(){
                let twitter_access_token = twitter.get_access_token();
                debug!("Access token: {twitter_access_token}");
                match Param::set(pool, "twitter_access_token", twitter_access_token).await{
                    Ok(response) => debug!("{:?}", response),
                    Err(e) => error!("{:?}", e),
                };
                let twitter_refresh_token = twitter.get_refresh_token();
                debug!("Refresh token: {twitter_refresh_token}");
                match Param::set(pool, "twitter_refresh_token", twitter_refresh_token).await{
                    Ok(response) => debug!("{:?}", response),
                    Err(e) => error!("{:?}", e),
                };
            }else{
                error!("Someting goes wrong");
            }
            debug!("What after access_token: {}", twitter.get_access_token());
            debug!("What after refresh_token: {}", twitter.get_refresh_token());
        }
        new_episodes.sort_by(|a, b| a.pub_date.cmp(&b.pub_date));
        for episode in new_episodes.as_slice(){
            let ctx = context!(
                title => episode.title().unwrap_or(""),
                description => from_read(
                    episode.description().unwrap_or("").as_bytes(),
                    5000),
                link => episode.link().unwrap(),
            );
            if telegram.is_active(){
                let template = Param::get(pool, "telegram_template")
                    .await
                    .unwrap();
                match populate_in_telegram(&ctx, &template, &telegram, episode).await{
                    Ok(_) => {
                        info!("Populated in Telegram: {}", episode.title().unwrap());
                    },
                    Err(error) => {
                        error!("Could NOT populate in Telegram: {error}");
                        let mut next_error = error.source();
                        // render causes as well
                        while next_error.is_some(){
                            error!("caused by: {:#}", next_error.unwrap());
                            next_error = next_error.unwrap().source();
                        }
                    },
                }
            }
            if twitter.is_active(){
                let template = Param::get(pool, "twitter_template")
                    .await
                    .unwrap();
                match populate_in_twitter(&ctx, &template, &twitter).await{
                    Ok(_) => info!("Populated in Twitter: {}", episode.title().unwrap()),
                    Err(error) => {
                        error!("Could NOT populate in Twitter: {error}");
                        let mut next_error = error.source();
                        // render causes as well
                        while next_error.is_some(){
                            error!("caused by: {:#}", next_error.unwrap());
                            next_error = next_error.unwrap().source();
                        }
                    },
                }
            }
            tokio::time::sleep(time::Duration::from_secs(1)).await;
        }
        // Sort episodes
        all_episodes.sort_by(|a, b| a.pub_date.cmp(&b.pub_date));
        older_than_episodes.sort_by(|a, b| a.pub_date.cmp(&b.pub_date));
        //Make short feed
        match feed.rss(older_than_episodes){
            Ok(short_feed) => {
                //debug!("{}", &short_feed);
                match std::fs::write("rss/short.xml", short_feed.as_bytes()){
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
                match std::fs::write("rss/long.xml", long_feed.as_bytes()){
                    Ok(response) => debug!("{:?}", response),
                    Err(e) => error!("{:?}", e),
                };
            },
            Err(e) => error!("{:?}", e),
        };
    }
    Ok(())
}

fn truncate(value: String, length: usize) -> String {
    debug!("truncate");
    match value.char_indices().nth(length) {
        Some((idx, _)) => value[..idx].to_string(),
        None => value,
    }
}

fn truncate2(value: String, length: usize) -> String {
    debug!("truncate");
    let mut cloned = value.clone();
    cloned.truncate(length);
    cloned
}

async fn populate_in_telegram(ctx: &Value, template: &str, telegram: &Telegram, episode: &Item) -> Result<(), Error>{
    let mut env = Environment::new();
    env.add_filter("truncate", truncate);
    env.add_template("telegram", template)?;
    let tmpl = env.get_template("telegram")?;
    let url = episode.enclosure().ok_or("Not enclosure")?.url();
    let name = util::normalize(episode.title().ok_or("Not title")?)?;
    let ext = util::get_extension_from_filename(url).ok_or("Not extension")?;
    let filename = format!("{name}.{ext}");
    let filepath = format!("/tmp/{filename}");
    util::fetch_url(url, &filepath).await?;
    let message = tmpl.render(ctx)?;
    telegram.send_audio(&filename, &filepath, &message).await?;
    tokio::fs::remove_file(filepath).await?;
    Ok(())
}

async fn populate_in_twitter(ctx: &Value, template: &str, twitter: &Twitter) -> Result<(), Error>{
    debug!("populate_in_twitter");
    let mut env = Environment::new();
    env.add_filter("truncate", truncate);
    env.add_template("twitter", template)?;
    let tmpl = env.get_template("twitter")?;
    debug!("Template: {template}");
    debug!("Context: {:?}", ctx);
    debug!("Env: {:?}", env);
    debug!("tmpl: {:?}", &tmpl);
    let message = tmpl.render(ctx)?;
    debug!("message: {message}");
    twitter.post(&message).await?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

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
        let result = truncate2(prueba.clone(), 100);
        assert_eq!(prueba, result);
    }
    #[test]
    fn truncate_test_5() {
        let prueba = "1234567890".to_string();
        let result = truncate2(prueba.clone(), 1);
        assert_eq!("1".to_string(), result);
    }
    #[test]
    fn truncate_test_6() {
        let prueba = "".to_string();
        let result = truncate2(prueba.clone(), 10);
        assert_eq!(prueba, result);
    }
    #[test]
    fn truncate_test_7() {
        let prueba = "".to_string();
        let result = truncate2(prueba.clone(), 0);
        assert_eq!(prueba, result);
    }
}
