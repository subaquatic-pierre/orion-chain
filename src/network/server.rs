use core::time;
use std::{
    sync::{
        mpsc::{channel, Receiver, Sender},
        Arc, Mutex,
    },
    thread,
    time::{Duration, Instant, SystemTime},
};

use crate::{
    core::{
        block::Block,
        header::{random_header, Header},
        transaction::Transaction,
    },
    crypto::{private_key::PrivateKey, utils::random_hash},
};

use super::{
    transport::{ArcMut, HttpTransport, LocalTransport, Transport, TransportManager, RPC},
    tx_pool::TxPool,
};

pub struct ServerConfig<T>
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

    pub fn mine_block<'a>(&self, header: &'a Header, txs: Vec<Transaction>) -> Block<'a> {
        // todo!()
        let block = Block::new(header, txs);
        println!("create new block in MINER {:#?}", block);
        // for &tx in txs {
        //     block.add_transaction(tx).unwrap();
        // }

        block
    }
}

pub struct Server<T>
where
    T: Transport,
{
    pub transport_manager: ArcMut<TransportManager<T>>,
    rx: ArcMut<Receiver<RPC>>,
    tx: ArcMut<Sender<RPC>>,
    block_time: time::Duration,
    mem_pool: Arc<Mutex<TxPool>>,
    miner: Arc<Mutex<BlockMiner>>,
}

impl Server<LocalTransport> {
    pub fn new(config: ServerConfig<LocalTransport>) -> Self {
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

    pub fn start(&mut self) {
        let ts_manager = self.transport_manager.clone();
        let tx = self.tx.clone();
        let rx = self.rx.clone();

        let mem_pool = self.mem_pool.clone();
        let miner = self.miner.clone();

        let block_time = self.block_time;

        if let Ok(ts_manager) = ts_manager.lock().as_mut() {
            ts_manager
                .init(tx)
                .expect("unable to initialize transport manager");
        }

        // Spawn thread to handle message, main RPC handler thread
        thread::spawn(move || {
            if let Ok(rx) = rx.lock() {
                for msg in rx.iter() {
                    println!("{msg:#?}");

                    // check if msg is transaction
                    let tx = Transaction::new(&msg.payload);
                    if let Ok(mut mem_pool) = mem_pool.lock() {
                        // add transaction to mem pool
                        mem_pool.add(&tx)

                        // if ok then broadcast transaction to all peers
                    }

                    // check is server has miner
                    // miner takes transactions from mem pool on each block duration
                    if let Ok(mut miner) = miner.lock() {
                        let now = Instant::now();
                        let duration_delta = miner.last_block_time + block_time;
                        // check time delta
                        if now > duration_delta {
                            if let Ok(mut pool) = mem_pool.lock() {
                                let txs = pool.take(2);

                                let header = random_header(1, random_hash());

                                // get block from miner
                                miner.mine_block(&header, txs);

                                // add block to blockchain

                                // broadcast added block

                                // update last block time
                                miner.last_block_time = Instant::now();
                            }
                        }
                    }
                }
            }
        });
    }
}
