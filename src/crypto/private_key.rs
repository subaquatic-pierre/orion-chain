use crate::core::{
    encoding::{ByteEncoding, HexEncoding},
    error::CoreError,
};
use ecdsa::{
    elliptic_curve::rand_core::OsRng, signature::Signer, Signature as ECDASignature, SigningKey,
    VerifyingKey,
};
use k256::Secp256k1;
use pem::{encode, parse, Pem};
use serde::{Deserialize, Serialize};
use std::io::Write;
use std::{fmt::Display, fs::File};
use std::{io::Read, path::Path};

use super::{address::Address, error::CryptoError, public_key::PublicKey, signature::Signature};

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

    pub fn address(&self) -> Address {
        self.pub_key().address().unwrap()
    }

    pub fn pub_key(&self) -> PublicKey {
        let verifying_key = VerifyingKey::from(&self.key);
        PublicKey::new(verifying_key)
    }

    pub fn sign(&self, msg: &[u8]) -> Signature {
        let sig: ECDASignature<Secp256k1> = self.key.sign(msg);

        Signature::new(sig)
    }

    pub fn from_pem(path: &Path) -> Result<Self, CoreError> {
        let mut file = File::open(path).map_err(|e| CoreError::Parsing(e.to_string()))?;
        let mut pem_data = Vec::new();
        file.read_to_end(&mut pem_data)
            .map_err(|e| CoreError::Parsing(e.to_string()))?;

        let pem = parse(&pem_data).map_err(|e| CoreError::Parsing(e.to_string()))?;

        let private_key_bytes = pem.contents();
        let private_key = PrivateKey::from_bytes(private_key_bytes)?;

        Ok(private_key)
    }

    pub fn write_pem(&self, path: &Path) -> Result<(), CoreError> {
        let bytes = self.to_bytes()?;

        // Create PEM encoding
        let pem = Pem::new(path.to_string_lossy(), bytes);
        // Write PEM to file
        match File::create(path) {
            Ok(mut file) => {
                let encode = encode(&pem);
                if let Err(e) = file.write_all(encode.as_bytes()) {
                    return Err(CoreError::Serialize(e.to_string()));
                }
                Ok(())
            }
            Err(e) => Err(CoreError::Serialize(e.to_string())),
        }
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
        Ok(self.key.to_bytes().to_vec())
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
mod tests {
    use std::fs;

    use crate::crypto::public_key;

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
        let is_valid = pub_key.verify(msg, &sig);

        let not_valid = pub_key_2.verify(msg, &sig);

        assert_eq!(is_valid, true);
        assert_eq!(not_valid, false);
    }

    #[test]
    fn test_pem() {
        let file_path = Path::new("private_key.pem");
        let pvt_key = PrivateKey::new();

        let data = [1, 2, 3, 4];

        let sign = pvt_key.sign(&data);

        pvt_key.write_pem(file_path).unwrap();

        let from_file = PrivateKey::from_pem(file_path).unwrap();

        let pub_key = from_file.pub_key();

        let val = pub_key.verify(&data, &sign);

        fs::remove_file(file_path).unwrap();

        assert_eq!(val, true);
    }
}
