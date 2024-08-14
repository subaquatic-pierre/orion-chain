use std::{
    fmt::Debug,
    net::SocketAddr,
    sync::{Arc, Mutex},
};

use log::{debug, info};
use serde::{Deserialize, Serialize};

use crate::{
    api::types::GetBlockReq,
    core::{
        block::Block, blockchain::Blockchain, encoding::ByteEncoding, error::CoreError,
        header::Header, transaction::Transaction,
    },
    crypto::private_key::PrivateKey,
    lock,
};

use crate::network::{
    error::NetworkError, miner::BlockMiner, tcp::TcpController, tx_pool::TxPool, types::Payload,
};

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
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

pub struct RpcHandler {
    mem_pool: Arc<Mutex<TxPool>>,
    _miner: Arc<Mutex<BlockMiner>>,
    chain: Arc<Mutex<Blockchain>>,
    _tcp_controller: Arc<Mutex<TcpController>>,
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
            _tcp_controller: tcp_controller,
        }
    }

    // simple wrapper method to be used in api routes/handlers
    // calls main handle_rpc method which is used for both peer RPC messages and client http requests
    pub fn handle_client_rpc(&self, rpc: &RPC) -> Result<RpcHandlerResponse, NetworkError> {
        self.handle_rpc(rpc, None)
    }

    pub fn handle_rpc(
        &self,
        rpc: &RPC,
        _peer_addr: Option<SocketAddr>,
    ) -> Result<RpcHandlerResponse, NetworkError> {
        // TODO: handle all RPC header types
        let payload = rpc.payload.clone();
        match rpc.header {
            RpcHeader::GetBlock => {
                debug!("rpc message received in handler at RpcHeader::GetBlock");
                match self.get_block(&payload) {
                    Ok(block) => Ok(RpcHandlerResponse::Block(block)),
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

    // ---
    // Private methods
    // ---

    fn get_block(&self, payload: &Vec<u8>) -> Result<Block, NetworkError> {
        let req: GetBlockReq = match bincode::deserialize(&payload) {
            Ok(req) => req,
            Err(e) => return Err(NetworkError::Decoding(e.to_string())),
        };

        let chain = lock!(self.chain);

        if req.hash.is_none() && req.height.is_none() {
            return Err(NetworkError::RPC(format!(
                "Incorrect request, must request with height or hash"
            )));
        }

        let block = if let Some(height) = &req.height {
            let block_id = match height.parse::<usize>() {
                Ok(height) => height,
                Err(e) => return Err(NetworkError::Decoding(e.to_string())),
            };

            chain.get_block(block_id)
        } else if let Some(hash) = &req.hash {
            chain.get_block_by_hash(&hash)
        } else {
            return Err(NetworkError::Decoding(
                "height or hash not supplied".to_string(),
            ));
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
            Ok(mut tx) => {
                let key = PrivateKey::new();
                tx.sign(&key)?;
                if let Ok(mut mem_pool) = self.mem_pool.lock() {
                    mem_pool.add(tx.clone());
                    debug!(
                        "adding transaction to the mem_pool in RpcHandler, hash: {}",
                        tx.hash()
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
