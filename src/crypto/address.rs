use ecdsa::{
    elliptic_curve::{rand_core::OsRng, NonZeroScalar},
    signature::{DigestVerifier, Signer, Verifier},
    Signature as ECDASignature, SigningKey, VerifyingKey,
};
use k256::{Secp256k1, SecretKey, U256};
use sha256::digest;
use std::ops::Deref;
use std::{error::Error, fmt::Display};

use crate::core::encoding::{ByteDecoding, ByteEncoding, HexDecoding, HexEncoding};

use super::{error::CryptoError, private_key::PrivateKey};

pub struct Address {
    inner: [u8; 20],
}

impl Deref for Address {
    type Target = [u8; 20];

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl Address {}

impl ByteDecoding for Address {
    type Target = Self;
    type Error = CryptoError;

    fn from_bytes(data: &[u8]) -> Result<Address, CryptoError> {
        let mut buf = [0_u8; 20];
        if data.len() != 20 {
            return Err(CryptoError::GenerateKey(
                "incorrect byte format for address".to_string(),
            ));
        }

        for (i, &b) in data.iter().enumerate() {
            buf[i] = b
        }

        Ok(Self { inner: buf })
    }
}

impl ByteEncoding for Address {
    fn to_bytes(&self) -> Vec<u8> {
        self.inner.to_vec()
    }
}

impl HexEncoding for Address {
    fn to_hex(&self) -> String {
        hex::encode(self.inner)
    }
}

impl HexDecoding for Address {
    type Target = Self;
    type Error = CryptoError;
    fn from_hex(data: &str) -> Result<Address, CryptoError> {
        if data.len() != 40 {
            return Err(CryptoError::GenerateKey(
                "incorrect hex format for address".to_string(),
            ));
        }

        let bytes = hex::decode(data);

        if bytes.is_err() {
            return Err(CryptoError::GenerateKey(
                "unable to generate bytes from hex".to_string(),
            ));
        }

        let bytes = bytes.unwrap();

        Self::from_bytes(&bytes)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_address() {
        let pvt_key = PrivateKey::new();
        let pub_key = pvt_key.pub_key();

        let pvt_key_2 = PrivateKey::new();
        let pub_key_2 = pvt_key_2.pub_key();

        let addr = pub_key.address().unwrap();

        let bytes = pub_key.to_bytes();

        let mut addr_bytes = [0_u8; 20];

        for (i, &b) in bytes.iter().rev().enumerate() {
            if i == 20 {
                break;
            }
            addr_bytes[i] = b
        }

        let addr_2 = Address::from_bytes(&addr_bytes).unwrap();

        assert_eq!(addr.to_hex(), addr_2.to_hex());

        let bytes = pub_key_2.to_bytes();

        let mut addr_bytes = [0_u8; 20];

        for (i, &b) in bytes.iter().rev().enumerate() {
            if i == 20 {
                break;
            }
            addr_bytes[i] = b
        }

        let addr_3 = Address::from_bytes(&addr_bytes).unwrap();
        assert_ne!(addr.to_hex(), addr_3.to_hex());

        let bytes = pub_key_2.to_bytes();
        let mut addr_bytes = [0_u8; 20];

        for (i, &b) in bytes.iter().rev().enumerate() {
            if i == 20 {
                break;
            }
            addr_bytes[i] = b
        }

        let new_hex = hex::encode(&addr_bytes);
        let addr_4 = Address::from_hex(&new_hex).unwrap();

        assert_eq!(pub_key_2.address().unwrap().to_hex(), addr_4.to_hex());
    }
}
