use std::fmt;

#[derive(Debug)]
pub enum CustomError{
    Reqwest(reqwest::Error),

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
            _ => write!(f, "Unknown error"),
        }
    }
}
impl IntoResponse for CustomError {
    fn into_response(self) -> Response {
        let (status, error_message) = match self {
            Self::ServerError(s) => (StatusCode::INTERNAL_SERVER_ERROR, s),
            Self::OtherError(s) => (StatusCode::INTERNAL_SERVER_ERROR, s),
            Self::BadRequest=> (StatusCode::BAD_REQUEST, "Bad Request".to_string()),
            Self::NotFound => (StatusCode::NOT_FOUND, "Not Found".to_string()),
        };
        (status, Json(json!({
            "status": "error",
            "message": error_message}))
        ).into_response()
    }
}
