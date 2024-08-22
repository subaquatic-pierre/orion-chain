use serde::de::Visitor;
use serde::{Deserialize, Serialize};
use sha256::Sha256Digest;
use std::fmt;
use std::fmt::Display;
use std::hash::{Hash as StdHash, Hasher as StdHasher};
use std::ops::Deref;

use crate::core::encoding::{ByteEncoding, HexEncoding};
use crate::core::error::CoreError;
use borsh::{BorshDeserialize, BorshSerialize};

use super::error::CryptoError;

#[derive(Clone, Debug, Ord, Copy, PartialOrd, BorshSerialize, BorshDeserialize)]
pub struct Hash([u8; 32]);

impl Hash {
    pub fn new(bytes: &[u8; 32]) -> Result<Self, CryptoError> {
        if bytes.len() != 32 {
            return Err(CryptoError::HashError("incorrect byte length".to_string()));
        }

        let mut buf = [0_u8; 32];

        for (i, &b) in bytes.iter().enumerate() {
            buf[i] = b
        }

        Ok(Self(buf))
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn sha256(data: &[u8]) -> Result<Self, CryptoError> {
        let bytes = hex::decode(sha256::digest(data));

        if bytes.is_err() {
            return Err(CryptoError::HashError(
                "unable to hex decode sha256 digest".to_string(),
            ));
        }
        let bytes = bytes.unwrap();
        let mut buf = [0_u8; 32];
        for (i, b) in bytes.iter().enumerate() {
            buf[i] = b.clone()
        }
        Self::new(&buf)
    }

    pub fn is_zero(&self) -> bool {
        for &b in self.0.iter() {
            if b != 0 {
                return false;
            }
        }
        true
    }
}

pub struct Hasher {
    data: Vec<u8>,
}

impl Hasher {
    pub fn new() -> Self {
        Self { data: vec![] }
    }

    pub fn update(&mut self, data: &Vec<u8>) -> Result<(), CryptoError> {
        let mut buf = vec![];
        buf.extend_from_slice(&self.data);
        buf.extend_from_slice(data);
        let bytes = hex::decode(sha256::digest(data));

        if bytes.is_err() {
            return Err(CryptoError::HashError(
                "unable to hex decode sha256 digest".to_string(),
            ));
        }

        let bytes = bytes.unwrap();

        self.data = bytes;
        Ok(())
    }

    pub fn finalize(&self) -> Result<Hash, CryptoError> {
        let mut buf: [u8; 32] = [0_u8; 32];
        for (i, b) in self.data.iter().enumerate() {
            buf[i] = b.clone()
        }
        Hash::new(&buf)
    }
}

impl ByteEncoding<Hash> for Hash {
    fn to_bytes(&self) -> Result<Vec<u8>, CoreError> {
        Ok(self.0.to_vec())
    }

    fn from_bytes(data: &[u8]) -> Result<Hash, CoreError> {
        let mut buf = [0_u8; 32];
        for (i, b) in data.iter().enumerate() {
            buf[i] = b.clone()
        }
        Ok(Self::new(&buf)?)
    }
}

impl HexEncoding<Hash> for Hash {
    fn to_hex(&self) -> Result<String, CoreError> {
        Ok(hex::encode(self.0))
    }

    fn from_hex(data: &str) -> Result<Hash, CoreError> {
        Ok(Self::from_bytes(&hex::decode(data)?)?)
    }
}

impl Display for Hash {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.to_hex().unwrap())
    }
}

impl PartialEq for Hash {
    fn eq(&self, other: &Self) -> bool {
        self.to_string() == other.to_string()
    }
}

impl Eq for Hash {}

impl StdHash for Hash {
    fn hash<H: StdHasher>(&self, state: &mut H) {
        self.0.hash(state);
    }
}

impl Deref for Hash {
    type Target = [u8; 32];

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::crypto::utils::{random_bytes, random_hash};

    #[test]
    fn test_hash() {
        let random_hash = random_hash();

        let random_hash = random_hash;

        let random_bytes = random_bytes(32);

        let buf = [0_u8; 32];

        let zero_hash = Hash::from_bytes(&buf).unwrap();

        assert_eq!(zero_hash.is_zero(), true);
        assert_ne!(random_hash.is_zero(), true);

        assert_eq!(random_hash.to_bytes().unwrap().len(), 32);

        let mut buf = [0_u8; 32];

        for i in 0..32 {
            buf[i] = i as u8
        }

        let hash_1 = Hash::new(&buf).unwrap();
        let hash_2 = Hash::new(&buf).unwrap();

        let hash_3 = Hash::from_hex(&hash_1.to_hex().unwrap());

        assert!(hash_3.is_ok());

        let hash_3 = hash_3.unwrap();

        assert_eq!(hash_3.to_hex().unwrap(), hash_1.to_hex().unwrap());

        assert_eq!(hash_1.to_string(), hash_2.to_string());

        assert_eq!(random_bytes.len(), 32);

        let _hash = sha256::digest("Hello world, Data is cool");

        let h = Hash::sha256(b"Hello world, Data is cool");

        assert!(h.is_ok());

        let hash = h.unwrap();

        assert_eq!(hash.to_bytes().unwrap().len(), 32);

        let sha_h = sha256::digest("Hello world, Data is cool");

        assert_eq!(hash.to_string(), sha_h);
    }
}
