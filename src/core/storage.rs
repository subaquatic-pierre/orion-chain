use super::{block::Block, error::CoreError};

pub trait Storage {
    fn put(&self, block: &Block) -> Result<(), CoreError>;
}

#[derive(Clone, Debug)]
pub struct MemoryStorage {}
impl MemoryStorage {
    pub fn new() -> Self {
        Self {}
    }

    pub fn new_boxed() -> Box<Self> {
        Box::new(MemoryStorage::new())
    }
}

impl Storage for MemoryStorage {
    fn put(&self, _block: &Block) -> Result<(), CoreError> {
        Ok(())
    }
}
