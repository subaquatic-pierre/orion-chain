use log::info;

use crate::{crypto::hash::Hash, state::manager::StateManager};

use super::{
    block::{random_block, Block},
    error::CoreError,
    header::{random_header, Header},
    manager::BlockManager,
    storage::BlockStorage,
};

pub struct Blockchain {
    block_manager: BlockManager,
    state_manager: StateManager,
}

impl Blockchain {
    pub fn new(
        state_storage_path: &str,
        block_storage_path: &str,
        genesis_block: Block,
    ) -> Result<Self, CoreError> {
        let mut bc = Self {
            block_manager: BlockManager::new(block_storage_path),
            state_manager: StateManager::new(state_storage_path),
        };

        bc.add_block_without_validation(genesis_block)?;

        Ok(bc)
    }

    pub fn add_block(&mut self, block: Block) -> Result<(), CoreError> {
        if self.has_block(block.height()) {
            return Err(CoreError::Block(
                "blockchain already contains block".to_string(),
            ));
        }
        self.block_manager.add(block)
    }

    pub fn height(&self) -> usize {
        let manager = &self.block_manager;
        manager.height()
    }

    pub fn has_block(&self, height: usize) -> bool {
        height <= self.height() as usize
    }

    pub fn last_block(&self) -> Option<Block> {
        self.block_manager.last()
    }

    pub fn get_block_by_height(&self, index: usize) -> Option<Block> {
        self.block_manager.get_block_by_height(index)
    }

    pub fn get_block_by_hash(&self, hash: &str) -> Option<Block> {
        self.block_manager.get_block_by_hash(hash)
    }

    pub fn get_prev_block_hash(&self, block_height: usize) -> Option<Hash> {
        self.get_block_by_height(block_height)
            .map(|b| b.header.prev_hash())
    }

    pub fn state(&self) -> &StateManager {
        &self.state_manager
    }

    // ---
    // Private Methods
    // ---

    fn add_block_without_validation(&mut self, block: Block) -> Result<(), CoreError> {
        let manager = &mut self.block_manager;

        manager.add(block)
    }

    // ---
    // Used for testing and development
    // ---

    pub fn new_with_genesis() -> Result<Self, CoreError> {
        let genesis_hash = Hash::new(&[0_u8; 32]).unwrap();
        let block = random_block(random_header(0, genesis_hash));
        let mut bc = Self::default();
        bc.add_block_without_validation(block).unwrap();
        Ok(bc)
    }

    pub fn new_with_genesis_in_memory() -> Result<Self, CoreError> {
        let genesis_hash = Hash::new(&[0_u8; 32]).unwrap();
        let block = random_block(random_header(0, genesis_hash));
        let mut bc = Self::new_in_memory()?;
        bc.add_block_without_validation(block).unwrap();
        Ok(bc)
    }

    pub fn new_in_memory() -> Result<Self, CoreError> {
        let bc: Blockchain = Self {
            block_manager: BlockManager::new_in_memory(),
            state_manager: StateManager::new_in_memory(),
        };

        Ok(bc)
    }
}

impl Default for Blockchain {
    fn default() -> Self {
        Self {
            block_manager: BlockManager::default(),
            state_manager: StateManager::default(),
        }
    }
}

#[cfg(test)]
mod tests {
    use log::{error, info};
    fn init() {
        env_logger::init();
    }

    use crate::{
        core::{
            block::{random_block, random_signed_block},
            header::random_header,
        },
        crypto::{hash::Hash, utils::random_hash},
        logger_init,
    };

    use super::*;

    #[test]
    fn test_new_blockchain() {
        let bc = Blockchain::new_with_genesis_in_memory();

        assert!(bc.is_ok())
    }

    #[test]
    fn test_add_block() {
        let mut bc = Blockchain::new_with_genesis_in_memory().unwrap();
        let genesis_block = bc.get_block_by_height(0).unwrap();
        let genesis_header = genesis_block.header().clone();

        // check cannot re-add existing block
        let err_msg = match bc.add_block(genesis_block.clone()) {
            Ok(_) => "wrong message".to_string(),
            Err(e) => e.to_string(),
        };
        assert_eq!("blockchain already contains block", err_msg);

        let new_header = random_header(1, genesis_header.hash().clone());

        let new_signed_block = random_signed_block(new_header.clone());

        match bc.add_block(new_signed_block.clone()) {
            Ok(_) => {}
            Err(e) => println!("{e}"),
        }

        // fails to re add same signed block
        assert!(bc.add_block(new_signed_block).is_err());

        // assert_eq!(bc.height(), 1);

        let new_height = bc.height() + 1;
        let last_block = bc.last_block();
        let last_block = last_block.unwrap();
        let new_header_2 = random_header(new_height, last_block.hash().clone());
        let new_block_2 = random_signed_block(new_header_2);

        assert!(bc.add_block(new_block_2).is_ok());

        assert_eq!(bc.height(), 2);
    }

    #[test]
    fn test_has_block() {
        let bc = Blockchain::new_with_genesis_in_memory().unwrap();

        assert!(bc.has_block(0));
    }

    #[test]
    fn test_get_header() {
        let mut bc = Blockchain::new_with_genesis_in_memory().unwrap();
        let genesis_block = bc.get_block_by_height(0).unwrap();

        let mut headers: Vec<Header> = vec![];
        let mut prev_header: Header = random_header(1, genesis_block.hash().clone());
        headers.push(prev_header.clone());

        for i in 2..50 {
            let new_header = random_header(i, prev_header.hash());
            prev_header = new_header.clone();

            headers.push(new_header)
        }

        let mut blocks: Vec<Block> = vec![];

        for header in &headers {
            let new_block = random_signed_block(header.to_owned());
            blocks.push(new_block)
        }

        for block in &blocks {
            if let Err(e) = bc.add_block(block.to_owned()) {
                println!("{e}");
                assert!(bc.add_block(block.to_owned()).is_ok());
            }
        }

        let last_block = bc.last_block().unwrap();

        let block = bc
            .get_block_by_height(last_block.height() as usize)
            .unwrap();

        // let last_block = blocks.last().unwrap();

        assert_eq!(last_block.hash(), block.hash());
    }
}
