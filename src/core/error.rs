use std::{convert, error::Error, fmt::Display, io};

use crate::crypto::error::CryptoError;

#[derive(Debug)]
pub enum CoreError {
    Parsing(String),
    Transaction(String),
    Block(String),
    CryptoError(String),
}

impl Error for CoreError {}

impl Display for CoreError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Parsing(msg) => write!(f, "{msg}"),
            Self::Transaction(msg) => write!(f, "{msg}"),
            Self::Block(msg) => write!(f, "{msg}"),
            Self::CryptoError(msg) => write!(f, "{msg}"),
        }
    }
}

impl From<CryptoError> for CoreError {
    fn from(value: CryptoError) -> Self {
        CoreError::CryptoError(format!("{value}"))
    }
}

impl convert::From<io::ErrorKind> for CoreError {
    fn from(value: io::ErrorKind) -> Self {
        CoreError::Parsing(format!("{value}"))
    }
}
