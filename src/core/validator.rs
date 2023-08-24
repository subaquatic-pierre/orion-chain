use super::{block::Block, blockchain::Blockchain, error::CoreError};

pub trait Validator {
    fn validate_block(&self, blockchain: &Blockchain, block: &Block) -> Result<(), CoreError>;
}

pub struct BlockValidator {}

impl BlockValidator {
    pub fn new() -> Self {
        Self {}
    }

    pub fn new_boxed() -> Box<Self> {
        Box::new(Self {})
    }
}

impl Validator for BlockValidator {
    fn validate_block(&self, blockchain: &Blockchain, block: &Block) -> Result<(), CoreError> {
        if blockchain.has_block(block.height()) {
            return Err(CoreError::Block(
                "blockchain already contains block".to_string(),
            ));
        }

        block.verify()
    }
}
