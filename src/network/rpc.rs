use std::{
    net::SocketAddr,
    sync::{Arc, Mutex},
};

use log::info;

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
    error::NetworkError, node::BlockMiner, tcp::TcpController, transport::Payload, tx_pool::TxPool,
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
    Block(Option<Block>),
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
    _mem_pool: Arc<Mutex<TxPool>>,
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
            _mem_pool: mem_pool,
            _miner: miner,
            chain: chain,
            tcp_controller,
        }
    }

    pub fn handle_client_rpc(&self, rpc: &RPC) -> Result<RpcHandlerResponse, NetworkError> {
        let chain = lock!(self.chain);
        // TODO: handle all RPC header types
        let payload = rpc.payload.clone();
        match rpc.header {
            RpcHeader::GetBlock => {
                info!(
                    "rpc message received with data: {}",
                    String::from_utf8(payload).unwrap()
                );

                Ok(RpcHandlerResponse::Generic(format!("Generic response")))
            }
            RpcHeader::GetLastBlock => {
                info!(
                    "rpc message received with data: {}",
                    String::from_utf8(payload).unwrap()
                );

                let block = chain.last_block();

                if let Some(block) = block {
                    Ok(RpcHandlerResponse::Block(Some(block.clone())))
                } else {
                    Ok(RpcHandlerResponse::Block(None))
                }
            }
            RpcHeader::NewBlock => {
                info!(
                    "rpc message received with data: {}",
                    String::from_utf8(payload).unwrap()
                );

                Ok(RpcHandlerResponse::Generic(format!("Generic response")))
            }
            RpcHeader::GetChainHeight => {
                info!(
                    "rpc message received with data: {}",
                    String::from_utf8(payload).unwrap()
                );

                Ok(RpcHandlerResponse::Generic(format!("Generic response")))
            }
            RpcHeader::GetTx => {
                info!(
                    "rpc message received with data: {}",
                    String::from_utf8(payload).unwrap()
                );

                Ok(RpcHandlerResponse::Generic(format!("Generic response")))
            }
            RpcHeader::NewTx => {
                info!(
                    "rpc message received with data: {}",
                    String::from_utf8(payload).unwrap()
                );

                Ok(RpcHandlerResponse::Generic(format!("Generic response")))
            }
            RpcHeader::GetBlockHeader => {
                let req: GetBlockReq = bincode::deserialize(&payload).unwrap();

                let block_id = req.id.parse::<usize>().unwrap();

                let chain = self.chain.lock().unwrap();
                let header = chain.get_header(block_id);

                if let Some(header) = header {
                    Ok(RpcHandlerResponse::Header(header.clone()))
                } else {
                    Ok(RpcHandlerResponse::Generic(format!(
                        "Block with id: {block_id} not found"
                    )))
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
                info!(
                    "rpc message received with data: {}",
                    String::from_utf8(payload).unwrap()
                );

                Ok(RpcHandlerResponse::Generic(format!("Generic response")))
            }
            RpcHeader::GetLastBlock => {
                info!(
                    "rpc message received with data: {}",
                    String::from_utf8(payload).unwrap()
                );

                Ok(RpcHandlerResponse::Generic(format!("Generic response")))
            }
            RpcHeader::NewBlock => {
                info!(
                    "rpc message received with data: {}",
                    String::from_utf8(payload).unwrap()
                );

                Ok(RpcHandlerResponse::Generic(format!("Generic response")))
            }
            RpcHeader::GetChainHeight => {
                info!(
                    "rpc message received with data: {}",
                    String::from_utf8(payload).unwrap()
                );

                Ok(RpcHandlerResponse::Generic(format!("Generic response")))
            }
            RpcHeader::GetTx => {
                info!(
                    "rpc message received with data: {}",
                    String::from_utf8(payload).unwrap()
                );

                Ok(RpcHandlerResponse::Generic(format!("Generic response")))
            }
            RpcHeader::NewTx => {
                info!(
                    "rpc message received with data: {}",
                    String::from_utf8(payload).unwrap()
                );

                Ok(RpcHandlerResponse::Generic(format!("Generic response")))
            }
            RpcHeader::GetBlockHeader => {
                let req: GetBlockReq = bincode::deserialize(&payload).unwrap();

                let block_id = req.id.parse::<usize>().unwrap();

                let chain = self.chain.lock().unwrap();
                let header = chain.get_header(block_id);

                if let Some(header) = header {
                    Ok(RpcHandlerResponse::Header(header.clone()))
                } else {
                    Ok(RpcHandlerResponse::Generic(format!(
                        "Block with id: {block_id} not found"
                    )))
                }
            }
            _ => Ok(RpcHandlerResponse::Generic(
                "unknown RPC header requested".to_string(),
            )),
        }
    }
}
