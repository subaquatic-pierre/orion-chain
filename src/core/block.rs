use std::collections::HashMap;
use std::error::Error;
use std::fs::File;
use std::io::Read;
use std::io::Write;
use std::path::PathBuf;

use borsh::{BorshDeserialize, BorshSerialize};
use log::info;
use serde::{Deserialize, Serialize};

use crate::crypto::public_key::PublicKeyBytes;
use crate::crypto::signature::SignatureBytes;
use crate::crypto::{
    hash::Hash, private_key::PrivateKey, public_key::PublicKey, signature::Signature,
};

use super::storage::DbBlockStorage;
use super::{
    block_manager::BlockManager,
    encoding::{ByteEncoding, HexEncoding},
    error::CoreError,
    header::Header,
    storage::{BlockStorage, MemoryBlockStorage},
    transaction::Transaction,
};

#[derive(Debug, Clone, BorshSerialize, BorshDeserialize, PartialEq)]
pub struct Block {
    pub header: Header,
    signer: Option<PublicKeyBytes>,
    signature: Option<SignatureBytes>,
    transactions: Vec<Transaction>,
}

impl Block {
    pub fn new(header: Header, txs: Vec<Transaction>) -> Result<Self, CoreError> {
        let mut b = Self {
            header,
            transactions: vec![],
            signer: None,
            signature: None,
        };

        for tx in &txs {
            b.add_transaction(tx.clone())?;
        }

        Ok(b)
    }

    pub fn txs(&self) -> Vec<&Transaction> {
        let mut txs = vec![];
        for tx in &self.transactions {
            txs.push(tx)
        }
        txs
    }

    pub fn header(&self) -> &Header {
        &self.header
    }

    pub fn num_txs(&self) -> usize {
        self.transactions.len()
    }

    pub fn sign(&mut self, private_key: &PrivateKey) -> Result<(), CoreError> {
        if self.signer.is_some() | self.signature.is_some() {
            return Err(CoreError::Block("block already has signature".to_string()));
        }

        let sig = private_key.sign(&self.hashable_data()?);
        let sig_bytes = SignatureBytes::new(&sig.to_bytes()?)?;
        let pub_key_bytes = PublicKeyBytes::new(&private_key.pub_key().to_bytes()?)?;

        self.signature = Some(sig_bytes);
        self.signer = Some(pub_key_bytes);

        Ok(())
    }

    pub fn verify(&self) -> Result<(), CoreError> {
        if self.signature.is_none() {
            return Err(CoreError::Block(
                "no signature exists for block".to_string(),
            ));
        }

        for tx in &self.transactions {
            tx.verify()?
        }

        match (&self.signer, &self.signature) {
            (Some(key_bytes), Some(sig_bytes)) => {
                let key = PublicKey::from_bytes(&key_bytes.to_bytes()?)?;
                let signature = Signature::from_bytes(&sig_bytes.to_bytes()?)?;

                match key.verify(&self.hashable_data()?, &signature) {
                    true => Ok(()),
                    false => Err(CoreError::Block("invalid signature".to_string())),
                }
            }
            _ => Err(CoreError::Block(
                "no signer or signature exists for block".to_string(),
            )),
        }
    }

    pub fn prev_hash(&self) -> &Hash {
        &self.header.prev_blockhash
    }

    pub fn hash(&self) -> &Hash {
        &self.header.blockhash
    }

    pub fn height(&self) -> usize {
        self.header.height as usize
    }

    // ---
    // Private Methods
    // ---

    fn add_transaction(&mut self, tx: Transaction) -> Result<(), CoreError> {
        match tx.verify() {
            Ok(_) => {
                self.transactions.push(tx);
                Ok(())
            }
            Err(e) => Err(e),
        }
    }

    // get sequential bytes of all transactions
    fn txs_bytes(&self) -> Result<Vec<u8>, CoreError> {
        let mut txs_bytes = vec![];
        for tx in self.transactions.iter() {
            let bytes = tx.to_bytes()?;
            txs_bytes.extend_from_slice(&bytes);
        }
        Ok(txs_bytes)
    }

    // get data to be hashed for the block
    fn hashable_data(&self) -> Result<Vec<u8>, CoreError> {
        let mut data = vec![];
        data.extend_from_slice(&&self.header.hashable_data());
        data.extend_from_slice(&self.txs_bytes()?);
        Ok(data)
    }
}

impl ByteEncoding<Block> for Block {
    fn to_bytes(&self) -> Result<Vec<u8>, CoreError> {
        match borsh::to_vec(self) {
            Ok(b) => Ok(b),
            Err(e) => Err(CoreError::Parsing(e.to_string())),
        }
    }

    fn from_bytes(data: &[u8]) -> Result<Block, CoreError> {
        match borsh::from_slice(data) {
            Ok(t) => Ok(t),
            Err(e) => Err(CoreError::Parsing(e.to_string())),
        }
    }
}

impl HexEncoding<Block> for Block {
    fn from_hex(data: &str) -> Result<Block, CoreError> {
        Ok(Self::from_bytes(&hex::decode(data)?)?)
    }

    fn to_hex(&self) -> Result<String, CoreError> {
        Ok(hex::encode(&self.to_bytes()?))
    }
}

#[cfg(test)]
mod test {
    use std::io::{BufWriter, Read, Write};

    use serde_json::json;

    use crate::crypto::address::random_sender_receiver;
    use crate::crypto::{
        hash::Hash, private_key::PrivateKey, public_key::PublicKey, signature::Signature,
        utils::random_hash,
    };

    use crate::core::{
        encoding::ByteEncoding, error::CoreError, header::Header, transaction::Transaction,
        util::timestamp,
    };

    use crate::core::{
        header::random_header,
        transaction::{random_signed_tx, random_tx},
    };

    use super::*;

    #[test]
    fn test_sign_block() {
        let header = random_header(0, random_hash());
        let private_key = PrivateKey::new();

        let mut block = Block::new(header, vec![]).unwrap();

        assert!(block.sign(&private_key).is_ok());

        assert!(block.signature.is_some());
        assert!(block.signer.is_some());
    }

    #[test]
    fn test_verify_block() {
        let r_hash = random_hash();
        let header = random_header(0, random_hash());
        let private_key = PrivateKey::new();

        let mut block = Block::new(header, vec![]).unwrap();

        let (sender, receiver) = random_sender_receiver();

        let mut new_tx =
            Transaction::new_transfer(sender, receiver, r_hash, b"Cool World").unwrap();
        new_tx.sign(&private_key).unwrap();
        block.add_transaction(new_tx).unwrap();
        // block.transactions.push(Transaction::new_transfer(b"hello world"));
        assert!(block.sign(&private_key).is_ok());

        let private_key = PrivateKey::new();

        assert!(matches!(block.sign(&private_key), Err(_)));

        assert!(block.verify().is_ok());

        let (sender, receiver) = random_sender_receiver();

        block
            .transactions
            .push(Transaction::new_transfer(sender, receiver, r_hash, b"hello world").unwrap());

        let msg = "transaction has no signature".to_string();

        let res = match block.verify() {
            Ok(_) => "wrong".to_string(),
            Err(e) => e.to_string(),
        };

        assert_eq!(res, msg);
    }

    #[test]
    fn test_verify_block_with_tx() {
        let header = random_header(0, random_hash());
        let private_key = PrivateKey::new();

        let mut block = Block::new(header, vec![]).unwrap();

        assert!(block.sign(&private_key).is_ok());

        let private_key = PrivateKey::new();

        assert!(matches!(block.sign(&private_key), Err(_)));

        assert!(block.verify().is_ok());

        block.transactions.push(random_signed_tx());

        let msg = "invalid signature".to_string();

        let res = match block.verify() {
            Ok(_) => "wrong".to_string(),
            Err(e) => e.to_string(),
        };

        assert_eq!(res, msg);
    }

    #[test]
    fn test_block_byte_parsing() {
        let header = random_header(1, random_hash());
        let block = random_block(header);

        let block_bytes = block.to_bytes().unwrap();

        assert!(Block::from_bytes(&block_bytes).is_ok());

        let decoded_block = Block::from_bytes(&block_bytes).unwrap();
        assert_eq!(format!("{:?}", block), format!("{:?}", decoded_block));
    }
}

pub fn random_block(header: Header) -> Block {
    Block::new(header, vec![]).unwrap()
}

pub fn random_signed_block(header: Header) -> Block {
    let mut block = Block::new(header, vec![]).unwrap();
    let pvt_key = PrivateKey::new();

    block.sign(&pvt_key).unwrap();
    block
}
