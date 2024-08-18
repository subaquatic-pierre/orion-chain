use std::sync::{Arc, Mutex};

use log::debug;

use crate::{
    api::routes::block::GetBlockReq,
    core::{
        block::Block, blockchain::Blockchain, encoding::ByteEncoding, header::Header,
        transaction::Transaction,
    },
    crypto::private_key::PrivateKey,
    lock,
    network::{error::NetworkError, node::ChainNode, tx_pool::TxPool, types::ArcMut},
    rpc::types::RPC,
};

pub fn get_block(rpc: &RPC, chain: Arc<Mutex<Blockchain>>) -> Result<Block, NetworkError> {
    let req: GetBlockReq = match bincode::deserialize(&rpc.payload) {
        Ok(req) => req,
        Err(e) => return Err(NetworkError::Decoding(e.to_string())),
    };

    let chain = lock!(chain);

    if req.hash.is_none() && req.height.is_none() {
        return Err(NetworkError::RPC(format!(
            "Incorrect request, must request with height or hash"
        )));
    }

    let block = if let Some(height) = &req.height {
        let block_height = match height.parse::<usize>() {
            Ok(height) => height,
            Err(e) => return Err(NetworkError::Decoding(e.to_string())),
        };

        chain.get_block_by_height(block_height)
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

pub fn get_block_header(rpc: &RPC, chain: Arc<Mutex<Blockchain>>) -> Result<Header, NetworkError> {
    match get_block(rpc, chain) {
        Ok(block) => return Ok(block.header().clone()),
        Err(msg) => Err(NetworkError::RPC(msg.to_string())),
    }
}

pub fn get_last_block(_rpc: &RPC, chain: Arc<Mutex<Blockchain>>) -> Result<Block, NetworkError> {
    let chain = lock!(chain);

    let block = chain.last_block();

    if let Some(block) = block {
        Ok(block.clone())
    } else {
        Err(NetworkError::RPC(format!("Last block not found")))
    }
}
