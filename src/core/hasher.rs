use sha256::digest;

use crate::crypto::{error::CryptoError, hash::Hash};

use super::block::Block;

trait Hasher<T> {
    fn hash(data: T) -> Result<Hash, CryptoError>;
}

pub struct BlockHasher {}

impl<'a> Hasher<Block<'a>> for BlockHasher {
    fn hash(block: Block) -> Result<Hash, CryptoError> {
        let str = digest(block.header_data());
        let bytes = str.as_bytes();
        Hash::new(bytes)
    }
}
