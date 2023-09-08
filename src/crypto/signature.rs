use ecdsa::Signature as ECDASignature;
use k256::Secp256k1;
use std::fmt::Display;

use crate::core::encoding::{ByteDecoding, ByteEncoding, HexDecoding, HexEncoding};

use super::error::CryptoError;

#[derive(Clone, Debug, PartialEq)]
pub struct Signature {
    pub inner: ECDASignature<Secp256k1>,
}

impl Signature {
    pub fn new(signature: ECDASignature<Secp256k1>) -> Self {
        Self { inner: signature }
    }
}

impl HexEncoding for Signature {
    fn to_hex(&self) -> String {
        hex::encode(self.to_bytes())
    }
}

impl HexDecoding for Signature {
    type Target = Self;
    type Error = CryptoError;

    fn from_hex(hex_str: &str) -> Result<Self, CryptoError> {
        let bytes = hex::decode(hex_str);
        if bytes.is_err() {
            return Err(CryptoError::SignatureError(
                "unable to parse hex from bytes".to_string(),
            ));
        }

        let bytes = bytes.unwrap();
        match ECDASignature::from_slice(&bytes) {
            Ok(sig) => Ok(Self { inner: sig }),
            Err(e) => Err(CryptoError::SignatureError(format!(
                "unable to generate signature from bytes: {e}"
            ))),
        }
    }
}

impl ByteDecoding for Signature {
    type Target = Self;
    type Error = CryptoError;

    fn from_bytes(bytes: &[u8]) -> Result<Self, CryptoError> {
        match ECDASignature::from_slice(bytes) {
            Ok(sig) => Ok(Self { inner: sig }),
            Err(e) => Err(CryptoError::SignatureError(format!(
                "unable to generate signature from bytes: {e}"
            ))),
        }
    }
}

impl ByteEncoding for Signature {
    fn to_bytes(&self) -> Vec<u8> {
        self.inner.to_vec()
    }
}

impl Display for Signature {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.to_hex())
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::crypto::private_key::PrivateKey;

    #[test]
    fn test_signature() {
        let pvt_key = PrivateKey::new();
        let pub_key = pvt_key.pub_key();

        let pvt_key_2 = PrivateKey::new();
        let pub_key_2 = pvt_key_2.pub_key();

        let msg = b"Hello world";

        let sig = pvt_key.sign(msg);
        let sig_bytes = sig.to_bytes();

        assert_eq!(sig_bytes.len(), 64);

        let sig_2 = Signature::from_bytes(&sig_bytes);

        assert_eq!(sig_2.is_err(), false);
        let sig_2 = sig_2.unwrap();

        assert_eq!(sig.to_hex(), sig_2.to_hex());

        let sig_3 = Signature::from_hex(&sig_2.to_hex());

        assert_eq!(sig_3.is_err(), false);
        let sig_3 = sig_3.unwrap();

        assert_eq!(sig.to_hex(), sig_3.to_hex());
    }
}
