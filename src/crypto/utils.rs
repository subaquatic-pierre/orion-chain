use rand::random;

use super::{error::CryptoError, hash::Hash};

pub fn random_bytes(num_bytes: u32) -> Vec<u8> {
    (0..num_bytes).map(|_| random::<u8>()).collect()
}

pub fn random_hash() -> Hash {
    Hash::new(&random_bytes(32)).unwrap()
}
