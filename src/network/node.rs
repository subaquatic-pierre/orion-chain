use core::time;
use std::{
    net::{IpAddr, Ipv4Addr, SocketAddr},
    sync::{
        mpsc::{channel, Receiver, Sender},
        Arc, Mutex,
    },
    thread,
    time::{Duration, Instant, SystemTime},
};

use log::info;
use serde::de::Error;

use crate::{
    core::{
        block::{random_block, Block},
        blockchain::Blockchain,
        header::{random_header, Header},
        transaction::Transaction,
    },
    crypto::{private_key::PrivateKey, utils::random_hash},
    GenericError,
};

use super::{
    error::NetworkError,
    rpc::RPC,
    transport::{HttpTransport, LocalTransport, NetAddr, Payload, Transport, TransportManager},
    tx_pool::TxPool,
};
use super::{tcp::TcpTransport, types::ArcMut};

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
    mem_pool: ArcMut<TxPool>,
    miner: ArcMut<BlockMiner>,
    pub chain: Arc<Blockchain>,
    tcp: ArcMut<TcpTransport>,
}

impl ChainNode<LocalTransport> {
    pub fn new(config: NodeConfig<LocalTransport>, chain: Blockchain) -> Self {
        let (tx, rx) = channel::<RPC>();
        let (tx, rx) = (ArcMut::new(tx), ArcMut::new(rx));
        let ts_manager = ArcMut::new(config.ts_manager);

        let addr = IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1));
        let tcp = TcpTransport::new(SocketAddr::new(addr, 5000), tx.clone(), rx.clone());

        Self {
            transport_manager: ts_manager,
            rx,
            tx,
            block_time: config.block_time,
            mem_pool: ArcMut::new(TxPool::new()),
            miner: ArcMut::new(BlockMiner::new()),
            chain: Arc::new(chain),
            tcp: ArcMut::new(tcp),
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

    pub fn start(&mut self) -> Result<(), GenericError> {
        let ts_manager = self.transport_manager.clone();
        let tx = self.tx.clone();

        let miner = self.miner.clone();

        let block_time = self.block_time;

        if let Ok(ts_manager) = ts_manager.lock().as_mut() {
            ts_manager
                .init(tx)
                .expect("unable to initialize transport manager");
        }

        self.tcp.lock().unwrap().init();

        let mem_pool = self.mem_pool.clone();
        let rx = self.rx.clone();
        // Spawn thread to handle message, main RPC handler thread
        thread::spawn(move || {
            if let Ok(rx) = rx.lock() {
                for msg in rx.iter() {
                    info!(
                        "MESSAGE: from: {} - to: {} with message: {}",
                        msg.sender,
                        msg.receiver,
                        String::from_utf8_lossy(&msg.payload)
                    );

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
        thread::spawn(move || {
            loop {
                thread::sleep(block_time);
                // check is server has miner
                // miner takes transactions from mem pool on each block duration
                if let Ok(mut miner) = miner.lock() {
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
            }
        });

        Ok(())
        // handle.await?
    }
}
