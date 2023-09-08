use core::time;
use std::{
    net::{IpAddr, Ipv4Addr, SocketAddr},
    sync::{
        mpsc::{channel, Receiver, Sender},
        Arc, Mutex,
    },
    thread,
    time::Instant,
    vec,
};

use log::{error, info};

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
}

impl BlockMiner {
    pub fn new() -> Self {
        Self {
            last_block_time: Instant::now(),
        }
    }

    pub fn mine_block(&self, header: Header, txs: Vec<Transaction>) -> Block {
        // todo!()
        let block = Block::new(header, txs);
        info!(
            "create new block in MINER {:}, num txs: {}, with height: {}",
            block.hash,
            block.num_txs(),
            block.height()
        );
        // for &tx in txs {
        //     block.add_transaction(tx).unwrap();
        // }

        block
    }
}

pub struct ChainNode {
    tcp_controller: ArcMut<TcpController>,
    rpc_rx: ArcMut<Receiver<RpcChanMsg>>,
    _rpc_tx: ArcMut<Sender<RpcChanMsg>>,
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
        let addr = IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1));
        let tcp_controller =
            TcpController::new(SocketAddr::new(addr, 5000), rpc_tx.clone()).unwrap();

        let tcp_controller = ArcMut::new(tcp_controller);
        let miner = ArcMut::new(BlockMiner::new());
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
            _rpc_tx: rpc_tx,
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
    pub fn send_rpc(&self, _from_addr: NetAddr, payload: Payload) -> Result<(), NetworkError> {
        let tcp = lock!(self.tcp_controller);
        let rpc = RPC {
            header: RpcHeader::GetBlock,
            payload,
        };
        tcp.send_rpc(rpc);

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

    // ---
    // Private Methods
    // ---
    fn spawn_rpc_thread(&self) {
        let rpc_rx = self.rpc_rx.clone();
        let handler = self.rpc_handler.clone();

        // Spawn thread to handle message, main RPC handler thread
        thread::spawn(move || {
            let rpc_rx = lock!(rpc_rx);
            for (_, rpc) in rpc_rx.iter() {
                let handler = lock!(handler);

                if let Err(e) = handler.handle_rpc(&rpc) {
                    error!("{e}");
                }
            }
        });
    }

    fn spawn_miner_thread(&self) {
        let block_time = self.block_time;
        let miner = self.miner.clone();
        let mem_pool = self.mem_pool.clone();

        // TODO: check if ChainNode has block miner
        // spawn mining thread if exists
        thread::spawn(move || {
            loop {
                thread::sleep(block_time);
                // check is server has miner
                // miner takes transactions from mem pool on each block duration
                let mut miner = lock!(miner);
                if let Ok(mut pool) = mem_pool.lock() {
                    let txs = pool.take(2);

                    let header = random_header(1, random_hash());

                    // if !txs.is_empty() {
                    // get block from miner
                    miner.mine_block(header, txs);

                    // add block to blockchain

                    // broadcast added block

                    // update last block time
                    miner.last_block_time = Instant::now();
                    // }
                }
            }
        });
    }
}
