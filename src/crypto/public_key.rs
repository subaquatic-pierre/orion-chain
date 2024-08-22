use borsh::{BorshDeserialize, BorshSerialize};
use ecdsa::{signature::Verifier, VerifyingKey};
use k256::Secp256k1;
use serde::{de::Visitor, Deserialize, Serialize};
use std::{
    fmt::{Display, Formatter, Result as FmtResult},
    ops::Deref,
};

use crate::core::{
    encoding::{ByteEncoding, HexEncoding},
    error::CoreError,
};

use super::{address::Address, error::CryptoError, signature::Signature};

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct PublicKey {
    key: VerifyingKey<Secp256k1>,
}

impl PublicKey {
    pub fn new(key: VerifyingKey<Secp256k1>) -> Self {
        Self { key }
    }

    pub fn address(&self) -> Result<Address, CryptoError> {
        let bytes = self.to_bytes()?;
        let mut addr_bytes = [0_u8; 20];

        for (i, &b) in bytes.iter().rev().enumerate() {
            if i == 20 {
                break;
            }
            addr_bytes[i] = b
        }

        Ok(Address::from_bytes(&addr_bytes)?)
    }

    pub fn verify(&self, msg: &[u8], signature: &Signature) -> bool {
        if self.key.verify(msg, &signature.inner).is_err() {
            return false;
        };
        true
    }
}

impl ByteEncoding<PublicKey> for PublicKey {
    fn from_bytes(data: &[u8]) -> Result<PublicKey, CoreError> {
        let res = VerifyingKey::<Secp256k1>::from_sec1_bytes(data);
        if res.is_err() {
            return Err(CoreError::Parsing(
                "unable to correctly parse bytes".to_string(),
            ));
        }
        Ok(Self { key: res.unwrap() })
    }

    fn to_bytes(&self) -> Result<Vec<u8>, CoreError> {
        let mut buf = [0_u8; 33];

        let bytes = self.key.to_sec1_bytes();

        for (i, &v) in bytes.iter().enumerate() {
            buf[i] = v
        }

        Ok(buf.to_vec())
    }
}

impl HexEncoding<PublicKey> for PublicKey {
    fn to_hex(&self) -> Result<String, CoreError> {
        Ok(hex::encode(&self.to_bytes()?))
    }

    fn from_hex(data: &str) -> Result<PublicKey, CoreError> {
        Ok(Self::from_bytes(&hex::decode(data)?)?)
    }
}

impl Display for PublicKey {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.to_hex().unwrap())
    }
}

impl From<PublicKeyBytes> for PublicKey {
    fn from(value: PublicKeyBytes) -> Self {
        Self::from_bytes(value.as_ref()).unwrap()
    }
}

#[derive(Debug, Clone, PartialEq, BorshDeserialize, BorshSerialize)]
pub struct PublicKeyBytes([u8; 33]);

impl PublicKeyBytes {
    pub fn new(data: &[u8]) -> Result<Self, CoreError> {
        let mut buf = [0_u8; 33];

        if data.len() != 33 {
            return Err(CoreError::Parsing(
                "incorrect data length for new PublicKeyBytes".to_string(),
            ));
        }
        for (i, b) in data.iter().enumerate() {
            buf[i] = b.clone();
        }
        Ok(Self(buf))
    }
}

impl Serialize for PublicKeyBytes {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.to_hex().unwrap())
    }
}
pub struct PublicKeyBytesVisitor;

impl<'de> Deserialize<'de> for PublicKeyBytes {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        deserializer.deserialize_str(PublicKeyBytesVisitor)
    }
}

impl<'de> Visitor<'de> for PublicKeyBytesVisitor {
    type Value = PublicKeyBytes;
    fn expecting(&self, formatter: &mut Formatter) -> FmtResult {
        formatter.write_str("Hex &str value")
    }

    fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        match PublicKeyBytes::from_hex(v) {
            Ok(hash) => Ok(hash),
            Err(e) => Err(E::custom(format!("{e}"))),
        }
    }
}

impl ByteEncoding<PublicKeyBytes> for PublicKeyBytes {
    fn from_bytes(data: &[u8]) -> Result<PublicKeyBytes, CoreError> {
        Ok(PublicKeyBytes::new(data)?)
    }

    fn to_bytes(&self) -> Result<Vec<u8>, CoreError> {
        Ok(self.0.to_vec())
    }
}

impl HexEncoding<PublicKeyBytes> for PublicKeyBytes {
    fn from_hex(data: &str) -> Result<PublicKeyBytes, CoreError> {
        let bytes = hex::decode(data)?;
        PublicKeyBytes::new(&bytes)
    }

    fn to_hex(&self) -> Result<String, CoreError> {
        Ok(hex::encode(&self.to_bytes()?))
    }
}

impl Deref for PublicKeyBytes {
    type Target = [u8; 33];
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

mod tests {

    #[test]
    fn test_public_key() {
        use super::*;
        use crate::crypto::private_key::PrivateKey;

        let pvt_key = PrivateKey::new();
        let pub_key = pvt_key.pub_key();

        let pub_bytes = pub_key.to_bytes().unwrap();
        let pub_hex = pub_key.to_hex().unwrap();

        let pub_key_2 = PublicKey::from_bytes(&pub_bytes).unwrap();

        assert_eq!(pub_key.to_hex().unwrap(), pub_key_2.to_hex().unwrap());

        let pub_key_3 = PublicKey::from_hex(&pub_hex).unwrap();

        assert_eq!(pub_key.to_hex().unwrap(), pub_key_3.to_hex().unwrap());

        assert_eq!(pub_key.to_bytes().unwrap().len(), 33);
        assert_eq!(66, pub_key.to_hex().unwrap().len());
    }
}
