use std::{
    fmt,
    convert::{
        TryFrom,
        TryInto
    },
};

#[derive(Debug)]
pub enum CustomError {
    BadRequest,
    NotFound,
    ServerError(String),
    OtherError(String),
    Io(tokio::io::Error),
    Yaml(serde_yaml::Error),
    Reqwest(reqwest::Error)
}

impl fmt::Display for CustomError {
    // This trait requires `fmt` with this exact signature.
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        // Write strictly the first element into the supplied output
        // stream: `f`. Returns `fmt::Result` which indicates whether the
        // operation succeeded or failed. Note that `write!` uses syntax which
        // is very similar to `println!`.
        match self{
            Self::BadRequest =>  write!(f, "Bad request"),
            Self::NotFound =>  write!(f, "Not found"),
            Self::ServerError(e) =>  write!(f, "Server error: {}", e),
            Self::OtherError(e) =>  write!(f, "Server error: {}", e),
            Self::Io(e) =>  write!(f, "Io error: {}", e),
            Self::Yaml(e) =>  write!(f, "Yaml error: {}", e),
            Self::Reqwest(e) =>  write!(f, "Reqwest error: {}", e),
            _ => write!(f, "Unknown error"),
        }
    }
}

impl From<serde_yaml::Error> for CustomError{
    fn from(value: serde_yaml::Error) -> Self{
        CustomError::Yaml(value)
    }
}

impl From<tokio::io::Error> for CustomError{
    fn from(value: tokio::io::Error) -> Self{
        CustomError::Io(value)
    }
}

impl From<reqwest::Error> for CustomError{
    fn from(value: reqwest::Error) -> Self{
        CustomError::Reqwest(value)
    }
}
