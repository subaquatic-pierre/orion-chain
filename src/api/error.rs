use std::{
    error::Error,
    fmt::{self, Display},
};

use actix_web::{body::BoxBody, http::StatusCode, web::Json, HttpResponse, Responder};
use serde_json::json;

#[derive(Debug)]
pub struct ApiError {
    pub message: String,
    pub status: StatusCode,
}

impl ApiError {
    pub fn new(msg: &str, status: u16) -> Self {
        let status = StatusCode::from_u16(status).unwrap_or(StatusCode::BAD_REQUEST);
        Self {
            message: msg.to_string(),
            status,
        }
    }

    pub fn new_400(msg: &str) -> Self {
        let status = StatusCode::from_u16(400).unwrap_or(StatusCode::BAD_REQUEST);
        Self {
            message: msg.to_string(),
            status,
        }
    }

    pub fn new_500(msg: &str) -> Self {
        let status = StatusCode::from_u16(500).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR);
        Self {
            message: msg.to_string(),
            status,
        }
    }
}

impl Default for ApiError {
    fn default() -> Self {
        Self::new("There was an error", 400)
    }
}

impl Error for ApiError {}

impl Display for ApiError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl Responder for ApiError {
    type Body = BoxBody;
    fn respond_to(self, _req: &actix_web::HttpRequest) -> HttpResponse<Self::Body> {
        HttpResponse::build(self.status).json(json!({"error": self.message}))
    }
}

pub type ApiResult<T> = Result<T, ApiError>;

pub trait ApiModel {
    fn save<T>(&self) -> ApiResult<T>;
}
