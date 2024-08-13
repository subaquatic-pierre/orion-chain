use serde::{Deserialize, Serialize};
use serde_json::Value;

use super::error::CoreError;

pub trait ByteEncoding<T> {
    fn to_bytes(&self) -> Result<Vec<u8>, CoreError>;
    fn from_bytes(data: &[u8]) -> Result<T, CoreError>;
}

pub trait HexEncoding<T> {
    fn to_hex(&self) -> Result<String, CoreError>;
    fn from_hex(data: &str) -> Result<T, CoreError>;
}

pub trait JsonEncoding<T> {
    fn to_json(&self) -> Result<Value, CoreError>;
    fn from_json(data: Value) -> Result<T, CoreError>;
}
