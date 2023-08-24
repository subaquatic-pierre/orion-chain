use std::{error::Error, fmt::Display};

#[derive(Debug)]
pub enum CoreError {
    Parsing(String),
    Transaction(String),
}

impl Error for CoreError {}

impl Display for CoreError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Parsing(msg) => write!(f, "{msg}"),
            Self::Transaction(msg) => write!(f, "{msg}"),
            _ => write!(f, "Unknown keypair error"),
        }
    }
}
