use super::{block::Block, blockchain::Blockchain, error::CoreError, hasher::Hasher};

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

        if block.height() != blockchain.height() + 1 {
            return Err(CoreError::Block(
                "block is to high too be added".to_string(),
            ));
        }

        let prev_header = match blockchain.get_header((block.height() - 1) as usize) {
            Some(header) => header,
            None => return Err(CoreError::Block("incorrect header height".to_string())),
        };

        // check correct prev hash
        let cur_header = block.header();

        if cur_header.prev_hash() != prev_header.hash() {
            return Err(CoreError::Block("incorrect previous hash".to_string()));
        }

        block.verify()
    }
}
