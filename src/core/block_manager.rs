use std::collections::HashMap;
use std::error::Error;
use std::fs::{File, OpenOptions};
use std::io::Read;
use std::io::Write;
use std::path::PathBuf;

use borsh::{BorshDeserialize, BorshSerialize};
use log::{error, info, warn};
use rocksdb::IteratorMode;
use serde::{Deserialize, Serialize};

use crate::crypto::public_key::PublicKeyBytes;
use crate::crypto::signature::SignatureBytes;
use crate::crypto::{
    hash::Hash, private_key::PrivateKey, public_key::PublicKey, signature::Signature,
};

use super::block::Block;
use super::storage::DbBlockStorage;
use super::{
    encoding::{ByteEncoding, HexEncoding},
    error::CoreError,
    header::Header,
    storage::{BlockStorage, MemoryBlockStorage},
    transaction::Transaction,
};

pub struct BlockManager {
    blocks: Vec<Block>,
    store: Box<dyn BlockStorage>,
}

impl BlockManager {
    pub fn new(storage_path: &str) -> Self {
        Self {
            blocks: vec![],
            store: DbBlockStorage::new_boxed(storage_path),
        }
    }

    pub fn headers(&self) -> Vec<&Header> {
        let mut headers = vec![];
        for block in &self.blocks {
            headers.push(&block.header);
        }
        headers
    }

    pub fn blocks(&self) -> Vec<&Block> {
        let mut blocks = vec![];
        for block in &self.blocks {
            blocks.push(block);
        }
        blocks
    }

    pub fn add(&mut self, block: Block) -> Result<(), CoreError> {
        info!(
            "adding block to chain with height: {}, and hash: {}",
            &block.height(),
            &block.hash().to_string()
        );
        self.store.put(&block)
    }

    pub fn get_block_by_height(&self, height: usize) -> Option<Block> {
        let hash = match self.store.height_to_hash(height) {
            Some(hash) => hash,
            None => {
                warn!("unable to get block hash from height:{height}");
                return None;
            }
        };

        match self.store.get(&hash) {
            Ok(b) => Some(b),
            Err(_) => {
                warn!("unable to get block by hash: {hash}");
                None
            }
        }
    }

    pub fn get_block_by_hash(&self, hash: &str) -> Option<Block> {
        match self.store.get(hash) {
            Ok(b) => Some(b),
            Err(_) => None,
        }
    }

    pub fn get_header_by_height(&self, height: usize) -> Option<Header> {
        match self.get_block_by_height(height) {
            Some(block) => Some(block.header().clone()),
            None => None,
        }
    }
    pub fn get_header_by_hash(&self, hash: &str) -> Option<Header> {
        match self.get_block_by_hash(hash) {
            Some(block) => Some(block.header().clone()),
            None => None,
        }
    }

    pub fn last(&self) -> Option<Block> {
        match self.store.last_block_height() {
            Some(height) => self.get_block_by_height(height),
            None => {
                error!("store.last_block_height is None");
                None
            }
        }
    }

    pub fn has_block(&self, height: usize) -> bool {
        height <= self.height()
    }

    pub fn height(&self) -> usize {
        match self.store.last_block_height() {
            Some(height) => height,
            None => 0,
        }
    }

    // ---
    // Private Methods
    // ---

    pub fn new_in_memory() -> Self {
        Self {
            blocks: vec![],
            store: MemoryBlockStorage::new_boxed(),
        }
    }
}

impl Default for BlockManager {
    fn default() -> Self {
        Self::new("data/chain.db")
    }
}

#[cfg(test)]
mod tests {

    use crate::core::block::random_block;
    use crate::crypto::utils::random_hash;

    use crate::core::header::random_header;

    use super::*;

    #[test]
    fn test_header_manager() {
        let mut manager = BlockManager::new_in_memory();

        for _ in 0..5 {
            let header = random_header(1, random_hash());
            let block = random_block(header);

            manager.add(block).unwrap();
        }

        let headers = manager.headers();
        let blocks = manager.blocks();

        assert_eq!(headers.len(), blocks.len());
    }

    // TODO: Test block manager with DbBlockStorage
}
