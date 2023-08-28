use std::sync::{Arc, Mutex, MutexGuard};

use log::info;

use crate::{
    core::{hasher::Hasher, storage::MemoryStorage},
    crypto::hash::Hash,
};

use super::{
    block::Block,
    error::CoreError,
    header::{Header, Headers},
    storage::Storage,
    validator::{BlockValidator, Validator},
};

pub struct Blockchain<'a> {
    store: Box<dyn Storage>,
    headers: Arc<Mutex<Headers<'a>>>,
    validator: Box<dyn Validator>,
}

impl<'a> Blockchain<'a> {
    pub fn new(
        genesis_block: &'a Block,
        validator: Box<impl Validator + 'static>,
    ) -> Result<Self, CoreError> {
        let mut bc = Self {
            store: MemoryStorage::new_boxed(),
            headers: Arc::new(Mutex::new(Headers::new())),
            validator,
        };

        bc.add_block_without_validation(genesis_block)?;

        Ok(bc)
    }

    pub fn new_with_genesis(genesis_block: &'a Block) -> Self {
        let mut bc = Self::default();
        bc.add_block_without_validation(genesis_block).unwrap();
        bc
    }

    pub fn add_block(&mut self, block: &'a Block) -> Result<(), CoreError> {
        let mut headers = self.unlock_headers()?;

        match self.validator.validate_block(&headers, block) {
            Ok(_) => {
                headers.add(block.header());
                self.store.put(block)
            }
            Err(e) => Err(e),
        }
    }

    pub fn set_validator(&mut self, validator: Box<impl Validator + 'static>) {
        self.validator = validator
    }

    pub fn height(&self) -> usize {
        let headers = self.unlock_headers().unwrap();
        headers.height()
    }

    pub fn has_block(&self, height: usize) -> bool {
        height <= self.height() as usize
    }

    pub fn get_header_cloned(&self, index: usize) -> Option<Header> {
        let headers = self.unlock_headers().unwrap();
        let header = match headers.get(index) {
            Some(h) => h.clone(),
            None => return None,
        };
        Some(header)
    }

    pub fn get_prev_block_hash(&self, block_number: usize) -> Option<Hash> {
        self.get_header_cloned(block_number).map(|h| h.prev_hash())
    }

    // ---
    // Private Methods
    // ---

    fn add_block_without_validation(&mut self, block: &'a Block) -> Result<(), CoreError> {
        let mut headers = self.unlock_headers()?;
        headers.add(block.header());

        info!(
            "adding new block: height: {}, header_hash:{}",
            block.height(),
            block.header().hash()
        );

        self.store.put(block)
    }

    fn unlock_headers(&self) -> Result<MutexGuard<'_, Headers<'a>>, CoreError> {
        match self.headers.lock() {
            Ok(h) => Ok(h),
            Err(e) => Err(CoreError::Block(
                "unable to lock headers on blockchain".to_string(),
            )),
        }
    }
}

impl<'a> Default for Blockchain<'a> {
    fn default() -> Self {
        Self {
            store: MemoryStorage::new_boxed(),
            headers: Arc::new(Mutex::new(Headers::new())),
            validator: BlockValidator::new_boxed(),
        }
    }
}

#[cfg(test)]
mod test {
    use log::{error, info};
    fn init() {
        env_logger::init();
    }

    use crate::{
        core::{
            block::{random_block, random_signed_block},
            hasher::Hasher,
            header::random_header,
            validator,
        },
        crypto::{hash::Hash, utils::random_hash},
        setup,
    };

    use super::*;

    #[test]
    fn test_new_blockchain() {
        let genesis_hash = Hash::new(&[0_u8; 32]).unwrap();
        let header = random_header(0, genesis_hash);
        let genesis_block = random_block(&header);
        let validator = BlockValidator::new_boxed();
        let mut bc = Blockchain::new(&genesis_block, validator);

        assert!(bc.is_ok())
    }

    #[test]
    fn test_add_block() {
        let genesis_hash = Hash::new(&[0_u8; 32]).unwrap();
        let genesis_header = random_header(0, genesis_hash);
        let genesis_block = random_block(&genesis_header);
        let mut bc = Blockchain::new_with_genesis(&genesis_block);

        // check cannot re-add existing block
        let err_msg = match bc.add_block(&genesis_block) {
            Ok(_) => "wrong message".to_string(),
            Err(e) => e.to_string(),
        };
        assert_eq!("blockchain already contains block", err_msg);

        let new_header = random_header(1, genesis_header.hash());

        let new_signed_block = random_signed_block(&new_header);

        match bc.add_block(&new_signed_block) {
            Ok(_) => {}
            Err(e) => println!("{e}"),
        }

        // fails to re add same signed block
        assert!(bc.add_block(&new_signed_block).is_err());

        assert_eq!(bc.height(), 1);

        let new_height = bc.height() + 1;
        let new_header_2 = random_header(new_height as u64, new_header.hash());
        let new_block_2 = random_signed_block(&new_header_2);

        assert!(bc.add_block(&new_block_2).is_ok());

        assert_eq!(bc.height(), 2);
    }

    #[test]
    fn test_has_block() {
        let header = random_header(0, random_hash());
        let genesis_block = random_block(&header);
        let mut bc = Blockchain::new_with_genesis(&genesis_block);

        assert!(bc.has_block(0));
    }

    #[test]
    fn get_header() {
        let header = random_header(0, random_hash());
        let genesis_block = random_block(&header);
        let mut bc = Blockchain::new_with_genesis(&genesis_block);

        let mut headers: Vec<Header> = vec![];
        let mut prev_header: Header = random_header(1 as u64, genesis_block.hash());
        headers.push(prev_header.clone());

        for i in 2..50 {
            let new_header = random_header(i as u64, prev_header.hash());
            prev_header = new_header.clone();

            headers.push(new_header)
        }

        let mut blocks: Vec<Block> = vec![];

        for header in &headers {
            let new_block = random_signed_block(header);
            blocks.push(new_block)
        }

        for block in &blocks {
            assert!(bc.add_block(&block).is_ok());
        }

        let last_block = blocks.last().unwrap();

        assert!(bc.get_header_cloned(last_block.height() as usize).is_some());
    }
}
