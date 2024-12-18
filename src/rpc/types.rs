use std::{
    fmt::Debug,
    net::SocketAddr,
    sync::{Arc, Mutex},
};

use log::{debug, info};
use serde::{Deserialize, Serialize};

use crate::{
    core::{
        block::Block, blockchain::Blockchain, encoding::ByteEncoding, error::CoreError,
        header::Header, transaction::Transaction,
    },
    crypto::private_key::PrivateKey,
    lock,
};

use crate::network::types::Payload;

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[repr(u16)]
pub enum RpcHeader {
    GetBlock = 1,
    GetBlockHeader,
    GetLastBlock,
    GetChainHeight,
    GetTx,
    NewTx,
    Generic,
    CommitBlock,
    BlockProposal,
    BlockVote,
}

impl From<u16> for RpcHeader {
    fn from(value: u16) -> Self {
        unsafe { ::std::mem::transmute(value) }
    }
}

impl From<RpcHeader> for u16 {
    fn from(value: RpcHeader) -> u16 {
        value as u16
    }
}

#[derive(Debug, Clone)]
pub enum RpcResponse {
    Block(Block),
    Transaction(Transaction),
    Error(String),
    Generic(String),
    Header(Header),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RPC {
    pub header: RpcHeader,
    pub payload: Payload,
}

impl ByteEncoding<RPC> for RPC {
    fn to_bytes(&self) -> Result<Vec<u8>, CoreError> {
        Ok(bincode::serialize(&self)?)
    }

    fn from_bytes(data: &[u8]) -> Result<RPC, CoreError> {
        Ok(bincode::deserialize(data)?)
    }
}
