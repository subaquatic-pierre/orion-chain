use core::time;
use std::time::Instant;

use log::{error, info, warn};

use crate::core::error::CoreError;

use crate::core::header::random_header;
use crate::{
    core::{block::Block, header::Header, transaction::Transaction},
    crypto::private_key::PrivateKey,
    GenericError,
};

pub struct NodeConfig {
    pub block_time: time::Duration,
    pub private_key: Option<PrivateKey>,
}

pub struct Validator {
    pub last_block_time: Instant,
    private_key: PrivateKey,
    pub pool_size: usize,
}

impl Validator {
    pub fn new(private_key: PrivateKey, pool_size: usize) -> Self {
        Self {
            last_block_time: Instant::now(),
            private_key,
            pool_size,
        }
    }

    pub fn validate_block(
        &self,
        last_header: &Header,
        txs: Vec<Transaction>,
    ) -> Result<Block, CoreError> {
        let height = last_header.height() + 1;
        let prev_hash = last_header.hash().clone();
        let hash = Block::gen_blockhash(height, &txs).unwrap();

        let header = random_header(height, prev_hash);
        let mut block = Block::new(header, txs)?;
        info!(
            "create new block in MINER {:}, num txs: {}, with height: {}",
            block.header().hash(),
            block.num_txs(),
            block.height()
        );

        if let Err(e) = block.sign(&self.private_key) {
            warn!("unable to sign block in miner: {e}")
        }
        Ok(block)
    }
}
