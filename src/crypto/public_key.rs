use ecdsa::{signature::Verifier, VerifyingKey};
use k256::Secp256k1;
use std::fmt::Display;

use crate::core::encoding::{ByteDecoding, ByteEncoding, HexDecoding, HexEncoding};

use super::{address::Address, error::CryptoError, private_key::PrivateKey, signature::Signature};

#[derive(Debug, Clone)]
pub struct PublicKey {
    key: VerifyingKey<Secp256k1>,
}

impl PublicKey {
    pub fn new(key: VerifyingKey<Secp256k1>) -> Self {
        Self { key }
    }

    pub fn address(&self) -> Result<Address, CryptoError> {
        let bytes = self.to_bytes();
        let mut addr_bytes = [0_u8; 20];

        for (i, &b) in bytes.iter().rev().enumerate() {
            if i == 20 {
                break;
            }
            addr_bytes[i] = b
        }

        Address::from_bytes(&addr_bytes)
    }

    pub fn verify(&self, msg: &[u8], signature: Signature) -> bool {
        if self.key.verify(msg, &signature.inner).is_err() {
            return false;
        };
        true
    }
}

impl ByteDecoding for PublicKey {
    type Target = Self;
    type Error = CryptoError;

    fn from_bytes(data: &[u8]) -> Result<PublicKey, CryptoError> {
        let res = VerifyingKey::<Secp256k1>::from_sec1_bytes(data);
        if res.is_err() {
            return Err(CryptoError::GenerateKey(
                "unable to correctly parse bytes".to_string(),
            ));
        }
        Ok(Self { key: res.unwrap() })
    }
}

impl ByteEncoding for PublicKey {
    fn to_bytes(&self) -> Vec<u8> {
        let mut buf = [0_u8; 33];

        let bytes = self.key.to_sec1_bytes();

        for (i, &v) in bytes.iter().enumerate() {
            buf[i] = v
        }

        buf.to_vec()
    }
}

impl HexEncoding for PublicKey {
    fn to_hex(&self) -> String {
        hex::encode(self.to_bytes())
    }
}

impl HexDecoding for PublicKey {
    type Target = Self;
    type Error = CryptoError;

    fn from_hex(hex_str: &str) -> Result<Self, CryptoError> {
        let res = hex::decode(hex_str);
        if res.is_err() {
            return Err(CryptoError::GenerateKey(
                "unable to correctly parse hex string".to_string(),
            ));
        }
        let bytes = res.unwrap();

        Self::from_bytes(&bytes)
    }
}

impl Display for PublicKey {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.to_hex())
    }
}

mod test {
    use super::*;

    #[test]
    fn test_public_key() {
        let pvt_key = PrivateKey::new();
        let pub_key = pvt_key.pub_key();

        let pub_bytes = pub_key.to_bytes();
        let pub_hex = pub_key.to_hex();

        let pub_key_2 = PublicKey::from_bytes(&pub_bytes).unwrap();

        assert_eq!(pub_key.to_hex(), pub_key_2.to_hex());

        let pub_key_3 = PublicKey::from_hex(&pub_hex).unwrap();

        assert_eq!(pub_key.to_hex(), pub_key_3.to_hex());

        assert_eq!(pub_key.to_bytes().len(), 33);
        assert_eq!(66, pub_key.to_hex().len());
    }
}
