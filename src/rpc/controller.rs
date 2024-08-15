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
    rpc::handlers::{
        block::{get_block, get_block_header, get_last_block},
        transaction::new_tx,
    },
};

use crate::network::{
    error::NetworkError, miner::BlockMiner, tcp::TcpController, tx_pool::TxPool, types::Payload,
};

use crate::rpc::types::{RpcHandlerResponse, RpcHeader, RPC};

pub struct RpcController {
    mem_pool: Arc<Mutex<TxPool>>,
    _miner: Arc<Mutex<BlockMiner>>,
    chain: Arc<Mutex<Blockchain>>,
    _tcp_controller: Arc<Mutex<TcpController>>,
}

impl RpcController {
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
        match rpc.header {
            RpcHeader::GetBlock => {
                debug!("rpc message received in handler at RpcHeader::GetBlock");
                match get_block(&rpc, self.chain.clone()) {
                    Ok(block) => Ok(RpcHandlerResponse::Block(block)),
                    Err(msg) => Ok(RpcHandlerResponse::Generic(msg.to_string())),
                }
            }
            RpcHeader::GetLastBlock => {
                debug!("rpc message received in handler at RpcHeader::GetLastBlock");

                match get_last_block(&rpc, self.chain.clone()) {
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

                match new_tx(&rpc, self.mem_pool.clone()) {
                    Ok(tx) => Ok(RpcHandlerResponse::Transaction(tx)),
                    Err(msg) => Ok(RpcHandlerResponse::Generic(msg.to_string())),
                }
            }
            RpcHeader::GetBlockHeader => {
                debug!("rpc message received in handler at RpcHeader::GetBlockHeader");
                match get_block_header(&rpc, self.chain.clone()) {
                    Ok(header) => Ok(RpcHandlerResponse::Header(header.clone())),
                    Err(msg) => Ok(RpcHandlerResponse::Generic(msg.to_string())),
                }
            }
            _ => Ok(RpcHandlerResponse::Generic(
                "unknown RPC header requested".to_string(),
            )),
        }
    }
}
