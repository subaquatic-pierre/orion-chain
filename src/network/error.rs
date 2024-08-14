use std::{error::Error, fmt::Display};

use crate::core::error::CoreError;

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
