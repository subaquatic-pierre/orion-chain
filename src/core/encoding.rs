use serde::Serialize;
use std::error::Error;

use crate::api::types::{BlockJson, TxsJson};

pub trait ByteEncoding {
    fn to_bytes(&self) -> Vec<u8>;
}
pub trait ByteDecoding {
    type Target;
    type Error: Error;
    fn from_bytes(data: &[u8]) -> Result<Self::Target, Self::Error>;
}

pub trait HexDecoding {
    type Target;
    type Error: Error;

    fn from_hex(data: &str) -> Result<Self::Target, Self::Error>;
}

pub trait HexEncoding {
    fn to_hex(&self) -> String;
}

pub trait JsonEncoding {
    type Target: Serialize;
    fn to_json(&self) -> Self::Target;
}
