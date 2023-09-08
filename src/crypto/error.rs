use std::{error::Error, fmt::Display};

#[derive(Debug)]
pub enum CryptoError {
    GenerateKey(String),
    HashError(String),
    SignatureError(String),
}

impl Error for CryptoError {}

impl Display for CryptoError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::GenerateKey(msg) => write!(f, "{msg}"),
            Self::HashError(msg) => write!(f, "{msg}"),
            Self::SignatureError(msg) => write!(f, "{msg}"),
        }
    }
}
