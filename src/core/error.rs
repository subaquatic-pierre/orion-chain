use actix_web::{
    body::BoxBody, error::ResponseError, http::StatusCode, web::Json, HttpResponse, Responder,
};
use serde_json::json;
use std::{convert, error::Error, fmt::Display, io};

use crate::crypto::error::CryptoError;

#[derive(Debug)]
pub enum CoreError {
    Serialize(String),
    Parsing(String),
    Transaction(String),
    Block(String),
    CryptoError(String),
    State(String),
}

impl Error for CoreError {}

impl Display for CoreError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Serialize(msg) => write!(f, "{}", msg),
            Self::Parsing(msg) => write!(f, "{}", msg),
            Self::Transaction(msg) => write!(f, "{}", msg),
            Self::Block(msg) => write!(f, "{}", msg),
            Self::CryptoError(msg) => write!(f, "{}", msg),
            Self::State(msg) => write!(f, "{}", msg),
        }
    }
}

impl From<CryptoError> for CoreError {
    fn from(value: CryptoError) -> Self {
        CoreError::CryptoError(format!("{value}"))
    }
}
impl From<bincode::ErrorKind> for CoreError {
    fn from(value: bincode::ErrorKind) -> Self {
        CoreError::Serialize(format!("{value}"))
    }
}

impl From<Box<bincode::ErrorKind>> for CoreError {
    fn from(value: Box<bincode::ErrorKind>) -> Self {
        CoreError::Serialize(format!("{value}"))
    }
}

impl From<hex::FromHexError> for CoreError {
    fn from(value: hex::FromHexError) -> Self {
        CoreError::Serialize(format!("{value}"))
    }
}

impl From<io::ErrorKind> for CoreError {
    fn from(value: io::ErrorKind) -> Self {
        CoreError::Parsing(format!("{value}"))
    }
}

impl From<Box<dyn Error>> for CoreError {
    fn from(value: Box<dyn Error>) -> Self {
        CoreError::Parsing(format!("{value}"))
    }
}

impl Responder for CoreError {
    type Body = BoxBody;
    fn respond_to(self, _req: &actix_web::HttpRequest) -> HttpResponse<Self::Body> {
        let message = match self {
            Self::Serialize(msg) => msg,
            Self::Parsing(msg) => msg,
            Self::Transaction(msg) => msg,
            Self::Block(msg) => msg,
            Self::CryptoError(msg) => msg,
            Self::State(msg) => msg,
        };

        let status = StatusCode::from_u16(403).unwrap_or(StatusCode::BAD_REQUEST);

        HttpResponse::build(status).json(json!({"error": message}))
    }
}
