use log::{debug, info};
use serde::{Deserialize, Serialize};
use serde_with::base64::{Base64, Bcrypt, BinHex, Standard};
use serde_with::serde_as;

use crate::crypto::{
    hash::Hash,
    private_key::PrivateKey,
    public_key::{PublicKey, PublicKeyBytes},
    signature::{Signature, SignatureBytes},
    utils::random_bytes,
};

use super::{
    encoding::{ByteEncoding, HexEncoding},
    error::CoreError,
};

#[serde_as]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Transaction {
    #[serde_as(as = "Base64")]
    pub data: Vec<u8>,
    pub hash: Hash,
    pub signature: Option<SignatureBytes>,
    pub signer: Option<PublicKeyBytes>,
}

impl Transaction {
    pub fn new(data: &[u8]) -> Result<Self, CoreError> {
        let data = data.to_vec();
        let hash = Hash::sha256(&data)?;
        Ok(Self {
            data,
            signature: None,
            signer: None,
            hash,
        })
    }

    pub fn hash(&self) -> Hash {
        self.hash
    }

    pub fn data_str(&self) -> String {
        String::from_utf8_lossy(&self.data).to_string()
    }

    pub fn sign(&mut self, private_key: &PrivateKey) -> Result<(), CoreError> {
        if self.signer.is_some() | self.signature.is_some() {
            return Err(CoreError::Transaction(
                "transaction already is already signed".to_string(),
            ));
        }

        let sig = private_key.sign(&self.data);
        let sig_bytes = SignatureBytes::new(&sig.to_bytes()?)?;
        let pub_key_bytes = PublicKeyBytes::new(&private_key.pub_key().to_bytes()?)?;

        self.signer = Some(pub_key_bytes);
        self.signature = Some(sig_bytes);

        Ok(())
    }

    pub fn verify(&self) -> Result<(), CoreError> {
        if self.signature.is_none() {
            return Err(CoreError::Transaction(
                "transaction has no signature".to_string(),
            ));
        }

        match (&self.signer, &self.signature) {
            (Some(key_bytes), Some(sig_bytes)) => {
                let key = PublicKey::from_bytes(&key_bytes.to_bytes()?)?;
                let signature = Signature::from_bytes(&sig_bytes.to_bytes()?)?;

                if !key.verify(&self.data, &signature) {
                    return Err(CoreError::Transaction(
                        "invalid transaction signature".to_string(),
                    ));
                }
            }
            _ => {
                return Err(CoreError::Transaction(
                    "transaction has no public key or signature".to_string(),
                ));
            }
        }
        Ok(())
    }
}

impl ByteEncoding<Transaction> for Transaction {
    fn from_bytes(data: &[u8]) -> Result<Transaction, CoreError> {
        Ok(bincode::deserialize(data)?)
    }

    fn to_bytes(&self) -> Result<Vec<u8>, CoreError> {
        Ok(bincode::serialize(&self)?)
    }
}

impl HexEncoding<Transaction> for Transaction {
    fn from_hex(data: &str) -> Result<Transaction, CoreError> {
        Ok(Self::from_bytes(&hex::decode(data)?)?)
    }

    fn to_hex(&self) -> Result<String, CoreError> {
        Ok(hex::encode(self.to_bytes()?))
    }
}

#[cfg(test)]
mod test {
    use super::*;
    #[test]
    fn test_transaction_sign() {
        let priv_key = PrivateKey::new();
        let data = b"Hello world, Data is cool";

        let mut tx = Transaction::new(data).unwrap();

        assert!(matches!(tx.verify(), Err(_)));

        tx.sign(&priv_key).unwrap();
        assert!(tx.verify().is_ok());

        // try change data
        tx.data = b"changed data".to_vec();
        assert!(matches!(tx.verify(), Err(_)));

        let priv_key = PrivateKey::new();
        let data = b"Hello world, Data is cool";

        let mut tx = Transaction::new(data).unwrap();

        // try double sign
        tx.sign(&priv_key).unwrap();
        assert!(matches!(tx.sign(&priv_key), Err(_)));
    }

    #[test]
    fn test_transaction_data_str() {
        let _priv_key = PrivateKey::new();
        let data = b"Hello world, Data is cool";

        let tx = Transaction::new(data).unwrap();
        assert_eq!(tx.data_str(), "Hello world, Data is cool");
    }

    #[test]
    fn test_transaction_parse_bytes() {
        let priv_key = PrivateKey::new();
        let data = b"Hello world, Data is cool";

        let mut tx = Transaction::new(data).unwrap();

        tx.sign(&priv_key).unwrap();
        let bytes = &tx.to_bytes().unwrap();

        let tx_1_sig = tx.signature.unwrap();

        let tx_2 = Transaction::from_bytes(&bytes);

        assert!(tx_2.is_ok());

        let tx_2 = tx_2.unwrap();

        assert_eq!(tx_2.data_str(), "Hello world, Data is cool");

        assert!(tx_2.verify().is_ok());

        let tx_2_sig = tx_2.signature.unwrap();
        let tx_2_pub_key = tx_2.signer.unwrap();

        let pub_key = priv_key.pub_key().to_bytes().unwrap();

        assert_eq!(tx_1_sig, tx_2_sig);
        assert_eq!(tx_2_pub_key.to_bytes().unwrap(), pub_key)
    }

    #[test]
    fn test_transaction_parse_hex() {
        let priv_key = PrivateKey::new();
        let data = b"Hello world, Data is cool";

        let mut tx = Transaction::new(data).unwrap();
        let _hex_str = tx.to_hex().unwrap();

        tx.sign(&priv_key).unwrap();
        let hex_str = tx.to_hex().unwrap();

        let _tx_1_hash = tx.hash();
        let tx_1_sig = tx.signature.unwrap();

        let tx_2 = Transaction::from_hex(&hex_str);

        assert!(tx_2.is_ok());

        let tx_2 = tx_2.unwrap();

        assert_eq!(tx_2.data_str(), "Hello world, Data is cool");

        assert!(tx_2.verify().is_ok());

        let tx_2_hash = tx_2.hash();
        let tx_2_sig = tx_2.signature.unwrap();

        assert_eq!(tx_2_hash.len(), 32);

        let tx_2_pub_key = tx_2.signer.unwrap();

        let pub_key = priv_key.pub_key().to_bytes().unwrap();

        assert_eq!(tx_1_sig, tx_2_sig);
        assert_eq!(tx_2_pub_key.to_bytes().unwrap(), pub_key);

        assert_eq!(tx_2_hash, tx_2_hash);
    }
}

pub fn random_tx() -> Transaction {
    let bytes = random_bytes(8);
    Transaction::new(&bytes).unwrap()
}

pub fn random_signed_tx() -> Transaction {
    let mut tx = random_tx();
    let pvt = PrivateKey::new();
    tx.sign(&pvt).unwrap();
    tx
}
