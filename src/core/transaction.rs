use crate::crypto::{
    hash::Hash, private_key::PrivateKey, public_key::PublicKey, signature::Signature,
    utils::random_bytes,
};

use super::{
    encoding::{ByteDecoding, ByteEncoding, HexDecoding, HexEncoding},
    error::CoreError,
};

#[derive(Debug, Clone, PartialEq)]
pub struct Transaction {
    data_len: u64,
    pub data: Vec<u8>,
    pub hash: Hash,
    pub signature: Option<Signature>,
    pub signer: Option<PublicKey>,
}

impl Transaction {
    pub fn new(data: &[u8]) -> Self {
        let data = data.to_vec();
        let data_len: u64 = data.len() as u64;
        let hash = Hash::sha256(&data).unwrap();
        Self {
            data,
            data_len,
            hash,
            signature: None,
            signer: None,
        }
    }

    pub fn hash(&self) -> Hash {
        self.hash.clone()
    }

    pub fn data_str(&self) -> String {
        String::from_utf8_lossy(&self.data).to_string()
    }

    pub fn sign(&mut self, private_key: PrivateKey) -> Result<(), CoreError> {
        if self.signer.is_some() | self.signature.is_some() {
            return Err(CoreError::Transaction(
                "transaction already is already signed".to_string(),
            ));
        }

        let sig = private_key.sign(&self.data);
        self.signer = Some(private_key.pub_key());
        self.signature = Some(sig);

        Ok(())
    }

    pub fn verify(&self) -> Result<(), CoreError> {
        if self.signature.is_none() {
            return Err(CoreError::Transaction(
                "transaction has no signature".to_string(),
            ));
        }

        match &self.signer {
            Some(key) => {
                if !key.verify(&self.data, self.signature.clone().unwrap()) {
                    return Err(CoreError::Transaction(
                        "invalid transaction signature".to_string(),
                    ));
                }
            }
            None => {
                return Err(CoreError::Transaction(
                    "transaction has no public key".to_string(),
                ));
            }
        }
        Ok(())
    }
}

impl ByteEncoding for Transaction {
    fn to_bytes(&self) -> Vec<u8> {
        let mut buf: Vec<u8> = vec![];

        let data_len = self.data_len.to_be_bytes();
        // append data length
        buf.extend_from_slice(&data_len);

        // append data
        buf.extend_from_slice(&self.data);

        // append hash
        buf.extend_from_slice(&self.hash.to_bytes());

        // append signature
        let sig_bytes = match &self.signature {
            Some(sig) => {
                let mut bytes = vec![1_u8];
                bytes.extend_from_slice(&sig.to_bytes());
                bytes
            }
            None => vec![0_u8],
        };
        buf.extend_from_slice(&sig_bytes);

        // append public key
        let sig_bytes = match &self.signer {
            Some(key) => {
                let mut bytes = vec![1_u8];
                bytes.extend_from_slice(&key.to_bytes());
                bytes
            }
            None => vec![0_u8],
        };
        buf.extend_from_slice(&sig_bytes);

        buf
    }
}

impl ByteDecoding for Transaction {
    type Target = Self;
    type Error = CoreError;

    fn from_bytes(data: &[u8]) -> Result<Transaction, CoreError> {
        if data.len() < 8 {
            return Err(CoreError::Transaction(
                "incorrectly formatted bytes, no data length in bytes".to_string(),
            ));
        }

        // create data buffer
        let mut data_buf: Vec<u8> = vec![];

        // get length of data bytes
        let data_len: usize = usize::from_be_bytes(data[0..8].try_into().unwrap());

        // hold offset for data bytes,
        // will be used as index for signature start
        let mut offset = 8 + data_len;

        // fill data buffer
        for (i, &b) in data.iter().skip(8).enumerate() {
            // reached end of data
            if i == data_len {
                break;
            }
            data_buf.push(b);

            // data_index += 1;
        }

        let hash = Hash::from_bytes(&data[offset..offset + 32]);

        if hash.is_err() {
            return Err(CoreError::Transaction("unable to parse hash".to_string()));
        }

        let hash = hash.unwrap();

        offset += 32;

        // get signature
        let has_sig_byte = u8::from_be_bytes(data[offset..offset + 1].try_into().unwrap());

        // inc offset for has sig byte
        offset += 1;

        let signature: Option<Signature> = if has_sig_byte == 0 {
            None
        } else {
            match Signature::from_bytes(&data[offset..offset + 64]) {
                Ok(sig) => Some(sig),
                Err(e) => {
                    return Err(CoreError::Transaction(format!(
                        "unable to parse signature {e}"
                    )))
                }
            }
        };

        // length of signature, inc offset of signature
        offset += 64;

        // get public key;
        let has_pub_key_byte = u8::from_be_bytes(data[offset..offset + 1].try_into().unwrap());

        // inc offset for has key byte
        offset += 1;

        let signer: Option<PublicKey> = if has_pub_key_byte == 0 {
            None
        } else {
            match PublicKey::from_bytes(&data[offset..offset + 33]) {
                Ok(key) => Some(key),
                Err(e) => {
                    return Err(CoreError::Transaction(format!(
                        "unable to parse public key {e}"
                    )))
                }
            }
        };

        Ok(Transaction {
            data_len: data_len as u64,
            hash,
            data: data_buf,
            signature,
            signer,
        })
    }
}

impl HexEncoding for Transaction {
    fn to_hex(&self) -> String {
        let bytes = &self.to_bytes();
        hex::encode(bytes)
    }
}

impl HexDecoding for Transaction {
    type Target = Self;
    type Error = CoreError;

    fn from_hex(data: &str) -> Result<Transaction, CoreError> {
        match hex::decode(data) {
            Ok(bytes) => Self::from_bytes(&bytes),
            Err(e) => {
                return Err(CoreError::Transaction(format!(
                    "unable to parse hex string {e})"
                )))
            }
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    #[test]
    fn test_transaction_sign() {
        let priv_key = PrivateKey::new();
        let data = b"Hello world, Data is cool";

        let mut tx = Transaction::new(data);

        assert!(matches!(tx.verify(), Err(_)));

        tx.sign(priv_key).unwrap();
        assert!(tx.verify().is_ok());

        // try change data
        tx.data = b"changed data".to_vec();
        assert!(matches!(tx.verify(), Err(_)));

        let priv_key = PrivateKey::new();
        let data = b"Hello world, Data is cool";

        let mut tx = Transaction::new(data);

        // try double sign
        tx.sign(priv_key.clone()).unwrap();
        assert!(matches!(tx.sign(priv_key), Err(_)));
    }

    #[test]
    fn test_transaction_data_str() {
        let priv_key = PrivateKey::new();
        let data = b"Hello world, Data is cool";

        let mut tx = Transaction::new(data);
        assert_eq!(tx.data_str(), "Hello world, Data is cool");
    }

    #[test]
    fn test_transaction_parse_bytes() {
        let priv_key = PrivateKey::new();
        let data = b"Hello world, Data is cool";

        let mut tx = Transaction::new(data);
        let bytes = tx.to_bytes();
        assert_eq!(bytes.len(), 67);

        tx.sign(priv_key.clone());
        let bytes = tx.to_bytes();

        let tx_1_sig = tx.signature.unwrap();

        // data_len = 8 bytes
        // data = 25 bytes
        // signature = 64 bytes
        // public_key = 33 bytes
        // total = 132 bytes
        assert_eq!(bytes.len(), 164);

        let tx_2 = Transaction::from_bytes(&bytes);

        assert!(tx_2.is_ok());

        let tx_2 = tx_2.unwrap();

        assert_eq!(tx_2.data_str(), "Hello world, Data is cool");

        assert!(tx_2.verify().is_ok());

        let tx_2_sig = tx_2.signature.unwrap();
        let tx_2_pub_key = tx_2.signer.unwrap();

        let pub_key = priv_key.pub_key();

        assert_eq!(tx_1_sig.to_string(), tx_2_sig.to_string());
        assert_eq!(tx_2_pub_key.to_string(), pub_key.to_string())
    }

    #[test]
    fn test_transaction_parse_hex() {
        let priv_key = PrivateKey::new();
        let data = b"Hello world, Data is cool";

        let mut tx = Transaction::new(data);
        let hex_str = tx.to_hex();
        assert_eq!(hex_str.len(), 134);

        tx.sign(priv_key.clone());
        let hex_str = tx.to_hex();

        let tx_1_hash = tx.hash();
        let tx_1_sig = tx.signature.unwrap();

        // data_len = 8 bytes
        // data = 25 bytes
        // signature = 64 bytes
        // public_key = 33 bytes
        // hex = 32 bytes
        // total_hex = 132 bytes * 2
        assert_eq!(hex_str.len(), 328);

        let tx_2 = Transaction::from_hex(&hex_str);

        assert!(tx_2.is_ok());

        let tx_2 = tx_2.unwrap();

        assert_eq!(tx_2.data_str(), "Hello world, Data is cool");

        assert!(tx_2.verify().is_ok());

        let tx_2_hash = tx_2.hash();
        let tx_2_sig = tx_2.signature.unwrap();

        assert_eq!(tx_2_hash.len(), 32);

        let tx_2_pub_key = tx_2.signer.unwrap();

        let pub_key = priv_key.pub_key();

        assert_eq!(tx_1_sig.to_string(), tx_2_sig.to_string());
        assert_eq!(tx_2_pub_key.to_string(), pub_key.to_string());

        assert_eq!(tx_2_hash, tx_2_hash);
    }
}

pub fn random_tx() -> Transaction {
    let bytes = random_bytes(8);
    Transaction::new(&bytes)
}

pub fn random_signed_tx() -> Transaction {
    let mut tx = random_tx();
    let pvt = PrivateKey::new();
    tx.sign(pvt).unwrap();
    tx
}
