use std::{error::Error, fmt::Display};

#[derive(Debug)]
pub enum KeyPairError {
    GenerateError(String),
}

impl Error for KeyPairError {}

impl Display for KeyPairError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::GenerateError(msg) => write!(f, "{msg}"),
            _ => write!(f, "Unknown keypair error"),
        }
    }
}
