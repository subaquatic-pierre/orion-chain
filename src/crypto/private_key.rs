use ecdsa::{
    elliptic_curve::rand_core::OsRng, signature::Signer, Signature as ECDASignature, SigningKey,
    VerifyingKey,
};
use k256::Secp256k1;
use serde::{Deserialize, Serialize};
use std::fmt::Display;

use crate::core::{
    encoding::{ByteEncoding, HexEncoding},
    error::CoreError,
};

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

impl ByteEncoding<PrivateKey> for PrivateKey {
    fn from_bytes(bytes: &[u8]) -> Result<PrivateKey, CoreError> {
        // let mut _bytes = [0_u8; 32];
        if bytes.len() != 32 {
            return Err(CoreError::Parsing(
                "unable to correctly parse bytes".to_string(),
            ));
        }

        Ok(Self {
            key: SigningKey::<Secp256k1>::from_bytes(bytes.into()).unwrap(),
        })
    }

    fn to_bytes(&self) -> Result<Vec<u8>, CoreError> {
        todo!()
        // Ok(bincode::serialize(&self)?)
    }
}

impl HexEncoding<PrivateKey> for PrivateKey {
    fn from_hex(data: &str) -> Result<PrivateKey, CoreError> {
        Ok(Self::from_bytes(&hex::decode(data)?)?)
    }

    fn to_hex(&self) -> Result<String, CoreError> {
        Ok(hex::encode(&self.to_bytes()?))
    }
}

impl Display for PrivateKey {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.to_hex().unwrap())
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_private_key() {
        let pvt_key = PrivateKey::new();

        assert_eq!(pvt_key.to_bytes().unwrap().len(), 32);

        let bytes = pvt_key.to_bytes().unwrap();

        let pvt_key_2 = PrivateKey::from_bytes(&bytes).expect("unable to create private key");

        assert_eq!(pvt_key.to_hex().unwrap(), pvt_key_2.to_hex().unwrap());
        assert_eq!(64, pvt_key_2.to_hex().unwrap().len());

        let hex = pvt_key.to_hex().unwrap();
        let new_pvt_key = PrivateKey::from_hex(&hex).expect("unable to create private key");

        assert_eq!(pvt_key.to_hex().unwrap(), new_pvt_key.to_hex().unwrap());
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
