use std::fmt::Display;

use ecdsa::elliptic_curve::rand_core;
use rand::random;

use crate::core::encoding::{ByteEncoding, HexEncoding};

use super::error::CryptoError;
use super::utils::{random_bytes, random_hash};

pub struct Hash {
    inner: [u8; 32],
}

impl Hash {
    pub fn new(bytes: &[u8]) -> Result<Self, CryptoError> {
        if bytes.len() != 32 {
            return Err(CryptoError::HashError("incorrect byte length".to_string()));
        }

        let mut buf = [0_u8; 32];

        for (i, &b) in bytes.iter().enumerate() {
            buf[i] = b
        }

        Ok(Self { inner: buf })
    }

    pub fn is_zero(&self) -> bool {
        for &b in self.inner.iter() {
            if b != 0 {
                return false;
            }
        }
        true
    }
}

impl ByteEncoding<Hash, CryptoError> for Hash {
    fn from_bytes(data: &[u8]) -> Result<Self, CryptoError> {
        Self::new(data)
    }

    fn to_bytes(&self) -> Vec<u8> {
        self.inner.to_vec()
    }
}

impl HexEncoding<Hash, CryptoError> for Hash {
    fn from_hex(data: &str) -> Result<Hash, CryptoError> {
        match hex::decode(data) {
            Ok(bytes) => Self::new(&bytes),
            Err(e) => Err(CryptoError::HashError("unable to decode hash".to_string())),
        }
    }

    fn to_hex(&self) -> String {
        hex::encode(self.inner)
    }
}

impl Display for Hash {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.to_hex())
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_hash() {
        let random_hash = random_hash();

        assert_eq!(random_hash.is_err(), false);

        let random_hash = random_hash.unwrap();

        let random_bytes = random_bytes(32);

        let buf = [0_u8; 32];

        let zero_hash = Hash::from_bytes(&buf).unwrap();

        assert_eq!(zero_hash.is_zero(), true);
        assert_ne!(random_hash.is_zero(), true);

        assert_eq!(random_hash.to_bytes().len(), 32);

        let mut buf = [0_u8; 32];

        for i in 0..32 {
            buf[i] = i as u8
        }

        let hash_1 = Hash::new(&buf).unwrap();
        let hash_2 = Hash::new(&buf).unwrap();

        let hash_3 = Hash::from_hex(&hash_1.to_hex());

        assert!(hash_3.is_ok());

        let hash_3 = hash_3.unwrap();

        assert_eq!(hash_3.to_hex(), hash_1.to_hex());

        assert_eq!(hash_1.to_string(), hash_2.to_string());

        assert_eq!(random_bytes.len(), 32);
    }
}
