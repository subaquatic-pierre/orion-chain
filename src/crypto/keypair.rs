use ecdsa::{
    elliptic_curve::rand_core::OsRng, signature::Signer, Signature, SigningKey, VerifyingKey,
};
use k256::{Secp256k1, U256};

use super::error::KeyPairError;
// use p256::{NistP256, };

struct PrivateKey {
    key: SigningKey<Secp256k1>,
}

impl PrivateKey {
    pub fn new() -> Self {
        Self {
            key: SigningKey::random(&mut OsRng),
        }
    }

    pub fn from_bytes(bytes: &[u8]) -> Result<Self, KeyPairError> {
        let mut _bytes = [0_u8; 32];
        if bytes.len() != 32 {
            return Err(KeyPairError::GenerateError(
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

    pub fn from_hex(hex_str: &str) -> Result<Self, KeyPairError> {
        if hex_str.len() != 64 {
            panic!("unable to correctly parse hex string");
        }

        let h_bytes = hex::decode(hex_str).unwrap();
        if Self::from_bytes(&h_bytes).is_err() {
            return Err(KeyPairError::GenerateError(
                "unable to correctly parse hex string".to_string(),
            ));
        }

        Self::from_bytes(&h_bytes)
    }

    pub fn to_bytes(&self) -> [u8; 32] {
        let mut buf = [0_u8; 32];
        for (i, v) in self.key.to_bytes().iter().copied().enumerate() {
            buf[i] = v
        }
        buf
    }

    pub fn to_hex(&self) -> String {
        hex::encode(self.to_bytes())
    }

    pub fn pub_key(&self) -> PublicKey {
        let verifying_key = VerifyingKey::from(&self.key);
        PublicKey { key: verifying_key }
    }
}

struct PublicKey {
    key: VerifyingKey<Secp256k1>,
}

impl PublicKey {
    pub fn to_bytes(&self) -> [u8; 33] {
        let mut buf = [0_u8; 33];

        let bytes = self.key.to_sec1_bytes();

        for (i, &v) in bytes.iter().enumerate() {
            buf[i] = v
        }

        buf
    }

    pub fn to_hex(&self) -> String {
        hex::encode(self.to_bytes())
    }

    pub fn from_hex(hex_str: &str) -> Result<Self, KeyPairError> {
        let res = hex::decode(hex_str);
        if res.is_err() {
            return Err(KeyPairError::GenerateError(
                "unable to correctly parse hex string".to_string(),
            ));
        }
        let bytes = res.unwrap();

        Self::from_bytes(&bytes)
    }

    pub fn from_bytes(bytes: &[u8]) -> Result<Self, KeyPairError> {
        let res = VerifyingKey::<Secp256k1>::from_sec1_bytes(bytes);
        if res.is_err() {
            return Err(KeyPairError::GenerateError(
                "unable to correctly parse bytes".to_string(),
            ));
        }
        Ok(Self { key: res.unwrap() })
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
