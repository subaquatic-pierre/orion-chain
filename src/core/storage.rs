use std::{collections::HashMap, iter::Map};

use super::{block::Block, encoding::HexEncoding, error::CoreError};
use crate::{core::encoding::ByteEncoding, crypto::hash::Hash};
use rocksdb::{Options, DB};

pub trait BlockStorage: Send + Sync {
    fn put(&mut self, block: &Block) -> Result<(), CoreError>;
    fn get(&self, hash: &str) -> Result<Block, CoreError>;
}

pub struct MemoryBlockStorage {
    store: HashMap<String, Block>,
}
impl MemoryBlockStorage {
    pub fn new() -> Self {
        Self {
            store: HashMap::new(),
        }
    }

    pub fn new_boxed() -> Box<Self> {
        Box::new(MemoryBlockStorage::new())
    }
}

impl BlockStorage for MemoryBlockStorage {
    fn put(&mut self, block: &Block) -> Result<(), CoreError> {
        self.store.insert(block.hash().to_string(), block.clone());
        Ok(())
    }
    fn get(&self, hash: &str) -> Result<Block, CoreError> {
        match self.store.get(hash) {
            Some(block) => Ok(block.clone()),
            None => Err(CoreError::Block(format!(
                "block with hash: {hash} to found"
            ))),
        }
    }
}

pub struct DbBlockStorage {
    db: DB,
}
impl DbBlockStorage {
    pub fn new(path: &str) -> Self {
        let mut options = Options::default();
        options.create_if_missing(true);
        let db = DB::open(&options, path).unwrap();

        Self { db }
    }

    pub fn new_boxed(path: &str) -> Box<Self> {
        Box::new(DbBlockStorage::new(path))
    }
}

impl BlockStorage for DbBlockStorage {
    fn put(&mut self, block: &Block) -> Result<(), CoreError> {
        let serialized = block.to_bytes().unwrap();
        self.db
            .put(block.hash().to_hex()?, serialized)
            .map_err(|e| CoreError::Block(e.to_string()))?;
        Ok(())
    }

    fn get(&self, hash: &str) -> Result<Block, CoreError> {
        match self.db.get(hash) {
            Ok(res) => match res {
                Some(bytes) => Ok(Block::from_bytes(&bytes)?),
                None => Err(CoreError::Block(format!(
                    "block not found with hash: {hash}"
                ))),
            },
            Err(e) => Err(CoreError::Block(e.to_string())),
        }
    }
}
