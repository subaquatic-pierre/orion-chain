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
            Self::Connect(msg) => write!(f, "{msg}"),
            Self::NotFound(msg) => write!(f, "{msg}"),
            Self::Message(msg) => write!(f, "{msg}"),
            Self::Decoding(msg) => write!(f, "{msg}"),
            Self::RPC(msg) => write!(f, "{msg}"),
        }
    }
}
