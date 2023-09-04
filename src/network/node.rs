use core::time;
use std::{
    sync::{
        mpsc::{channel, Receiver, Sender},
        Arc, Mutex,
    },
    thread,
    time::{Duration, Instant, SystemTime},
};

use log::info;

use crate::{
    core::{
        block::Block,
        header::{random_header, Header},
        transaction::Transaction,
    },
    crypto::{private_key::PrivateKey, utils::random_hash},
};

use super::{
    error::NetworkError,
    rpc::RPC,
    transport::{
        ArcMut, HttpTransport, LocalTransport, NetAddr, Payload, Transport, TransportManager,
    },
    tx_pool::TxPool,
};

pub struct NodeConfig<T>
where
    T: Transport,
{
    pub ts_manager: TransportManager<T>,
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

pub struct ChainNode<T>
where
    T: Transport,
{
    transport_manager: ArcMut<TransportManager<T>>,
    rx: ArcMut<Receiver<RPC>>,
    tx: ArcMut<Sender<RPC>>,
    block_time: time::Duration,
    mem_pool: Arc<Mutex<TxPool>>,
    miner: Arc<Mutex<BlockMiner>>,
}

impl ChainNode<LocalTransport> {
    pub fn new(config: NodeConfig<LocalTransport>) -> Self {
        let (tx, rx) = channel::<RPC>();
        let (tx, rx) = (ArcMut::new(tx), ArcMut::new(rx));
        let ts_manager = ArcMut::new(config.ts_manager);

        Self {
            transport_manager: ts_manager,
            rx,
            tx,
            block_time: config.block_time,
            mem_pool: Arc::new(Mutex::new(TxPool::new())),
            miner: Arc::new(Mutex::new(BlockMiner::new())),
        }
    }

    pub fn send_msg(
        &self,
        from_addr: NetAddr,
        to_addr: NetAddr,
        payload: Payload,
    ) -> Result<(), NetworkError> {
        if let Ok(ts_manager) = self.transport_manager.lock() {
            ts_manager.send_msg(from_addr, to_addr, payload)?
        }

        Ok(())
    }

    pub fn start(&mut self) {
        let ts_manager = self.transport_manager.clone();
        let tx = self.tx.clone();
        let rx = self.rx.clone();

        let miner = self.miner.clone();

        let block_time = self.block_time;

        if let Ok(ts_manager) = ts_manager.lock().as_mut() {
            ts_manager
                .init(tx)
                .expect("unable to initialize transport manager");
        }

        let mem_pool = self.mem_pool.clone();
        // Spawn thread to handle message, main RPC handler thread
        thread::spawn(move || {
            if let Ok(rx) = rx.lock() {
                for msg in rx.iter() {
                    info!("MESSAGE: from: {} - to: {}", msg.sender, msg.receiver);

                    // check if msg is transaction
                    let tx = Transaction::new(&msg.payload);
                    if let Ok(mut mem_pool) = mem_pool.lock() {
                        // add transaction to mem pool
                        mem_pool.add(tx)

                        // if ok then broadcast transaction to all peers
                    }
                }
            }
        });

        let mem_pool = self.mem_pool.clone();
        thread::spawn(move || loop {
            thread::sleep(block_time);
            // check is server has miner
            // miner takes transactions from mem pool on each block duration
            if let Ok(mut miner) = miner.lock() {
                if let Ok(mut pool) = mem_pool.lock() {
                    let txs = pool.take(2);

                    let header = random_header(1, random_hash());

                    if !txs.is_empty() {
                        // get block from miner
                        miner.mine_block(header, txs);

                        // add block to blockchain

                        // broadcast added block

                        // update last block time
                        miner.last_block_time = Instant::now();
                    }
                }
            }
        });
    }
}
