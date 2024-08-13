use std::{error::Error, fmt::Display};

use crate::core::error::CoreError;

#[derive(Debug)]
pub enum CryptoError {
    GenerateKey(String),
    HashError(String),
    SignatureError(String),
    CoreError(String),
}

impl Error for CryptoError {}

impl Display for CryptoError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            msg => write!(f, "{msg}"),
        }
    }
}

impl From<CoreError> for CryptoError {
    fn from(value: CoreError) -> Self {
        CryptoError::CoreError(value.to_string())
    }
}
