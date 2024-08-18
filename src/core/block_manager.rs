use std::collections::HashMap;
use std::error::Error;
use std::fs::{File, OpenOptions};
use std::io::Read;
use std::io::Write;
use std::path::PathBuf;

use borsh::{BorshDeserialize, BorshSerialize};
use log::info;
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
    height_to_hash_filepath: PathBuf,
}

impl BlockManager {
    pub fn new(storage_path: &str) -> Self {
        Self {
            blocks: vec![],
            store: DbBlockStorage::new_boxed(storage_path),
            height_to_hash_filepath: PathBuf::from("data/height_to_hash.json"),
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
        match self.store.put(&block) {
            Ok(_) => {
                self.update_height_to_hash_mapping(&block)?;
                self.blocks.push(block);
                Ok(())
            }
            Err(e) => Err(e),
        }
    }

    pub fn get_block_by_height(&self, index: usize) -> Option<Block> {
        // TODO: handle mapping in more efficiently
        let mapping = self
            .load_height_to_hash_mapping(&self.height_to_hash_filepath.to_string_lossy())
            .unwrap();

        let hash = match mapping.get(&index) {
            Some(hash) => hash,
            None => return None,
        };

        match self.store.get(hash) {
            Ok(b) => Some(b),
            Err(_) => None,
        }
    }

    pub fn get_block_by_hash(&self, hash: &str) -> Option<Block> {
        match self.store.get(hash) {
            Ok(b) => Some(b),
            Err(_) => None,
        }
    }

    pub fn get_header(&self, index: usize) -> Option<&Header> {
        if let Some(b) = self.blocks.get(index) {
            Some(&b.header)
        } else {
            None
        }
    }

    pub fn last(&self) -> Option<&Block> {
        self.blocks.last()
    }

    pub fn has_block(&self, height: usize) -> bool {
        height <= self.height()
    }

    pub fn height(&self) -> usize {
        self.blocks.len() - 1
    }

    // ---
    // Private Methods
    // ---
    fn update_height_to_hash_mapping(&self, block: &Block) -> Result<(), Box<dyn Error>> {
        let mapping =
            self.load_height_to_hash_mapping(&self.height_to_hash_filepath.to_string_lossy())?;

        self.write_height_to_hash_mapping(block, mapping)?;

        Ok(())
    }

    fn load_height_to_hash_mapping(
        &self,
        file_path: &str,
    ) -> Result<HashMap<usize, String>, Box<dyn Error>> {
        let mut file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true) // Create the file if it doesn't exist
            .open(file_path)?;

        let mut json_str = String::new();

        file.read_to_string(&mut json_str)?;
        if json_str.is_empty() {
            // File is newly created, initialize with empty JSON object
            json_str = "{}".to_string()
        }

        Ok(serde_json::from_str(&json_str)?)
    }

    fn write_height_to_hash_mapping(
        &self,
        block: &Block,
        mut mapping: HashMap<usize, String>,
    ) -> Result<(), Box<dyn Error>> {
        // Update the height_to_hash mapping
        mapping.insert(block.height(), block.hash().to_hex()?);

        // Serialize the updated HashMap to JSON
        let json_str = serde_json::to_string_pretty(&mapping)?;

        // Write the JSON back to the file
        let mut file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true) // Create the file if it doesn't exist
            .open(&self.height_to_hash_filepath)?;

        file.write_all(json_str.as_bytes())?;
        Ok(())
    }

    pub fn new_in_memory() -> Self {
        Self {
            blocks: vec![],
            store: MemoryBlockStorage::new_boxed(),
            height_to_hash_filepath: PathBuf::from("data/height_to_hash.json"),
        }
    }
}

impl Default for BlockManager {
    fn default() -> Self {
        Self::new("data/chain.db")
    }
}

#[cfg(test)]
mod test {

    use crate::core::block::random_block;
    use crate::crypto::utils::random_hash;

    use crate::core::header::random_header;

    use super::*;

    #[test]
    fn test_header_manager() {
        let mut manager = BlockManager::default();

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
