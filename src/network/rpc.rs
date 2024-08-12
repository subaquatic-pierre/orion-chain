use std::{
    fmt::Debug,
    net::SocketAddr,
    sync::{Arc, Mutex},
};

use log::{debug, info};
use serde::Serialize;

use crate::{
    api::types::GetBlockReq,
    core::{
        block::Block,
        blockchain::Blockchain,
        encoding::{ByteDecoding, ByteEncoding},
        header::Header,
        transaction::Transaction,
    },
    lock,
};

use super::{
    error::NetworkError, node::BlockMiner, tcp::TcpController, tx_pool::TxPool, types::Payload,
};

#[derive(Debug, Clone, Copy)]
#[repr(u16)]
pub enum RpcHeader {
    GetBlock = 1,
    GetBlockHeader,
    GetLastBlock,
    GetChainHeight,
    GetTx,
    NewTx,
    NewBlock,
    Generic,
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
pub enum RpcHandlerResponse {
    Block(Block),
    Transaction(Transaction),
    Error(String),
    Generic(String),
    Header(Header),
}

#[derive(Debug, Clone)]
pub struct RPC {
    pub header: RpcHeader,
    pub payload: Payload,
}

impl ByteEncoding for RPC {
    fn to_bytes(&self) -> Vec<u8> {
        let mut buf = vec![];
        let header_num: u16 = self.header.into();
        let header_bytes = header_num.to_be_bytes();
        buf.extend_from_slice(&header_bytes);
        buf.extend_from_slice(&self.payload);
        buf
    }
}

impl ByteDecoding for RPC {
    type Error = NetworkError;
    type Target = RPC;

    fn from_bytes(data: &[u8]) -> Result<RPC, NetworkError> {
        if data.is_empty() {
            return Err(NetworkError::Decoding(
                "empty bytes passed to RPC decoding".to_string(),
            ));
        }

        if data.len() < 2 {
            return Err(NetworkError::Decoding(
                "incorrect header bytes passed to RPC decoding".to_string(),
            ));
        }

        let buf: [u8; 2] = [data[0], data[1]];

        let header = RpcHeader::from(u16::from_be_bytes(buf));

        Ok(RPC {
            header,
            payload: data[2..].to_vec(),
        })
    }
}

pub struct RpcHandler {
    mem_pool: Arc<Mutex<TxPool>>,
    _miner: Arc<Mutex<BlockMiner>>,
    chain: Arc<Mutex<Blockchain>>,
    tcp_controller: Arc<Mutex<TcpController>>,
}

impl RpcHandler {
    pub fn new(
        mem_pool: Arc<Mutex<TxPool>>,
        miner: Arc<Mutex<BlockMiner>>,
        chain: Arc<Mutex<Blockchain>>,
        tcp_controller: Arc<Mutex<TcpController>>,
    ) -> Self {
        Self {
            mem_pool,
            _miner: miner,
            chain: chain,
            tcp_controller,
        }
    }

    pub fn handle_client_rpc(&self, rpc: &RPC) -> Result<RpcHandlerResponse, NetworkError> {
        // TODO: handle all RPC header types
        let payload = rpc.payload.clone();
        match rpc.header {
            RpcHeader::GetBlock => {
                debug!("rpc message received in handler at RpcHeader::GetBlock");
                match self.get_block(&payload) {
                    Ok(block) => Ok(RpcHandlerResponse::Block(block.clone())),
                    Err(msg) => Ok(RpcHandlerResponse::Generic(msg.to_string())),
                }
            }
            RpcHeader::GetLastBlock => {
                debug!("rpc message received in handler at RpcHeader::GetLastBlock");

                match self.get_last_block() {
                    Ok(block) => Ok(RpcHandlerResponse::Block(block.clone())),
                    Err(msg) => Ok(RpcHandlerResponse::Generic(msg.to_string())),
                }
            }
            RpcHeader::NewBlock => {
                debug!("rpc message received in handler at RpcHeader::NewBlock");

                Ok(RpcHandlerResponse::Generic(format!("Generic response")))
            }
            RpcHeader::GetChainHeight => {
                debug!("rpc message received in handler at RpcHeader::GetChainHeight");

                Ok(RpcHandlerResponse::Generic(format!("Generic response")))
            }
            RpcHeader::GetTx => {
                debug!("rpc message received in RpcHeader::GetTx");

                Ok(RpcHandlerResponse::Generic(format!("Generic response")))
            }
            RpcHeader::NewTx => {
                debug!("rpc message received in handler at RpcHeader::NewTx");

                match self.new_tx(&payload) {
                    Ok(tx) => Ok(RpcHandlerResponse::Transaction(tx)),
                    Err(msg) => Ok(RpcHandlerResponse::Generic(msg.to_string())),
                }
            }
            RpcHeader::GetBlockHeader => {
                debug!("rpc message received in handler at RpcHeader::GetBlockHeader");
                match self.get_block_header(&payload) {
                    Ok(header) => Ok(RpcHandlerResponse::Header(header.clone())),
                    Err(msg) => Ok(RpcHandlerResponse::Generic(msg.to_string())),
                }
            }
            _ => Ok(RpcHandlerResponse::Generic(
                "unknown RPC header requested".to_string(),
            )),
        }
    }

    pub fn handle_peer_rpc(
        &self,
        rpc: &RPC,
        peer_addr: SocketAddr,
    ) -> Result<RpcHandlerResponse, NetworkError> {
        // TODO: handle all RPC header types
        let tcp = lock!(self.tcp_controller);
        let payload = rpc.payload.clone();
        match rpc.header {
            RpcHeader::GetBlock => {
                debug!("rpc message received in peer handler at RpcHeader::GetBlock");
                match self.get_block(&payload) {
                    Ok(block) => Ok(RpcHandlerResponse::Block(block.clone())),
                    Err(msg) => Ok(RpcHandlerResponse::Generic(msg.to_string())),
                }
            }
            RpcHeader::GetLastBlock => {
                debug!("rpc message received in handler at RpcHeader::GetBlock",);

                Ok(RpcHandlerResponse::Generic(format!("Generic response")))
            }
            RpcHeader::NewBlock => {
                debug!("rpc message received in handler at RpcHeader::GetBlock",);

                Ok(RpcHandlerResponse::Generic(format!("Generic response")))
            }
            RpcHeader::GetChainHeight => {
                debug!("rpc message received in handler at RpcHeader::GetBlock",);

                Ok(RpcHandlerResponse::Generic(format!("Generic response")))
            }
            RpcHeader::GetTx => {
                debug!("rpc message received in handler at RpcHeader::GetBlock",);

                Ok(RpcHandlerResponse::Generic(format!("Generic response")))
            }
            RpcHeader::NewTx => {
                debug!("rpc message received in handler at RpcHeader::GetBlock",);

                Ok(RpcHandlerResponse::Generic(format!("Generic response")))
            }
            _ => Ok(RpcHandlerResponse::Generic(
                "unknown RPC header requested".to_string(),
            )),
        }
    }

    // ---
    // Private methods
    // ---

    fn get_block(&self, payload: &Vec<u8>) -> Result<Block, NetworkError> {
        let req: GetBlockReq = bincode::deserialize(&payload).unwrap();
        let chain = lock!(self.chain);

        if req.hash.is_none() && req.height.is_none() {
            return Err(NetworkError::RPC(format!(
                "Incorrect request, must request with height or hash"
            )));
        }

        let block = if let Some(height) = &req.height {
            let block_id = height.parse::<usize>().unwrap();

            chain.get_block(block_id)
        } else {
            let hash = req.hash.clone();
            let hash = hash.unwrap();

            chain.get_block_by_hash(&hash)
        };

        if let Some(block) = block {
            Ok(block.clone())
        } else {
            if let Some(height) = req.height {
                return Err(NetworkError::RPC(format!(
                    "Block with height: {height} not found"
                )));
            } else {
                let hash = req.hash.unwrap();
                return Err(NetworkError::RPC(format!(
                    "Block with hash: {hash} not found"
                )));
            }
        }
    }

    fn get_block_header(&self, payload: &Vec<u8>) -> Result<Header, NetworkError> {
        match self.get_block(payload) {
            Ok(block) => return Ok(block.header().clone()),
            Err(msg) => Err(NetworkError::RPC(msg.to_string())),
        }
    }

    fn get_last_block(&self) -> Result<Block, NetworkError> {
        let chain = lock!(self.chain);

        let block = chain.last_block();

        if let Some(block) = block {
            Ok(block.clone())
        } else {
            Err(NetworkError::RPC(format!("Last block not found")))
        }
    }

    fn new_tx(&self, payload: &Vec<u8>) -> Result<Transaction, NetworkError> {
        let tx = Transaction::from_bytes(&payload);

        match tx {
            Ok(tx) => {
                if let Ok(mut mem_pool) = self.mem_pool.lock() {
                    mem_pool.add(tx.clone());
                    debug!(
                        "adding transaction to the mem_pool in RpcHandler, hash: {}",
                        tx.hash().to_string()
                    );
                    Ok(tx)
                } else {
                    Err(NetworkError::RPC(
                        "unable to lock mem_pool in RpcHandler".to_string(),
                    ))
                }
            }
            Err(e) => Err(NetworkError::RPC(format!(
                "unable to handle RpcHeader::NewTx in RpcHandler, {e}"
            ))),
        }
    }
}
