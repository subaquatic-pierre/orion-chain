use serde::{Deserialize, Serialize};
use std::path::Path;
use std::sync::Arc;

use crate::core::encoding::ByteEncoding;
use crate::core::error::CoreError;

#[derive(Serialize, Deserialize, Debug)]
pub struct Account {
    pub balance: u64,
    pub nonce: u64,
    // Add other account fields as needed
}

impl ByteEncoding<Account> for Account {
    fn from_bytes(data: &[u8]) -> Result<Account, CoreError> {
        Ok(bincode::deserialize(data)?)
    }
    fn to_bytes(&self) -> Result<Vec<u8>, CoreError> {
        Ok(bincode::serialize(&self)?)
    }
}