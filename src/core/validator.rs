use super::{
    block::{Block, BlockManager},
    error::CoreError,
    hasher::Hasher,
    header::HeaderManager,
};

pub struct BlockValidator {}

impl BlockValidator {
    pub fn new() -> Self {
        Self {}
    }

    pub fn new_boxed() -> Box<Self> {
        Box::new(Self {})
    }
}

impl BlockValidator {
    pub fn validate_block(&self, manager: &BlockManager, block: &Block) -> Result<(), CoreError> {
        if manager.has_block(block.height()) {
            return Err(CoreError::Block(
                "blockchain already contains block".to_string(),
            ));
        }

        if block.height() != manager.height() + 1 {
            return Err(CoreError::Block(
                "block is to high too be added".to_string(),
            ));
        }

        let last_header = match manager.last() {
            Some(header) => header,
            None => return Err(CoreError::Block("incorrect header height".to_string())),
        };

        // check correct prev hash
        let cur_header = block.header();

        if cur_header.prev_hash() != last_header.hash().clone() {
            return Err(CoreError::Block("incorrect previous hash".to_string()));
        }

        block.verify()
    }
}
