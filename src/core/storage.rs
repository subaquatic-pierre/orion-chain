use super::{block::Block, encoding::HexEncoding, error::CoreError};
use crate::{core::encoding::ByteEncoding, crypto::hash::Hash};
use log::{error, warn};
use rocksdb::{ColumnFamily, ColumnFamilyDescriptor, IteratorMode, Options, WriteBatch, DB};
use std::str::FromStr;
use std::{collections::HashMap, iter::Map};

pub trait BlockStorage: Send + Sync {
    fn put(&mut self, block: &Block) -> Result<(), CoreError>;
    fn get(&self, hash: &str) -> Result<Block, CoreError>;
    fn height_to_hash(&self, height: usize) -> Option<String>;
    fn last_block_height(&self) -> Option<usize>;
}

pub struct MemoryBlockStorage {
    store: HashMap<String, Block>,
    height_to_hash: HashMap<usize, String>,
    last_block_height: usize,
}
impl MemoryBlockStorage {
    pub fn new() -> Self {
        Self {
            store: HashMap::new(),
            last_block_height: 0,
            height_to_hash: HashMap::new(),
        }
    }

    pub fn new_boxed() -> Box<Self> {
        Box::new(MemoryBlockStorage::new())
    }
}

impl BlockStorage for MemoryBlockStorage {
    fn put(&mut self, block: &Block) -> Result<(), CoreError> {
        self.last_block_height = block.height();
        self.height_to_hash
            .insert(block.height(), block.hash().to_hex()?);
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

    fn height_to_hash(&self, height: usize) -> Option<String> {
        self.height_to_hash.get(&height).cloned()
    }

    fn last_block_height(&self) -> Option<usize> {
        Some(self.last_block_height)
    }
}

pub struct DbBlockStorage {
    db: DB,
    block_cf: String,
    height_to_hash_cf: String,
}

impl DbBlockStorage {
    pub fn new(path: &str) -> Self {
        let block_cf = "block_cf".to_string();
        let height_to_hash_cf = "height_to_hash_cf".to_string();
        let mut options = Options::default();
        options.create_if_missing(true);
        options.create_missing_column_families(true);

        let block_cf_descriptor = ColumnFamilyDescriptor::new(&block_cf, Options::default());
        let height_cf_descriptor =
            ColumnFamilyDescriptor::new(&height_to_hash_cf, Options::default());

        let db = DB::open_cf_descriptors(
            &options,
            path,
            vec![block_cf_descriptor, height_cf_descriptor],
        )
        .expect("Unable to open DB with column families");

        Self {
            db,
            block_cf,
            height_to_hash_cf,
        }
    }

    pub fn new_boxed(path: &str) -> Box<Self> {
        Box::new(DbBlockStorage::new(path))
    }

    fn get_cf_handle(&self, name: &str) -> Option<&ColumnFamily> {
        self.db.cf_handle(name)
    }
}

impl BlockStorage for DbBlockStorage {
    fn put(&mut self, block: &Block) -> Result<(), CoreError> {
        let mut batch = WriteBatch::default();

        let block_cf = match self.get_cf_handle(&self.block_cf) {
            Some(cf) => cf,
            None => {
                return Err(CoreError::Block(
                    "unable to get block column family from db".to_string(),
                ))
            }
        };
        let height_cf = match self.get_cf_handle(&self.height_to_hash_cf) {
            Some(cf) => cf,
            None => {
                return Err(CoreError::Block(
                    "unable to get height column family from db".to_string(),
                ))
            }
        };

        // Store block by hash in block_cf
        batch.put_cf(block_cf, block.hash().to_hex()?, block.to_bytes()?);

        let block_height = block.height();
        batch.put_cf(
            height_cf,
            block_height.to_string(),
            block.hash().to_bytes()?,
        );

        // Write batch
        self.db.write(batch).unwrap();

        Ok(())
    }

    fn get(&self, hash: &str) -> Result<Block, CoreError> {
        let block_cf = match self.get_cf_handle(&self.block_cf) {
            Some(cf) => cf,
            None => {
                return Err(CoreError::Block(
                    "unable to get block column family from db".to_string(),
                ))
            }
        };
        match self.db.get_cf(block_cf, hash) {
            Ok(res) => match res {
                Some(bytes) => Ok(Block::from_bytes(&bytes)?),
                None => Err(CoreError::Block(format!(
                    "block not found with hash: {hash}"
                ))),
            },
            Err(e) => Err(CoreError::Block(e.to_string())),
        }
    }

    fn height_to_hash(&self, height: usize) -> Option<String> {
        let height_to_hash_cf = match self.get_cf_handle(&self.height_to_hash_cf) {
            Some(cf) => cf,
            None => return None,
        };

        match self.db.get_cf(height_to_hash_cf, height.to_string()) {
            Ok(Some(hash_bytes)) => match Hash::from_bytes(&hash_bytes) {
                Ok(hash) => Some(hash.to_hex().unwrap()),
                Err(_) => {
                    warn!("unable to get hash from height_to_hash_cf for height: {height}");
                    None
                }
            },
            Ok(None) => {
                warn!("unable to get hash from height_to_hash_cf for height: {height}");
                None
            }
            Err(e) => {
                error!("error getting hash from height_to_hash_cf, {e}");
                None
            }
        }
    }

    fn last_block_height(&self) -> Option<usize> {
        let height_to_hash_cf = match self.get_cf_handle(&self.height_to_hash_cf) {
            Some(cf) => cf,
            None => {
                error!("unable to get ColumnFamily handle in last_block_height");
                return None;
            }
        };

        let mut iter = self.db.iterator_cf(height_to_hash_cf, IteratorMode::End);

        if let Some(Ok((key, _))) = iter.next() {
            let key_str = String::from_utf8(key.to_vec()).ok()?;
            // Convert the string key to usize
            let height = usize::from_str(&key_str).ok();
            height
        } else {
            error!("no blocks found in database");

            None // No blocks in the database
        }
    }
}

// aa951ce3b56f48e77e81d6caad03438ddaaaff880d5ad15abfbee2b5a6560ee5
// aa951ce3b56f48e77e81d6caad03438ddaaaff880d5ad15abfbee2b5a6560ee5
