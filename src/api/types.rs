use crate::network::rpc::RpcHandler;
use bytes::Bytes;
use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};

pub type GenericError = Box<dyn std::error::Error + Send + Sync>;
pub type Result<T> = std::result::Result<T, GenericError>;
pub type BoxBody = http_body_util::combinators::BoxBody<Bytes, hyper::Error>;

pub type ArcRcpHandler = Arc<Mutex<RpcHandler>>;

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TxsJson {
    pub count: usize,
    pub hashes: Vec<String>,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BlockJson {
    pub version: u8,
    pub height: usize,
    pub hash: String,
    pub prev_hash: String,
    pub timestamp: u64,
    pub txs: TxsJson,
    // pub nonce: usize,
    // pub difficulty:
}

#[derive(Serialize, Deserialize, Debug)]
pub struct GetBlockReq {
    pub height: Option<String>,
    pub hash: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct NewTxReq {
    pub value: String,
}

#[derive(Serialize, Deserialize)]
pub struct GetTxReq {
    pub id: String,
}

#[derive(Serialize, Deserialize)]
pub struct GenericReq {
    pub ts: String,
}
