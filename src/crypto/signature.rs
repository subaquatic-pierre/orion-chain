use bytes::Bytes;
use ecdsa::Signature as ECDASignature;
use k256::Secp256k1;
use serde::{Deserialize, Serialize};
use serde_with::{serde_as, SerializeAs};
use std::{fmt::Display, ops::Deref};

use crate::core::{
    encoding::{ByteEncoding, HexEncoding},
    error::CoreError,
};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Signature {
    pub inner: ECDASignature<Secp256k1>,
}

impl Signature {
    pub fn new(signature: ECDASignature<Secp256k1>) -> Self {
        Self { inner: signature }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SignatureBytes {
    #[serde(with = "serde_bytes")]
    inner: [u8; 64],
}

impl SignatureBytes {
    pub fn new(data: &[u8]) -> Self {
        let mut buf = [0_u8; 64];
        for (i, b) in data.iter().enumerate() {
            buf[i] = b.clone()
        }

        Self { inner: buf }
    }
}

impl ByteEncoding<Signature> for SignatureBytes {
    fn from_bytes(data: &[u8]) -> Result<Signature, CoreError> {
        Signature::from_bytes(data)
    }

    fn to_bytes(&self) -> Result<Vec<u8>, CoreError> {
        Ok(self.inner.to_vec())
    }
}

impl Deref for SignatureBytes {
    type Target = [u8; 64];
    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl HexEncoding<Signature> for Signature {
    fn from_hex(data: &str) -> Result<Signature, CoreError> {
        let bytes = hex::decode(data)?;

        match ECDASignature::from_slice(&bytes) {
            Ok(sig) => Ok(Self { inner: sig }),
            Err(e) => Err(CoreError::Parsing(format!(
                "unable to generate signature from bytes: {e}"
            ))),
        }

        // Ok(Self::from_bytes(&hex::decode(data)?)?)
    }

    fn to_hex(&self) -> Result<String, CoreError> {
        Ok(hex::encode(&self.to_bytes()?))
    }
}

impl ByteEncoding<Signature> for Signature {
    fn to_bytes(&self) -> Result<Vec<u8>, CoreError> {
        Ok(self.inner.to_vec())
    }

    fn from_bytes(bytes: &[u8]) -> Result<Signature, CoreError> {
        match ECDASignature::from_slice(bytes) {
            Ok(sig) => Ok(Self { inner: sig }),
            Err(e) => Err(CoreError::Parsing(format!(
                "unable to generate signature from bytes: {e}"
            ))),
        }
    }
}

impl Display for Signature {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.to_hex().unwrap())
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::crypto::private_key::PrivateKey;

    #[test]
    fn test_signature() {
        let pvt_key = PrivateKey::new();
        let _pub_key = pvt_key.pub_key();

        let pvt_key_2 = PrivateKey::new();
        let _pub_key_2 = pvt_key_2.pub_key();

        let msg = b"Hello world";

        let sig = pvt_key.sign(msg);
        let sig_bytes = sig.to_bytes().unwrap();

        assert_eq!(sig_bytes.len(), 64);

        let sig_2 = Signature::from_bytes(&sig_bytes);

        assert_eq!(sig_2.is_err(), false);
        let sig_2 = sig_2.unwrap();

        assert_eq!(sig.to_hex().unwrap(), sig_2.to_hex().unwrap());

        let sig_3 = Signature::from_hex(&sig_2.to_hex().unwrap());

        assert_eq!(sig_3.is_err(), false);
        let sig_3 = sig_3.unwrap();

        assert_eq!(sig.to_hex().unwrap(), sig_3.to_hex().unwrap());
    }
}
