use core::time;
use std::{
    net::SocketAddr,
    path::{Path, PathBuf},
    sync::{
        mpsc::{channel, Receiver, Sender},
        Arc, Mutex,
    },
    thread,
    time::Instant,
    vec,
};

use log::{error, info, warn};

use crate::{
    core::{block::random_block, error::CoreError},
    crypto::hash::Hash,
    lock,
};

use crate::rpc::{
    controller::RpcController,
    types::{RpcHeader, RpcResponse, RPC},
};

use crate::{
    core::{
        block::Block,
        blockchain::Blockchain,
        header::{random_header, Header},
        transaction::Transaction,
    },
    crypto::{private_key::PrivateKey, utils::random_hash},
    vm::validator::Validator,
    GenericError,
};

use super::{
    error::NetworkError,
    tx_pool::TxPool,
    types::{Payload, RpcChanMsg},
};
use super::{tcp::TcpController, types::ArcMut};

pub struct NodeConfig {
    pub block_time: time::Duration,
    pub private_key: PrivateKey,
    pub state_storage_path: PathBuf,
    pub chain_storage_path: PathBuf,
}

impl Default for NodeConfig {
    fn default() -> Self {
        NodeConfig {
            block_time: time::Duration::from_secs(5),
            private_key: PrivateKey::from_pem(Path::new("data/private_key.pem")).unwrap(),
            state_storage_path: Path::new("data/state.db").to_owned(),
            chain_storage_path: Path::new("data/chain.db").to_owned(),
        }
    }
}

pub struct ChainNode {
    config: NodeConfig,
    tcp_controller: ArcMut<TcpController>,
    rpc_rx: ArcMut<Receiver<RpcChanMsg>>,
    rpc_tx: ArcMut<Sender<RpcChanMsg>>,
    mem_pool: ArcMut<TxPool>,
    validator: ArcMut<Validator>,
    pub chain: ArcMut<Blockchain>,
    rpc_controller: Arc<RpcController>,
}

impl ChainNode {
    pub fn new(config: NodeConfig) -> Self {
        // TODO: start chain from config
        let chain = Blockchain::new_with_genesis().unwrap();

        // TODO: clear storage if config requires fresh state eg. during dev

        // TODO: create helper function to build ArcMut chanel
        let (tx, rx) = channel::<RpcChanMsg>();
        let (rpc_tx, rpc_rx) = (ArcMut::new(tx), ArcMut::new(rx));

        // TODO: CONFIG, get listener address from config
        let addr: SocketAddr = "127.0.0.1:5000".parse().unwrap();
        let tcp_controller = TcpController::new(addr, rpc_tx.clone()).unwrap();

        let tcp_controller = ArcMut::new(tcp_controller);

        // TODO: get private key from config
        // TODO: get pool size from config
        let validator = ArcMut::new(Validator::new(config.private_key.clone(), 50));

        let mem_pool = ArcMut::new(TxPool::new());
        let chain = ArcMut::new(chain);

        let rpc_controller = RpcController::new(
            mem_pool.clone(),
            validator.clone(),
            chain.clone(),
            tcp_controller.clone(),
        );
        let rpc_controller = Arc::new(rpc_controller);

        Self {
            config,
            rpc_rx,
            rpc_tx,
            mem_pool,
            validator,
            chain,
            tcp_controller,
            rpc_controller,
        }
    }

    // Proxy method for TCP Controller
    // calls TcpController.send_rpc()
    pub fn send_rpc(&self, peer_addr: SocketAddr, payload: Payload) -> Result<(), NetworkError> {
        let tcp = lock!(self.tcp_controller);
        let rpc = RPC {
            // TODO: get header from args
            header: RpcHeader::GetBlock,
            payload,
        };
        tcp.send_rpc(peer_addr, &rpc);

        Ok(())
    }

    pub fn start(&mut self) -> Result<(), GenericError> {
        // Start TcpController
        // launches all threads need to communicate with peers
        // all messages received from peers are send back on self.rpc_tx
        // chanel which is handled by RpcController struct withing api module
        let tcp_controller = self.tcp_controller.clone();
        let mut tcp = lock!(tcp_controller);
        // TODO: get peer addresses from config
        tcp.start(vec![]);

        // Start thread to listen for all incoming RPC
        // messages from peers
        self.spawn_peer_rpc_thread();

        // Spawn validator thread if ChainNode is validator
        // TODO: Check if is full node in config, if not full node then validator is not needed
        self.spawn_validator_thread();

        Ok(())
    }

    // Get the a ArcMut of RPC handler
    pub fn rpc_controller(&self) -> Arc<RpcController> {
        self.rpc_controller.clone()
    }

    pub fn rpc_tx(&self) -> Arc<Mutex<Sender<RpcChanMsg>>> {
        self.rpc_tx.clone()
    }

    // ---
    // Private Methods
    // ---
    // Main thread that listens for RPC messages from peers,
    // messages are then handled by rpc_controller
    fn spawn_peer_rpc_thread(&self) {
        let rpc_rx = self.rpc_rx.clone();
        let handler = self.rpc_controller();

        // Spawn thread to handle message, main RPC handler thread
        thread::spawn(move || {
            let rpc_rx = lock!(rpc_rx);
            for (peer_addr, rpc) in rpc_rx.iter() {
                if let Err(e) = handler.handle_rpc(&rpc, Some(peer_addr)) {
                    error!("{e}");
                }
            }
        });
    }

    // TODO: change validator to VM
    fn spawn_validator_thread(&self) {
        let block_time = self.config.block_time;
        let validator = self.validator.clone();
        let mem_pool = self.mem_pool.clone();
        let chain = self.chain.clone();

        // TODO: check if ChainNode has block validator
        // spawn mining thread if exists
        thread::spawn(move || {
            loop {
                thread::sleep(block_time);
                // TODO: check is validator is current leader

                // TODO: check is node is full validator node or just slim node

                let mut validator = lock!(validator);
                if let Ok(mut pool) = mem_pool.lock() {
                    // validator takes transactions from mem pool on each block duration
                    let txs = pool.take(validator.pool_size);

                    if let Ok(mut chain) = chain.lock() {
                        if let Some(last_block) = chain.last_block() {
                            if let Ok(block) = validator.validate_block(last_block.header(), txs) {
                                // TODO: propose block to network
                                // broadcast added block
                                // once block is confirmed by majority voting
                                // adding block to chain is handled by RPC Controller
                                if let Err(e) = chain.add_block(block) {
                                    warn!(
                                        "unable to add block in Node::spawn_validator_thread: {e}"
                                    );
                                }
                            }
                        } else {
                            warn!("unable to get last block chain in Node::spawn_validator_thread");
                        }
                    } else {
                        warn!("unable to lock chain in Node::spawn_validator_thread");
                    }

                    // update last block time
                    validator.last_block_time = Instant::now();
                }
            }
        });
    }
}
