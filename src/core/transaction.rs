use std::error::Error;

use crate::crypto::{private_key::PrivateKey, public_key::PublicKey, signature::Signature};

use super::{encoding::ByteEncoding, error::CoreError};

pub struct Transaction {
    pub data: Vec<u8>,
    pub signature: Option<Signature>,
    data_len: usize,
    public_key: Option<PublicKey>,
}

impl Transaction {
    pub fn new(data: &[u8]) -> Self {
        let data = data.to_vec();
        let data_len = data.len();
        Self {
            data,
            data_len,
            signature: None,
            public_key: None,
        }
    }

    pub fn sign(&mut self, private_key: PrivateKey) {
        let sig = private_key.sign(&self.data);
        self.public_key = Some(private_key.pub_key());
        self.signature = Some(sig);
    }

    pub fn verify(&self) -> Result<(), CoreError> {
        if self.signature.is_none() {
            return Err(CoreError::Transaction(
                "transaction has no signature".to_string(),
            ));
        }

        match &self.public_key {
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

#[cfg(test)]
mod test {
    use super::*;
    #[test]
    fn test_transaction() {
        let priv_key = PrivateKey::new();
        let data = b"Hello world, Data is cool";

        let mut tx = Transaction::new(data);

        assert!(matches!(tx.verify(), Err(_)));

        tx.sign(priv_key);
        assert!(tx.verify().is_ok());

        tx.data = b"changed data".to_vec();

        assert!(matches!(tx.verify(), Err(_)));
    }
}
