use std::{
    net::SocketAddr,
    sync::{Arc, Mutex},
};

use log::{debug, info};

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
        let chain = lock!(self.chain);
        // TODO: handle all RPC header types
        let payload = rpc.payload.clone();
        match rpc.header {
            RpcHeader::GetBlock => {
                debug!("rpc message received in handler at RpcHeader::GetBlock",);

                Ok(RpcHandlerResponse::Generic(format!("Generic response")))
            }
            RpcHeader::GetLastBlock => {
                debug!("rpc message received in handler at RpcHeader::GetLastBlock",);

                let block = chain.last_block();

                if let Some(block) = block {
                    Ok(RpcHandlerResponse::Block(Some(block.clone())))
                } else {
                    Ok(RpcHandlerResponse::Block(None))
                }
            }
            RpcHeader::NewBlock => {
                debug!("rpc message received in handler at RpcHeader::NewBlock",);

                Ok(RpcHandlerResponse::Generic(format!("Generic response")))
            }
            RpcHeader::GetChainHeight => {
                debug!("rpc message received in handler at RpcHeader::GetChainHeight",);

                Ok(RpcHandlerResponse::Generic(format!("Generic response")))
            }
            RpcHeader::GetTx => {
                debug!("rpc message received in RpcHeader::GetTx");

                Ok(RpcHandlerResponse::Generic(format!("Generic response")))
            }
            RpcHeader::NewTx => {
                let tx = Transaction::from_bytes(&payload);

                match tx {
                    Ok(tx) => {
                        if let Ok(mut mem_pool) = self.mem_pool.lock() {
                            mem_pool.add(tx.clone());
                            info!(
                                "adding transaction to the mem_pool in RpcHandler, hash: {}",
                                tx.hash().to_string()
                            );
                            Ok(RpcHandlerResponse::Transaction(tx))
                        } else {
                            Ok(RpcHandlerResponse::Generic(
                                "unable to lock mem_pool in RpcHandler".to_string(),
                            ))
                        }
                    }
                    Err(e) => Ok(RpcHandlerResponse::Generic(
                        "unable to handle RpcHeader::NewTx in RpcHandler".to_string(),
                    )),
                }
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
                debug!("rpc message received in handler at RpcHeader::GetBlock",);

                Ok(RpcHandlerResponse::Generic(format!("Generic response")))
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
