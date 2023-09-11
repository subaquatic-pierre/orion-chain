use core::time;
use std::{
    net::SocketAddr,
    sync::{
        mpsc::{channel, Receiver, Sender},
        Arc, Mutex,
    },
    thread,
    time::Instant,
    vec,
};

use log::{error, info, warn};

use crate::lock;

use crate::{
    core::{
        block::Block,
        blockchain::Blockchain,
        header::{random_header, Header},
        transaction::Transaction,
    },
    crypto::{private_key::PrivateKey, utils::random_hash},
    GenericError,
};

use super::{
    error::NetworkError,
    rpc::{RpcHandler, RpcHeader, RPC},
    transport::{NetAddr, Payload},
    tx_pool::TxPool,
    types::RpcChanMsg,
};
use super::{tcp::TcpController, types::ArcMut};

pub struct NodeConfig {
    pub block_time: time::Duration,
    pub private_key: Option<PrivateKey>,
}

pub struct BlockMiner {
    pub last_block_time: Instant,
    private_key: PrivateKey,
    pub pool_size: usize,
}

impl BlockMiner {
    pub fn new(private_key: PrivateKey, pool_size: usize) -> Self {
        Self {
            last_block_time: Instant::now(),
            private_key,
            pool_size,
        }
    }

    pub fn mine_block(&self, header: Header, txs: Vec<Transaction>) -> Block {
        let mut block = Block::new(header, txs);
        info!(
            "create new block in MINER {:}, num txs: {}, with height: {}",
            block.hash,
            block.num_txs(),
            block.height()
        );

        if let Err(e) = block.sign(&self.private_key) {
            warn!("unable to sign block in miner: {e}")
        }
        block
    }
}

pub struct ChainNode {
    tcp_controller: ArcMut<TcpController>,
    rpc_rx: ArcMut<Receiver<RpcChanMsg>>,
    rpc_tx: ArcMut<Sender<RpcChanMsg>>,
    block_time: time::Duration,
    mem_pool: ArcMut<TxPool>,
    miner: ArcMut<BlockMiner>,
    pub chain: ArcMut<Blockchain>,
    rpc_handler: ArcMut<RpcHandler>,
}

impl ChainNode {
    pub fn new(config: NodeConfig, chain: Blockchain) -> Self {
        // TODO: create helper function to build ArcMut chanel
        let (tx, rx) = channel::<RpcChanMsg>();
        let (rpc_tx, rpc_rx) = (ArcMut::new(tx), ArcMut::new(rx));

        // TODO: CONFIG, get listener address from config
        let addr: SocketAddr = "127.0.0.1:5000".parse().unwrap();
        let tcp_controller = TcpController::new(addr, rpc_tx.clone()).unwrap();

        let tcp_controller = ArcMut::new(tcp_controller);

        // TODO: get private key from config
        // TODO: get pool size from config
        let pk = PrivateKey::new();
        let miner = ArcMut::new(BlockMiner::new(pk, 50));

        let mem_pool = ArcMut::new(TxPool::new());
        let chain = ArcMut::new(chain);

        let rpc_handler = RpcHandler::new(
            mem_pool.clone(),
            miner.clone(),
            chain.clone(),
            tcp_controller.clone(),
        );
        let rpc_handler = ArcMut::new(rpc_handler);

        Self {
            rpc_rx,
            rpc_tx,
            block_time: config.block_time,
            mem_pool,
            miner,
            chain,
            tcp_controller,
            rpc_handler,
        }
    }

    // Proxy method for TCP Controller
    // calls TcpController.send_rpc()
    pub fn send_rpc(&self, peer_addr: SocketAddr, payload: Payload) -> Result<(), NetworkError> {
        let tcp = lock!(self.tcp_controller);
        let rpc = RPC {
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
        // chanel
        let mut tcp = lock!(self.tcp_controller);
        tcp.start(vec![]);

        // Start thread to listen for all incoming RPC
        // messages
        self.spawn_rpc_thread();

        // Spawn miner thread if ChainNode is miner
        self.spawn_miner_thread();

        Ok(())
    }

    // Get the a ArcMut of RPC handler
    pub fn rpc_handler(&self) -> Arc<Mutex<RpcHandler>> {
        self.rpc_handler.clone()
    }

    pub fn rpc_tx(&self) -> Arc<Mutex<Sender<RpcChanMsg>>> {
        self.rpc_tx.clone()
    }

    // ---
    // Private Methods
    // ---
    fn spawn_rpc_thread(&self) {
        let rpc_rx = self.rpc_rx.clone();
        let handler = self.rpc_handler.clone();

        // Spawn thread to handle message, main RPC handler thread
        thread::spawn(move || {
            let rpc_rx = lock!(rpc_rx);
            for (peer_addr, rpc) in rpc_rx.iter() {
                let handler = lock!(handler);

                if let Err(e) = handler.handle_peer_rpc(&rpc, peer_addr) {
                    error!("{e}");
                }
            }
        });
    }

    fn spawn_miner_thread(&self) {
        let block_time = self.block_time;
        let miner = self.miner.clone();
        let mem_pool = self.mem_pool.clone();
        let chain = self.chain.clone();

        // TODO: check if ChainNode has block miner
        // spawn mining thread if exists
        thread::spawn(move || {
            loop {
                thread::sleep(block_time);
                // check is server has miner
                // miner takes transactions from mem pool on each block duration
                let mut miner = lock!(miner);
                if let Ok(mut pool) = mem_pool.lock() {
                    let pool_size = miner.pool_size;
                    let txs = pool.take(pool_size);

                    if let Ok(mut chain) = chain.lock() {
                        if let Some(last_block) = chain.last_block() {
                            let height = chain.height() + 1;
                            let prev_hash = last_block.hash().clone();
                            let hash = Block::generate_block_hash(&txs);
                            let header = Header::new(height, hash, prev_hash);

                            // if !txs.is_empty() {
                            // get block from miner
                            let block = miner.mine_block(header, txs);

                            // add block to blockchain
                            if let Err(e) = chain.add_block(block) {
                                warn!("unable to add block in Node::spawn_miner_thread: {e}");
                            }
                        } else {
                            warn!("unable to get last block chain in Node::spawn_miner_thread");
                        }
                    } else {
                        warn!("unable to lock chain in Node::spawn_miner_thread");
                    }

                    // broadcast added block

                    // update last block time
                    miner.last_block_time = Instant::now();
                    // }
                }
            }
        });
    }
}
