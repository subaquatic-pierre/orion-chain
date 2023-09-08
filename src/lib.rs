#![allow(clippy::needless_range_loop)]
#![allow(clippy::new_without_default)]
#![allow(clippy::all)]

use crypto::private_key::PrivateKey;
use crypto::utils::random_hash;

pub mod api;
pub mod core;
pub mod crypto;
pub mod network;
pub mod util;

use std::{sync::Once, time};

use crate::core::{block::random_block, blockchain::Blockchain, header::random_header};

use network::node::{ChainNode, NodeConfig};

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
