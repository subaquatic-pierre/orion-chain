use std::{convert, error::Error, fmt::Display, io};

use crate::crypto::error::CryptoError;

#[derive(Debug)]
pub enum CoreError {
    Serialize(String),
    Parsing(String),
    Transaction(String),
    Block(String),
    CryptoError(String),
}

impl Error for CoreError {}

impl Display for CoreError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            msg => write!(f, "{msg}"),
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
