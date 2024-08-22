use serde::{Deserialize, Serialize};
use std::path::Path;
use std::sync::Arc;

use crate::core::encoding::ByteEncoding;
use crate::core::error::CoreError;

#[derive(Serialize, Deserialize, Debug)]
pub struct Account {
    pub balance: u64,
    // TODO: implement nonce on account
    // pub nonce: u64,
}

impl ByteEncoding<Account> for Account {
    fn from_bytes(data: &[u8]) -> Result<Account, CoreError> {
        Ok(bincode::deserialize(data)?)
    }
    fn to_bytes(&self) -> Result<Vec<u8>, CoreError> {
        Ok(bincode::serialize(&self)?)
    }
}
