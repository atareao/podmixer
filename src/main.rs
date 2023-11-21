use tokio;
use models::Configuration;
use tracing_subscriber::{
    EnvFilter,
    layer::SubscriberExt,
    util::SubscriberInitExt
};
use core::time;
use std::str::FromStr;

mod models;



#[tokio::main]
async fn main(){
    let configuration = Configuration::load()
        .await
        .expect("Cant load configuration");
    tracing_subscriber::registry()
        .with(EnvFilter::from_str(configuration.get_log_level()).unwrap())
        .with(tracing_subscriber::fmt::layer())
        .init();
    tracing::info!("Log level: {}", configuration.get_log_level());
    tokio::spawn(async move {
        loop {
            do_the_work(&configuration).await;
            tokio::time::sleep(
                time::Duration::from_secs(
                    configuration.get_sleep_time().try_into().unwrap()
                )
            ).await;
        }
    });
    tracing::info!("🚀 Server started successfully");
}

async fn do_the_work(configuration: &Configuration) {
    println!("1");

}
