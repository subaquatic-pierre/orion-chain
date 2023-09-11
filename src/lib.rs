#![allow(clippy::needless_range_loop)]
#![allow(clippy::new_without_default)]
#![allow(clippy::all)]

use crypto::private_key::PrivateKey;
use crypto::utils::random_hash;
use log::{info, warn};

pub mod api;
pub mod core;
pub mod crypto;
pub mod network;
pub mod util;

use std::{
    net::SocketAddr,
    sync::{mpsc::Sender, Arc, Mutex, Once},
    thread, time,
};

use crate::core::transaction::random_signed_tx;
use crate::core::{block::random_block, blockchain::Blockchain, header::random_header};
use crate::{core::encoding::ByteEncoding, network::rpc::RpcHandlerResponse};

use network::{
    node::{ChainNode, NodeConfig},
    rpc::{RpcHandler, RpcHeader, RPC},
    types::RpcChanMsg,
};

pub type GenericError = Box<dyn std::error::Error + Send + Sync>;
pub type Result<T> = std::result::Result<T, GenericError>;

pub fn build_full_node() -> Result<ChainNode> {
    let config = NodeConfig {
        block_time: time::Duration::from_secs(5),
        private_key: Some(PrivateKey::new()),
    };

    let block = random_block(random_header(0, random_hash()));
    let chain = Blockchain::new_with_genesis(block);

    Ok(ChainNode::new(config, chain))
}

static INIT: Once = Once::new();

/// Setup function that is only run once, even if called multiple times.
pub fn logger_init() {
    INIT.call_once(|| {
        pretty_env_logger::init();
        // env_logger::init();
    });
}

pub fn transaction_tester_thread(handler: Arc<Mutex<RpcHandler>>) {
    thread::spawn(move || loop {
        // TODO: Remove this thread, only used to add
        // transactions every 2 seconds for testing
        thread::sleep(time::Duration::from_secs(2));

        let tx = random_signed_tx();

        let rpc = RPC {
            header: RpcHeader::NewTx,
            payload: tx.to_bytes(),
        };

        if let Ok(handler) = handler.lock() {
            if let Ok(res) = handler.handle_client_rpc(&rpc) {
                match res {
                    RpcHandlerResponse::Generic(msg) => {
                        warn!("incorrect generic response from RpcHandler: {msg}");
                    }
                    RpcHandlerResponse::Transaction(tx) => {
                        // info!("transaction successfully received from RpcHandler");
                    }
                    _ => {
                        warn!("unable to handle rpc in transaction_tester_thread");
                    }
                }
            } else {
                warn!("unable to handle rpc in transaction_tester_thread");
            }
        } else {
            warn!("unable to lock handler in transaction_tester_thread");
        }
    });
}
