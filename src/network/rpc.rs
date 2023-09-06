use crate::core::encoding::ByteEncoding;

use super::transport::{NetAddr, Payload};

#[derive(Debug, Clone)]
pub struct RPC {
    pub sender: NetAddr,
    pub receiver: NetAddr,
    pub payload: Payload,
}

impl ByteEncoding for RPC {
    fn to_bytes(&self) -> Vec<u8> {
        self.payload.clone()
    }
}
