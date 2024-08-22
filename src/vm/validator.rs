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
        let prev_blockhash = last_header.hash().clone();
        let poh = Header::gen_poh(&txs)?;
        let tx_root = Header::gen_tx_root(&txs)?;

        // TODO: get actual state root
        let state_root = Header::gen_state_root()?;

        let blockhash = Header::gen_blockhash(height, prev_blockhash, poh, tx_root, state_root)?;

        let header = Header::new(height, blockhash, poh, tx_root, state_root, prev_blockhash);
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
