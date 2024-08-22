use core::time;
use std::sync::{Arc, Mutex, MutexGuard};
use std::time::Instant;

use log::{error, info, warn};

use crate::core::blockchain::Blockchain;
use crate::core::error::CoreError;

use crate::core::header::random_header;
use crate::lock;
use crate::network::types::ArcMut;
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
    chain: Arc<Mutex<Blockchain>>,
    private_key: PrivateKey,
    pub pool_size: usize,
}

impl Validator {
    pub fn new(chain: Arc<Mutex<Blockchain>>, private_key: PrivateKey, pool_size: usize) -> Self {
        Self {
            chain,
            private_key,
            pool_size,
        }
    }

    pub fn validate_block(&self, block: &Block) -> Result<(), CoreError> {
        let chain = self.chain();
        if chain.has_block(block.height()) {
            return Err(CoreError::Block(
                "blockchain already contains block".to_string(),
            ));
        }

        if block.height() != chain.height() + 1 {
            return Err(CoreError::Block(
                "block is to high too be added".to_string(),
            ));
        }

        let last_block = match chain.last_block() {
            Some(last_block) => last_block,
            None => return Err(CoreError::Block("incorrect header height".to_string())),
        };

        // check correct prev hash
        let cur_header = block.header();

        if cur_header.prev_hash() != last_block.header().hash().clone() {
            return Err(CoreError::Block("incorrect previous hash".to_string()));
        }

        block.verify()
    }

    pub fn propose_block(&self, txs: Vec<Transaction>) -> Result<Block, CoreError> {
        let chain = self.chain();
        let last_block = chain.last_block().ok_or(CoreError::Block(
            "unable to get last block from chain".to_string(),
        ))?;

        let last_header = last_block.header();
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
            "create new block in Validator {:}, num txs: {}, with height: {}",
            block.header().hash(),
            block.num_txs(),
            block.height()
        );

        if let Err(e) = block.sign(&self.private_key) {
            warn!("unable to sign block in miner: {e}")
        }

        Ok(block)
    }

    // ---
    // Private Methods
    // ---
    fn chain(&self) -> MutexGuard<Blockchain> {
        lock!(self.chain)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::block::Block;
    use crate::core::blockchain::Blockchain;
    use crate::core::header::Header;
    use crate::core::transaction::{random_signed_tx, Transaction};
    use crate::crypto::hash::Hash;
    use crate::crypto::private_key::{self, PrivateKey};
    use std::sync::{Arc, Mutex};
    use std::time::Duration;

    fn setup_blockchain() -> Arc<Mutex<Blockchain>> {
        let chain = Blockchain::new_with_genesis_in_memory().unwrap();
        Arc::new(Mutex::new(chain))
    }

    #[test]
    fn test_validate_block_success() {
        let blockchain = setup_blockchain();
        let private_key = PrivateKey::new();
        let validator = Validator::new(blockchain.clone(), private_key.clone(), 10);

        let txs = vec![random_signed_tx()];
        let block = validator.propose_block(txs).unwrap();

        let result = validator.validate_block(&block);
        assert!(result.is_ok(), "Block should be valid");
    }

    #[test]
    fn test_validate_block_failure_duplicate() {
        let blockchain = setup_blockchain();
        let private_key = PrivateKey::new();
        let validator = Validator::new(blockchain.clone(), private_key.clone(), 10);

        let txs = vec![random_signed_tx()];
        let block = validator.propose_block(txs.clone()).unwrap();

        let _ = blockchain.lock().unwrap().add_block(block.clone());

        let result = validator.validate_block(&block);
        assert!(result.is_err(), "Block should be rejected as duplicate");
    }

    #[test]
    fn test_propose_block_success() {
        let blockchain = setup_blockchain();
        let private_key = PrivateKey::new();
        let validator = Validator::new(blockchain.clone(), private_key.clone(), 10);

        let txs = vec![random_signed_tx()];
        let result = validator.propose_block(txs);
        assert!(result.is_ok(), "Block should be proposed successfully");

        let block = result.unwrap();
        assert_eq!(block.height(), 1, "Block height should be 1");
    }

    #[test]
    fn test_propose_block_with_signature() {
        let blockchain = setup_blockchain();
        let private_key = PrivateKey::new();
        let validator = Validator::new(blockchain.clone(), private_key.clone(), 10);

        let txs = vec![random_signed_tx()];
        let result = validator.propose_block(txs);
        assert!(result.is_ok(), "Block should be proposed successfully");

        let block = result.unwrap();
        assert!(block.verify().is_ok(), "Block signature should be valid");
    }
}
