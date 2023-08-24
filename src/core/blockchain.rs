use crate::core::storage::MemoryStorage;

use super::{
    block::Block,
    error::CoreError,
    header::Header,
    storage::Storage,
    validator::{BlockValidator, Validator},
};

pub struct Blockchain<'a> {
    store: Box<dyn Storage>,
    headers: Vec<&'a Header>,
    validator: Box<dyn Validator>,
}

impl<'a> Blockchain<'a> {
    pub fn new(
        genesis_block: &'a Block,
        validator: Box<impl Validator + 'static>,
    ) -> Result<Self, CoreError> {
        let mut bc = Self {
            store: MemoryStorage::new_boxed(),
            headers: vec![],
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
        match self.validator.validate_block(self, block) {
            Ok(_) => {
                self.headers.push(block.header());
                self.store.put(block)
            }
            Err(e) => Err(e),
        }
    }

    pub fn set_validator(&mut self, validator: Box<impl Validator + 'static>) {
        self.validator = validator
    }

    pub fn height(&self) -> u64 {
        (self.headers.len() - 1) as u64
    }

    pub fn has_block(&self, height: u64) -> bool {
        height <= self.height()
    }

    // ---
    // Private Methods
    // ---

    fn add_block_without_validation(&mut self, block: &'a Block) -> Result<(), CoreError> {
        self.headers.push(block.header());

        self.store.put(block)
    }
}

impl<'a> Default for Blockchain<'a> {
    fn default() -> Self {
        Self {
            store: MemoryStorage::new_boxed(),
            headers: vec![],
            validator: BlockValidator::new_boxed(),
        }
    }
}

#[cfg(test)]
mod test {
    use crate::core::{
        block::{random_block, random_signed_block},
        header::random_header,
        validator,
    };

    use super::*;

    #[test]
    fn test_new_blockchain() {
        let header = random_header(0);
        let genesis_block = random_block(&header);
        let validator = BlockValidator::new_boxed();
        let mut bc = Blockchain::new(&genesis_block, validator);

        assert!(bc.is_ok())
    }

    #[test]
    fn test_add_block() {
        let header = random_header(0);
        let genesis_block = random_block(&header);
        let mut bc = Blockchain::new_with_genesis(&genesis_block);

        let err_msg = match bc.add_block(&genesis_block) {
            Ok(_) => "wrong message".to_string(),
            Err(e) => e.to_string(),
        };
        assert_eq!("blockchain already contains block", err_msg);

        let new_header = random_header(1);
        let new_block = random_block(&new_header);

        let err_msg = match bc.add_block(&new_block) {
            Ok(_) => "wrong message".to_string(),
            Err(e) => e.to_string(),
        };

        assert_eq!("no signature exists for block", err_msg);

        let new_signed_block = random_signed_block(&new_header);

        assert!(bc.add_block(&new_signed_block).is_ok());

        // fails to re add same signed block
        assert!(bc.add_block(&new_signed_block).is_err());

        assert_eq!(bc.height(), 1);

        let new_height = bc.height() + 1;
        let new_header_2 = random_header(new_height);
        let new_block_2 = random_signed_block(&new_header_2);

        assert!(bc.add_block(&new_block_2).is_ok());

        assert_eq!(bc.height(), 2);
    }

    #[test]
    fn test_has_block() {
        let header = random_header(0);
        let genesis_block = random_block(&header);
        let mut bc = Blockchain::new_with_genesis(&genesis_block);

        assert!(bc.has_block(0));
    }
}
