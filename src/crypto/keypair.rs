use ecdsa::{
    elliptic_curve::{rand_core::OsRng, NonZeroScalar},
    signature::{DigestVerifier, Signer, Verifier},
    Signature as ECDASignature, SigningKey, VerifyingKey,
};

use sha256::digest;

use k256::{Secp256k1, SecretKey, U256};

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

    pub fn sign(&self, msg: &[u8]) -> Signature {
        let d_str = digest(msg);
        let d = d_str.as_bytes();
        let sig: ECDASignature<Secp256k1> = self.key.sign(d);

        Signature { inner: sig }
    }
}

#[derive(Clone)]
pub struct Signature {
    inner: ECDASignature<Secp256k1>,
}

impl Signature {}

pub struct Address {
    inner: [u8; 20],
}

impl Address {
    pub fn to_hex(&self) -> String {
        hex::encode(self.inner)
    }
    pub fn to_bytes(&self) -> [u8; 20] {
        self.inner
    }

    pub fn from_hex(hex_str: &str) -> Result<Self, KeyPairError> {
        if hex_str.len() != 40 {
            return Err(KeyPairError::GenerateError(
                "incorrect hex format for address".to_string(),
            ));
        }

        let bytes = hex::decode(hex_str);

        if bytes.is_err() {
            return Err(KeyPairError::GenerateError(
                "unable to generate bytes from hex".to_string(),
            ));
        }

        let bytes = bytes.unwrap();

        Self::from_bytes(&bytes)
    }

    pub fn from_bytes(bytes: &[u8]) -> Result<Self, KeyPairError> {
        let mut buf = [0_u8; 20];
        if bytes.len() != 20 {
            return Err(KeyPairError::GenerateError(
                "incorrect byte format for address".to_string(),
            ));
        }

        for (i, &b) in bytes.iter().enumerate() {
            buf[i] = b
        }

        Ok(Self { inner: buf })
    }
}

struct PublicKey {
    key: VerifyingKey<Secp256k1>,
}

impl PublicKey {
    pub fn address(&self) -> Result<Address, KeyPairError> {
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

    pub fn verify(&self, msg: &[u8], signature: Signature) -> bool {
        let d_str = digest(msg);
        let d = d_str.as_bytes();
        if self.key.verify(d, &signature.inner).is_err() {
            return false;
        };
        true
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
