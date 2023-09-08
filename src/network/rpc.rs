use std::{
    net::SocketAddr,
    sync::{Arc, Mutex},
};

use log::info;

use crate::{
    core::{
        block::Block,
        blockchain::Blockchain,
        encoding::{ByteDecoding, ByteEncoding},
        transaction::Transaction,
    },
    lock,
};

use super::{
    error::NetworkError,
    message::PeerMessage,
    node::BlockMiner,
    tcp::TcpController,
    transport::{NetAddr, Payload},
    tx_pool::TxPool,
    types::ArcMut,
};
// use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy)]
#[repr(u16)]
pub enum RpcHeader {
    GetBlock = 1,
    GetTransaction,
    NewBlock,
    NewTransaction,
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

        // TODO: update RPC header to use from bytes,
        // placeholder is used for now
        let header = RpcHeader::Generic;
        // let header = RpcHeader::from(u16::from_be_bytes(buf));

        Ok(RPC {
            header,
            payload: data[2..].to_vec(),
        })
    }
}

pub struct RpcHandler {
    mem_pool: Arc<Mutex<TxPool>>,
    miner: Arc<Mutex<BlockMiner>>,
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
            miner,
            chain,
            tcp_controller,
        }
    }

    pub fn handle_rpc(&self, rpc: &RPC) -> Result<RpcHandlerResponse, NetworkError> {
        // TODO: handle all RPC header types
        match rpc.header {
            _ => {
                let mut mem_pool = lock!(self.mem_pool);

                // TODO: may need to use TcpController to send message back
                let tcp = lock!(self.tcp_controller);
                info!(
                    "RPC received with message: {}",
                    String::from_utf8_lossy(&rpc.payload)
                );

                // check if msg is transaction
                let tx = Transaction::new(&rpc.payload);
                // let mut mem_pool = lock!(mem_pool);
                mem_pool.add(tx);
                Ok(RpcHandlerResponse::Generic(
                    "Transaction added to mempool".to_string(),
                ))
            }
        }
    }
}
