use std::io::Write;

use log::info;
use serde::{Deserialize, Serialize};

use crate::crypto::{
    hash::Hash, private_key::PrivateKey, public_key::PublicKey, signature::Signature,
};

use crate::api::types::BlockJson;

use super::{
    encoding::{ByteEncoding, HexEncoding, JsonEncoding},
    error::CoreError,
    header::Header,
    storage::{MemoryStorage, Storage},
    transaction::Transaction,
};
#[derive(Clone, Debug)]
struct HeaderPointer(*const Header);

unsafe impl Send for HeaderPointer {}
unsafe impl Sync for HeaderPointer {}
#[derive(Clone, Debug)]
struct BlockPointer(*const Block);

unsafe impl Send for BlockPointer {}
unsafe impl Sync for BlockPointer {}

#[derive(Clone, Debug)]
pub struct BlockManager {
    blocks: Vec<Block>,
    store: MemoryStorage,
}

impl BlockManager {
    pub fn new() -> Self {
        Self {
            blocks: vec![],
            store: MemoryStorage::new(),
        }
    }

    pub fn headers(&self) -> Vec<&Header> {
        let mut headers = vec![];
        for block in &self.blocks {
            headers.push(&block.header);
        }
        headers
    }

    pub fn blocks(&self) -> Vec<&Block> {
        let mut blocks = vec![];
        for block in &self.blocks {
            blocks.push(block);
        }
        blocks
    }

    pub fn add(&mut self, block: Block) -> Result<(), CoreError> {
        info!(
            "adding block to chain with height: {}, and hash: {}",
            &block.height(),
            &block.hash().to_string()
        );
        match self.store.put(&block) {
            Ok(_) => {
                self.blocks.push(block);
                Ok(())
            }
            Err(e) => Err(e),
        }
    }

    pub fn get_block(&self, index: usize) -> Option<&Block> {
        self.blocks.get(index)
    }

    pub fn get_block_by_hash(&self, hash: &str) -> Option<&Block> {
        // TODO: implement get block by hash
        self.blocks.get(0)
    }

    pub fn get_header(&self, index: usize) -> Option<&Header> {
        if let Some(b) = self.blocks.get(index) {
            Some(&b.header)
        } else {
            None
        }
    }

    pub fn last(&self) -> Option<&Block> {
        self.blocks.last()
    }

    pub fn has_block(&self, height: usize) -> bool {
        height <= self.height()
    }

    pub fn height(&self) -> usize {
        self.blocks.len() - 1
    }

    // TODO: implement pointers to be used to get
    // block by hash and get header by hash
    // create new HashMaps on manager struct
    // implement and to and remove from hashmap when adding
    // or removing blocks

    // pub fn pointers() {
    // for ptr in &self.headers {
    //     unsafe {
    //         let header = &*(ptr.0 as *const Header);
    //         headers.push(header);
    //     };
    // }
    // headers
    // }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Block {
    pub header: Header,
    signer: Option<PublicKey>,
    signature: Option<Signature>,
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

        // TODO: return result if any transaction is invalid
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

        let signature = private_key.sign(&self.hashable_data()?);
        let signer = private_key.pub_key();

        self.signature = Some(signature);
        self.signer = Some(signer);

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

        match &self.signer {
            Some(pub_key) => {
                match pub_key.verify(&self.hashable_data()?, self.signature.clone().unwrap()) {
                    true => Ok(()),
                    false => Err(CoreError::Block("invalid signature".to_string())),
                }
            }
            None => Err(CoreError::Block("no signer exists for block".to_string())),
        }
    }

    pub fn prev_hash(&self) -> &Hash {
        &self.header.prev_hash
    }

    pub fn hash(&self) -> &Hash {
        &self.header.hash
    }

    pub fn header_data(&self) -> Result<Vec<u8>, CoreError> {
        self.header.to_bytes()
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
        data.extend_from_slice(&self.header_data()?);
        data.extend_from_slice(&self.txs_bytes()?);
        Ok(data)
    }

    // ---
    // Static methods
    // ---
    // TODO: implement merkle root
    pub fn generate_block_hash(txs: &[Transaction]) -> Result<Hash, CoreError> {
        let hash: Hash = match txs.len() {
            0 => Hash::sha256(&[]).unwrap(),
            1 => Hash::sha256(&[]).unwrap(),
            2 => {
                let mut buf: Vec<u8> = vec![];
                let tx1_bytes = &txs[0].hash().to_bytes()?;
                let tx2_bytes = &txs[1].hash().to_bytes()?;

                buf.extend_from_slice(&tx1_bytes);
                buf.extend_from_slice(&tx2_bytes);
                return Ok(Hash::sha256(&buf)?);
            }
            _ => return Block::generate_block_hash(&txs[..txs.len() - 2]),
        };

        Ok(hash)
    }
}

impl ByteEncoding<Block> for Block {
    fn from_bytes(data: &[u8]) -> Result<Block, CoreError> {
        Ok(bincode::deserialize(data)?)
    }

    fn to_bytes(&self) -> Result<Vec<u8>, CoreError> {
        Ok(bincode::serialize(&self)?)
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

impl JsonEncoding<BlockJson> for Block {
    fn from_json(data: serde_json::Value) -> Result<BlockJson, CoreError> {
        todo!()
    }
    fn to_json(&self) -> Result<serde_json::Value, CoreError> {
        todo!()
    }
}

#[cfg(test)]
mod test {
    use std::io::{BufWriter, Read, Write};

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
        let header = random_header(0, random_hash());
        let private_key = PrivateKey::new();

        let mut block = Block::new(header, vec![]).unwrap();

        let mut new_tx = Transaction::new(b"Cool World");
        new_tx.sign(&private_key).unwrap();
        block.add_transaction(new_tx).unwrap();
        // block.transactions.push(Transaction::new(b"hello world"));
        assert!(block.sign(&private_key).is_ok());

        let private_key = PrivateKey::new();

        assert!(matches!(block.sign(&private_key), Err(_)));

        assert!(block.verify().is_ok());

        block.transactions.push(Transaction::new(b"hello world"));

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

    #[test]
    fn test_header_manager() {
        let mut manager = BlockManager::new();

        for _ in 0..5 {
            let header = random_header(1, random_hash());
            let block = random_block(header);

            manager.add(block).unwrap();
        }

        let headers = manager.headers();
        let blocks = manager.blocks();

        assert_eq!(headers.len(), blocks.len());
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
