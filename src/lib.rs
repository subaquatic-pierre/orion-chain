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
pub mod rpc;
pub mod state;
pub mod util;
pub mod vm;

use std::{
    net::SocketAddr,
    path::Path,
    sync::{mpsc::Sender, Arc, Mutex, Once},
    thread, time,
};

use crate::core::transaction::random_signed_tx;
use crate::core::{block::random_block, blockchain::Blockchain, header::random_header};
use crate::network::{
    node::{ChainNode, NodeConfig},
    types::RpcChanMsg,
};
use crate::rpc::{
    controller::RpcController,
    types::{RpcHeader, RpcResponse, RPC},
};

use crate::core::encoding::ByteEncoding;

pub type GenericError = Box<dyn std::error::Error + Send + Sync>;
pub type Result<T> = std::result::Result<T, GenericError>;

pub fn build_full_node() -> Result<ChainNode> {
    let config = NodeConfig::default();

    Ok(ChainNode::new(config))
}

static INIT: Once = Once::new();

/// Setup function that is only run once, even if called multiple times.
pub fn logger_init() {
    INIT.call_once(|| {
        pretty_env_logger::init();
        // env_logger::init();
    });
}

pub fn transaction_tester_thread(handler: Arc<Mutex<RpcController>>) {
    thread::spawn(move || loop {
        // TODO: Remove this thread, only used to add
        // transactions every 2 seconds for testing
        thread::sleep(time::Duration::from_secs(2));

        let tx = random_signed_tx();

        let rpc = RPC {
            header: RpcHeader::NewTx,
            // TODO: Error handling on byte encoding
            payload: tx.to_bytes().unwrap(),
        };

        if let Ok(handler) = handler.lock() {
            if let Ok(res) = handler.handle_client_rpc(&rpc) {
                match res {
                    RpcResponse::Generic(msg) => {
                        warn!("incorrect generic response from RpcController: {msg}");
                    }
                    RpcResponse::Transaction(tx) => {
                        // info!("transaction successfully received from RpcController");
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
