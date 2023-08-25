use sha256::digest;

use crate::crypto::{error::CryptoError, hash::Hash};

use super::{
    block::Block,
    encoding::{ByteDecoding, ByteEncoding},
};

pub trait Hasher<E>
where
    E: ByteEncoding,
{
    fn hash(&self) -> Hash;
}
