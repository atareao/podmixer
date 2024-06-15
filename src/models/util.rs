use std::error::Error;
use tokio::io::AsyncWriteExt;
use regex::Regex;
use std::{
    path::Path,
    ffi::OsStr
};

pub async fn fetch_url(url: &str, filename: &str) -> Result<(), Box<dyn Error>> {
    let mut file = tokio::fs::File::create(filename).await?;
    let mut response = reqwest::get(url).await?;
    while let Some(chunck) = response.chunk().await?{
        file.write_all(&chunck).await?;
    }
    Ok(())
}

pub fn normalize(title: &str) -> Result<String, Box<dyn Error>>{
    let re = Regex::new(r"[^a-zA-Z0-9\._-]");
    Ok(re?.replace_all(title, "_"))
}

pub fn get_extension_from_filename(filename: &str) -> Option<&str> {
    Path::new(filename)
        .extension()
        .and_then(OsStr::to_str)
}



#[cfg(test)]
mod test{
    use super::fetch_url;
    use super::normalize;
    use std::str::FromStr;
    use tracing_subscriber::{
        EnvFilter,
        layer::SubscriberExt,
        util::SubscriberInitExt
    };
    #[test]
    fn test_normalize(){
        let response = normalize("Esto es una prueba de audio.mp3");
        println!("{:?}", response);
        assert!(response.is_ok());
    }
    #[tokio::test]
    async fn test_fech_url(){
        tracing_subscriber::registry()
            .with(EnvFilter::from_str("debug").unwrap())
            .with(tracing_subscriber::fmt::layer())
        .init();
        let url = "https://anchor.fm/s/5a5b39c/podcast/play/87968517/https%3A%2F%2Fd3ctxlq1ktw2nl.cloudfront.net%2Fstaging%2F2024-5-13%2F5d279930-4426-d35f-7c3b-90314d30595d.mp3";
        let filename = "ejemplo.mp3";
        let result = fetch_url(url, filename).await;
        println!("{:?}", result);
        assert!(result.is_ok());
    }
}

