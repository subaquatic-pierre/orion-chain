use std::sync::MutexGuard;

use super::{
    block::Block,
    blockchain::Blockchain,
    error::CoreError,
    hasher::Hasher,
    header::{Header, HeaderManager},
};

// pub trait Validator {
//     fn validate_block(&self, headers: &HeaderManager, block: &Block) -> Result<(), CoreError>;
// }

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
    pub fn validate_block(&self, headers: &HeaderManager, block: &Block) -> Result<(), CoreError> {
        if headers.has_block(block.height()) {
            return Err(CoreError::Block(
                "blockchain already contains block".to_string(),
            ));
        }

        if block.height() != headers.height() + 1 {
            return Err(CoreError::Block(
                "block is to high too be added".to_string(),
            ));
        }

        let last_header = match headers.last() {
            Some(header) => header,
            None => return Err(CoreError::Block("incorrect header height".to_string())),
        };

        // check correct prev hash
        let cur_header = block.header();

        if cur_header.prev_hash() != last_header.hash() {
            return Err(CoreError::Block("incorrect previous hash".to_string()));
        }

        block.verify()
    }
}
