use crate::crypto::hash::Hash;

use super::encoding::ByteEncoding;

pub trait Hasher<E>
where
    E: ByteEncoding,
{
    fn hash(&self) -> Hash;
}
