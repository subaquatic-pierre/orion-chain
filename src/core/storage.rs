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

        let block_cf = self.get_cf_handle(&self.block_cf).ok_or_else(|| {
            CoreError::Block("unable to get block column family from db".to_string())
        })?;

        let height_cf = self.get_cf_handle(&self.height_to_hash_cf).ok_or_else(|| {
            CoreError::Block("unable to get height column family from db".to_string())
        })?;

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
        let block_cf = self.get_cf_handle(&self.block_cf).ok_or_else(|| {
            CoreError::Block("unable to get block column family from db".to_string())
        })?;

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
            None => {
                error!("unable to get ColumnFamily handle in height_to_hash");
                return None;
            }
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::block::random_block;
    use crate::core::header::random_header;
    use crate::core::{block::Block, header::Header}; // Adjust the import path based on your project structure
    use crate::crypto::utils::random_hash; // Adjust the import path based on your project structure
    use tempfile::tempdir;

    #[test]
    fn test_in_mem_put_block() {
        let mut storage = MemoryBlockStorage::new();

        let random_header = random_header(1, random_hash());
        let block = random_block(random_header);
        assert!(storage.put(&block).is_ok());

        assert_eq!(storage.last_block_height(), Some(1));
        assert_eq!(
            storage.height_to_hash(1),
            Some(block.hash().to_hex().unwrap())
        );
        assert_eq!(storage.get(&block.hash().to_hex().unwrap()).unwrap(), block);
    }

    #[test]
    fn test_in_mem_get_block() {
        let mut storage = MemoryBlockStorage::new();

        let random_header = random_header(0, random_hash());
        let block = random_block(random_header);
        storage.put(&block).unwrap();

        let retrieved_block = storage.get(&block.hash().to_hex().unwrap()).unwrap();
        assert_eq!(retrieved_block, block);

        let non_existent_block = storage.get("non_existent_hash");
        assert!(non_existent_block.is_err());
    }

    #[test]
    fn test_in_mem_height_to_hash() {
        let mut storage = MemoryBlockStorage::new();

        let random_header_1 = random_header(1, random_hash());
        let random_header_2 = random_header(2, random_hash());
        let block1 = random_block(random_header_1);
        let block2 = random_block(random_header_2);

        storage.put(&block1).unwrap();
        storage.put(&block2).unwrap();

        assert_eq!(
            storage.height_to_hash(1),
            Some(block1.hash().to_hex().unwrap())
        );
        assert_eq!(
            storage.height_to_hash(2),
            Some(block2.hash().to_hex().unwrap())
        );
        assert_eq!(storage.height_to_hash(3), None);
    }

    #[test]
    fn test_in_mem_last_block_height() {
        let mut storage = MemoryBlockStorage::new();

        assert_eq!(storage.last_block_height(), Some(0)); // Initially no blocks, so height should

        let random_header_1 = random_header(1, random_hash());
        let random_header_2 = random_header(2, random_hash());
        let block1 = random_block(random_header_1);
        let block2 = random_block(random_header_2);

        storage.put(&block1).unwrap();
        assert_eq!(storage.last_block_height(), Some(1));

        storage.put(&block2).unwrap();
        assert_eq!(storage.last_block_height(), Some(2));
    }

    // DB Storage Tests

    #[test]
    fn test_db_put_block() {
        let temp_dir = tempdir().unwrap();
        let db_path = temp_dir.path().to_str().unwrap();
        let mut storage = DbBlockStorage::new(db_path);

        let random_header = random_header(1, random_hash());
        let block = random_block(random_header);
        assert!(storage.put(&block).is_ok());

        assert_eq!(storage.last_block_height(), Some(1));
        assert_eq!(
            storage.height_to_hash(1),
            Some(block.hash().to_hex().unwrap())
        );
        assert_eq!(storage.get(&block.hash().to_hex().unwrap()).unwrap(), block);
    }

    #[test]
    fn test_db_get_block() {
        let temp_dir = tempdir().unwrap();
        let db_path = temp_dir.path().to_str().unwrap();
        let mut storage = DbBlockStorage::new(db_path);

        let random_header = random_header(0, random_hash());
        let block = random_block(random_header);
        storage.put(&block).unwrap();

        let retrieved_block = storage.get(&block.hash().to_hex().unwrap()).unwrap();
        assert_eq!(retrieved_block, block);

        let non_existent_block = storage.get("non_existent_hash");
        assert!(non_existent_block.is_err());
    }

    #[test]
    fn test_db_height_to_hash() {
        let temp_dir = tempdir().unwrap();
        let db_path = temp_dir.path().to_str().unwrap();
        let mut storage = DbBlockStorage::new(db_path);

        let random_header_1 = random_header(1, random_hash());
        let random_header_2 = random_header(2, random_hash());
        let block1 = random_block(random_header_1);
        let block2 = random_block(random_header_2);

        storage.put(&block1).unwrap();
        storage.put(&block2).unwrap();

        assert_eq!(
            storage.height_to_hash(1),
            Some(block1.hash().to_hex().unwrap())
        );
        assert_eq!(
            storage.height_to_hash(2),
            Some(block2.hash().to_hex().unwrap())
        );
        assert_eq!(storage.height_to_hash(3), None);
    }

    #[test]
    fn test_db_last_block_height() {
        let temp_dir = tempdir().unwrap();
        let db_path = temp_dir.path().to_str().unwrap();
        let mut storage = DbBlockStorage::new(db_path);

        assert_eq!(storage.last_block_height(), None); // Initially no blocks, so height should

        let random_header_1 = random_header(1, random_hash());
        let random_header_2 = random_header(2, random_hash());
        let block1 = random_block(random_header_1);
        let block2 = random_block(random_header_2);

        storage.put(&block1).unwrap();
        assert_eq!(storage.last_block_height(), Some(1));

        storage.put(&block2).unwrap();
        assert_eq!(storage.last_block_height(), Some(2));
    }
}
