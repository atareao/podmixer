use axum::{
    http::StatusCode,
    Json,
    body::Body,
    response::{
        Response,
        IntoResponse,
    }
};
use serde::Serialize;

use super::Data;

#[derive(Debug, Serialize, Clone)]
pub struct ApiResponse {
    pub status: u16,
    pub message: String,
    pub data: Data,
}

impl ApiResponse {
    pub fn new(status: StatusCode, message: &str, data: Data) -> Self {
        Self {
            status: status.as_u16(),
            message: message.to_string(),
            data,
        }
    }
    pub fn create(status: StatusCode, message: &str, data: Data) -> Json<ApiResponse> {
        Json(ApiResponse::new(status, message, data))
    }
}

impl IntoResponse for ApiResponse {
    fn into_response(self) -> Response<Body> {
        let body = serde_json::to_string(&self).unwrap();
        Response::builder()
            .status(self.status)
            .header("Content-Type", "application/json")
            .body(Body::from(body))
            .unwrap()
    }
}

