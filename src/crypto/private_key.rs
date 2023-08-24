use ecdsa::{
    elliptic_curve::rand_core::OsRng, signature::Signer, Signature as ECDASignature, SigningKey,
    VerifyingKey,
};
use k256::Secp256k1;
use std::fmt::Display;

use crate::core::encoding::{ByteDecoding, ByteEncoding, HexDecoding, HexEncoding};

use super::{error::CryptoError, public_key::PublicKey, signature::Signature};

#[derive(Clone)]
pub struct PrivateKey {
    key: SigningKey<Secp256k1>,
}

impl PrivateKey {
    pub fn new() -> Self {
        Self {
            key: SigningKey::random(&mut OsRng),
        }
    }

    pub fn pub_key(&self) -> PublicKey {
        let verifying_key = VerifyingKey::from(&self.key);
        PublicKey::new(verifying_key)
    }

    pub fn sign(&self, msg: &[u8]) -> Signature {
        let sig: ECDASignature<Secp256k1> = self.key.sign(msg);

        Signature::new(sig)
    }
}

impl ByteDecoding for PrivateKey {
    type Target = Self;
    type Error = CryptoError;

    fn from_bytes(bytes: &[u8]) -> Result<Self, CryptoError> {
        let mut _bytes = [0_u8; 32];
        if bytes.len() != 32 {
            return Err(CryptoError::GenerateKey(
                "unable to correctly parse bytes".to_string(),
            ));
        }

        for (i, &b) in bytes.iter().enumerate() {
            _bytes[i] = b
        }

        Ok(Self {
            key: SigningKey::<Secp256k1>::from_bytes(&_bytes.into()).unwrap(),
        })
    }
}

impl ByteEncoding for PrivateKey {
    fn to_bytes(&self) -> Vec<u8> {
        let mut buf = [0_u8; 32];
        for (i, v) in self.key.to_bytes().iter().copied().enumerate() {
            buf[i] = v
        }
        buf.to_vec()
    }
}

impl HexDecoding for PrivateKey {
    type Target = Self;
    type Error = CryptoError;

    fn from_hex(hex_str: &str) -> Result<Self, CryptoError> {
        if hex_str.len() != 64 {
            panic!("unable to correctly parse hex string");
        }

        let h_bytes = hex::decode(hex_str).unwrap();
        if Self::from_bytes(&h_bytes).is_err() {
            return Err(CryptoError::GenerateKey(
                "unable to correctly parse hex string".to_string(),
            ));
        }

        Self::from_bytes(&h_bytes)
    }
}

impl HexEncoding for PrivateKey {
    fn to_hex(&self) -> String {
        hex::encode(self.to_bytes())
    }
}

impl Display for PrivateKey {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.to_hex())
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_private_key() {
        let pvt_key = PrivateKey::new();

        assert_eq!(pvt_key.to_bytes().len(), 32);

        let bytes = pvt_key.to_bytes();

        let pvt_key_2 = PrivateKey::from_bytes(&bytes).expect("unable to create private key");

        assert_eq!(pvt_key.to_hex(), pvt_key_2.to_hex());
        assert_eq!(64, pvt_key_2.to_hex().len());

        let hex = pvt_key.to_hex();
        let new_pvt_key = PrivateKey::from_hex(&hex).expect("unable to create private key");

        assert_eq!(pvt_key.to_hex(), new_pvt_key.to_hex());
    }

    #[test]
    fn test_sign() {
        let pvt_key = PrivateKey::new();
        let pub_key = pvt_key.pub_key();

        let pvt_key_2 = PrivateKey::new();
        let pub_key_2 = pvt_key_2.pub_key();

        let msg = b"Hello world";

        let sig = pvt_key.sign(msg);
        let is_valid = pub_key.verify(msg, sig.clone());

        let not_valid = pub_key_2.verify(msg, sig);

        assert_eq!(is_valid, true);
        assert_eq!(not_valid, false);
    }
}
