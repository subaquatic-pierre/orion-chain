use std::{
    fmt::Debug,
    net::SocketAddr,
    sync::{Arc, Mutex},
};

use log::{debug, info};
use serde::{Deserialize, Serialize};

use crate::{
    core::{block::Block, blockchain::Blockchain, encoding::ByteEncoding, error::CoreError},
    lock,
    rpc::handlers::{
        block::{get_block, get_block_header, get_last_block},
        transaction::new_tx,
    },
    vm::validator::BlockValidator,
};

use crate::network::{error::NetworkError, tcp::TcpController, tx_pool::TxPool, types::Payload};

use crate::rpc::types::{RpcHeader, RpcResponse, RPC};

pub struct RpcController {
    mem_pool: Arc<Mutex<TxPool>>,
    validator: Arc<Mutex<BlockValidator>>,
    chain: Arc<Mutex<Blockchain>>,
    tcp_controller: Arc<Mutex<TcpController>>,
}

impl RpcController {
    pub fn new(
        mem_pool: Arc<Mutex<TxPool>>,
        validator: Arc<Mutex<BlockValidator>>,
        chain: Arc<Mutex<Blockchain>>,
        tcp_controller: Arc<Mutex<TcpController>>,
    ) -> Self {
        Self {
            mem_pool,
            validator,
            chain: chain,
            tcp_controller,
        }
    }

    // simple wrapper method to be used in api routes/handlers
    // calls main handle_rpc method which is used for both peer RPC messages and client http requests
    pub fn handle_client_rpc(&self, rpc: &RPC) -> Result<RpcResponse, NetworkError> {
        self.handle_rpc(rpc, None)
    }

    pub fn handle_rpc(
        &self,
        rpc: &RPC,
        _peer_addr: Option<SocketAddr>,
    ) -> Result<RpcResponse, NetworkError> {
        match rpc.header {
            RpcHeader::GetBlock => {
                debug!("rpc message received in handler at RpcHeader::GetBlock");
                match get_block(&rpc, self.chain.clone()) {
                    Ok(block) => Ok(RpcResponse::Block(block)),
                    Err(msg) => Ok(RpcResponse::Generic(msg.to_string())),
                }
            }
            RpcHeader::GetLastBlock => {
                debug!("rpc message received in handler at RpcHeader::GetLastBlock");

                match get_last_block(&rpc, self.chain.clone()) {
                    Ok(block) => Ok(RpcResponse::Block(block.clone())),
                    Err(msg) => Ok(RpcResponse::Generic(msg.to_string())),
                }
            }
            RpcHeader::NewBlock => {
                debug!("rpc message received in handler at RpcHeader::NewBlock");

                Ok(RpcResponse::Generic(format!("Generic response")))
            }
            RpcHeader::GetChainHeight => {
                debug!("rpc message received in handler at RpcHeader::GetChainHeight");

                Ok(RpcResponse::Generic(format!("Generic response")))
            }
            RpcHeader::GetTx => {
                debug!("rpc message received in RpcHeader::GetTx");

                Ok(RpcResponse::Generic(format!("Generic response")))
            }
            RpcHeader::NewTx => {
                debug!("rpc message received in handler at RpcHeader::NewTx");

                match new_tx(&rpc, self.mem_pool.clone()) {
                    Ok(tx) => Ok(RpcResponse::Transaction(tx)),
                    Err(msg) => Ok(RpcResponse::Generic(msg.to_string())),
                }
            }
            RpcHeader::GetBlockHeader => {
                debug!("rpc message received in handler at RpcHeader::GetBlockHeader");
                match get_block_header(&rpc, self.chain.clone()) {
                    Ok(header) => Ok(RpcResponse::Header(header.clone())),
                    Err(msg) => Ok(RpcResponse::Generic(msg.to_string())),
                }
            }
            _ => Ok(RpcResponse::Generic(
                "unknown RPC header requested".to_string(),
            )),
        }
    }
}
