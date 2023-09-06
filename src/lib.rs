#![allow(clippy::needless_range_loop)]
#![allow(clippy::new_without_default)]
#![allow(clippy::all)]

use crypto::private_key::PrivateKey;
use crypto::utils::random_hash;
use log::{info, trace, warn};
use rand::Rng;
use std::thread::JoinHandle;

pub mod api;
pub mod core;
pub mod crypto;
pub mod network;
pub mod util;

use std::{
    error::Error,
    sync::{Arc, Once},
    thread, time,
};

use crate::core::{block::random_block, blockchain::Blockchain, header::random_header};

use network::{
    node::{ChainNode, NodeConfig},
    transport::{LocalTransport, Transport, TransportManager},
    types::ArcMut,
};

pub type GenericError = Box<dyn std::error::Error + Send + Sync>;
pub type Result<T> = std::result::Result<T, GenericError>;

pub fn build_full_node() -> Result<ChainNode> {
    let ts1 = LocalTransport::new("local");
    let ts2 = LocalTransport::new("custom");
    let ts3 = LocalTransport::new("remote");
    let mut ts_manager = TransportManager::new();

    ts_manager.connect(ts1)?;
    ts_manager.connect(ts2)?;
    ts_manager.connect(ts3)?;

    let config = NodeConfig {
        ts_manager,
        block_time: time::Duration::from_secs(5),
        private_key: Some(PrivateKey::new()),
    };

    let block = random_block(random_header(0, random_hash()));
    let chain = Blockchain::new_with_genesis(block);

    Ok(ChainNode::new(config, chain))
}

pub fn send_tx_loop(mut server: ChainNode) -> JoinHandle<()> {
    struct Ticker {
        pub val: i32,
    }

    impl Ticker {
        pub fn inc(&mut self) {
            self.val += 1
        }
    }

    let ticker = ArcMut::new(Ticker { val: 0 });

    // simulate sending messages thread
    let handle = thread::spawn(move || {
        server.start();
        loop {
            let addr = match ticker.lock() {
                Ok(tick) => {
                    if tick.val % 4 == 0 {
                        "local".to_string()
                    } else if tick.val % 3 == 1 {
                        "remote".to_string()
                    } else if tick.val % 3 == 2 {
                        "custom".to_string()
                    } else {
                        "street".to_string()
                    }
                }
                _ => "local".to_string(),
            };

            let random_number: Vec<u8> = (0..1024).map(|_| rand::random::<u8>()).collect();

            server
                .send_rpc(addr, "remote".to_string(), random_number)
                .ok();
            thread::sleep(time::Duration::from_secs(1));

            if let Ok(ticker) = ticker.lock().as_mut() {
                ticker.inc()
            }
        }
    });

    handle
}

static INIT: Once = Once::new();

/// Setup function that is only run once, even if called multiple times.
pub fn logger_init() {
    INIT.call_once(|| {
        env_logger::init();
    });
}
