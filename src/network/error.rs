use crate::core::error::CoreError;
use actix_web::{
    body::BoxBody, error::ResponseError, http::StatusCode, web::Json, HttpResponse, Responder,
};
use serde_json::json;

use std::{error::Error, fmt::Display};

#[derive(Debug)]
pub enum NetworkError {
    Connect(String),
    NotFound(String),
    Message(String),
    Decoding(String),
    RPC(String),
}

impl Error for NetworkError {}

impl Display for NetworkError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            NetworkError::Connect(msg) => write!(f, "{msg}"),
            NetworkError::NotFound(msg) => write!(f, "{msg}"),
            NetworkError::Message(msg) => write!(f, "{msg}"),
            NetworkError::Decoding(msg) => write!(f, "{msg}"),
            NetworkError::RPC(msg) => write!(f, "{msg}"),
        }
    }
}

impl From<CoreError> for NetworkError {
    fn from(value: CoreError) -> Self {
        NetworkError::RPC(format!("{value}"))
    }
}

impl Responder for NetworkError {
    type Body = BoxBody;
    fn respond_to(self, _req: &actix_web::HttpRequest) -> HttpResponse<Self::Body> {
        let message = match self {
            NetworkError::Connect(msg) => msg,
            NetworkError::NotFound(msg) => msg,
            NetworkError::Message(msg) => msg,
            NetworkError::Decoding(msg) => msg,
            NetworkError::RPC(msg) => msg,
        };

        let status = StatusCode::from_u16(403).unwrap_or(StatusCode::BAD_REQUEST);

        HttpResponse::build(status).json(json!({"error": message}))
    }
}
