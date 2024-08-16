use borsh::{BorshDeserialize, BorshSerialize};
use serde::{Deserialize, Serialize};
use std::ops::Deref;

use super::{error::CryptoError, private_key::PrivateKey, public_key::PublicKey};
use crate::core::{
    encoding::{ByteEncoding, HexEncoding},
    error::CoreError,
};

#[derive(Debug, Clone, BorshDeserialize, BorshSerialize, PartialEq, Eq, PartialOrd, Ord)]
pub struct Address {
    inner: [u8; 20],
}

impl Deref for Address {
    type Target = [u8; 20];

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl AsRef<[u8]> for Address {
    fn as_ref(&self) -> &[u8] {
        &self.inner
    }
}

impl Address {
    pub fn new(data: &[u8]) -> Self {
        let mut bytes = [0_u8; 20];
        for (i, byte) in data.iter().enumerate() {
            bytes[i] = byte.clone()
        }
        Self { inner: bytes }
    }
}

impl ByteEncoding<Address> for Address {
    fn to_bytes(&self) -> Result<Vec<u8>, CoreError> {
        match borsh::to_vec(self) {
            Ok(b) => Ok(b),
            Err(e) => Err(CoreError::Parsing(e.to_string())),
        }
    }

    fn from_bytes(data: &[u8]) -> Result<Address, CoreError> {
        match borsh::from_slice(data) {
            Ok(t) => Ok(t),
            Err(e) => Err(CoreError::Parsing(e.to_string())),
        }
    }
}

impl HexEncoding<Address> for Address {
    fn from_hex(data: &str) -> Result<Address, CoreError> {
        Ok(Self::from_bytes(&hex::decode(data)?)?)
    }

    fn to_hex(&self) -> Result<String, CoreError> {
        Ok(hex::encode(self.to_bytes()?))
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::crypto::{error::CryptoError, private_key::PrivateKey};

    #[test]
    fn test_address() {
        let pvt_key = PrivateKey::new();
        let pub_key = pvt_key.pub_key();

        let pvt_key_2 = PrivateKey::new();
        let pub_key_2 = pvt_key_2.pub_key();

        let addr = pub_key.address().unwrap();

        let bytes = pub_key.to_bytes().unwrap();

        let mut addr_bytes = [0_u8; 20];

        for (i, &b) in bytes.iter().rev().enumerate() {
            if i == 20 {
                break;
            }
            addr_bytes[i] = b
        }

        let addr_2 = Address::from_bytes(&addr_bytes).unwrap();

        assert_eq!(addr.to_hex().unwrap(), addr_2.to_hex().unwrap());

        let bytes = pub_key_2.to_bytes().unwrap();

        let mut addr_bytes = [0_u8; 20];

        for (i, &b) in bytes.iter().rev().enumerate() {
            if i == 20 {
                break;
            }
            addr_bytes[i] = b
        }

        let addr_3 = Address::from_bytes(&addr_bytes).unwrap();
        assert_ne!(addr.to_hex().unwrap(), addr_3.to_hex().unwrap());

        let bytes = pub_key_2.to_bytes().unwrap();
        let mut addr_bytes = [0_u8; 20];

        for (i, &b) in bytes.iter().rev().enumerate() {
            if i == 20 {
                break;
            }
            addr_bytes[i] = b
        }

        let new_hex = hex::encode(&addr_bytes);
        let addr_4 = Address::from_hex(&new_hex).unwrap();

        assert_eq!(
            pub_key_2.address().unwrap().to_hex().unwrap(),
            addr_4.to_hex().unwrap()
        );
    }
}

pub fn random_sender_receiver() -> (Address, Address) {
    let pub1 = PrivateKey::new().pub_key();
    let pub2 = PrivateKey::new().pub_key();

    (pub1.address().unwrap(), pub2.address().unwrap())
}
