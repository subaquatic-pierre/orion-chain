use std::io::{BufWriter, Read, Write};

use ecdsa::signature::rand_core::block;
use k256::pkcs8::der::Reader;

use crate::crypto::{
    hash::Hash, private_key::PrivateKey, public_key::PublicKey, signature::Signature,
    utils::random_hash,
};

use super::{
    encoding::{ByteDecoding, ByteEncoding, HexDecoding, HexEncoding},
    error::CoreError,
    hasher::Hasher,
    header::{random_header, Header},
    transaction::Transaction,
    utils::timestamp,
};

#[derive(Debug, Clone)]
pub struct Block {
    header: Header,
    signer: Option<PublicKey>,
    signature: Option<Signature>,
    transactions: Vec<Transaction>,

    // cached hash
    pub hash: Hash,
}

impl Block {
    pub fn new(header: Header, txs: Vec<Transaction>) -> Self {
        let hash = Self::generate_block_hash(&txs);
        Self {
            header,
            transactions: txs,
            signer: None,
            signature: None,
            hash,
        }
    }

    pub fn header(&self) -> &Header {
        &self.header
    }

    pub fn num_txs(&self) -> usize {
        self.transactions.len()
    }

    pub fn sign(&mut self, private_key: PrivateKey) -> Result<(), CoreError> {
        if self.signer.is_some() | self.signature.is_some() {
            return Err(CoreError::Block("block already has signature".to_string()));
        }

        // if self.hash.is_none() {
        //     self.hash();
        // }

        let signature = private_key.sign(&self.hashable_data());
        let signer = private_key.pub_key();

        self.signature = Some(signature);
        self.signer = Some(signer);

        Ok(())
    }

    pub fn add_transaction(&mut self, tx: Transaction) -> Result<(), CoreError> {
        match tx.verify() {
            Ok(_) => {
                self.transactions.push(tx);
                Ok(())
            }
            Err(e) => Err(e),
        }
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
                match pub_key.verify(&self.hashable_data(), self.signature.clone().unwrap()) {
                    true => Ok(()),
                    false => Err(CoreError::Block("invalid signature".to_string())),
                }
            }
            None => Err(CoreError::Block("no signer exists for block".to_string())),
        }
    }

    pub fn hash(&mut self) -> &Hash {
        // let hashable_bytes = &self.hashable_data();
        &self.hash
    }

    pub fn header_data(&self) -> Vec<u8> {
        self.header.to_bytes()
    }

    pub fn height(&self) -> usize {
        self.header.height() as usize
    }

    // TODO: ENCODE AND DECODE
    pub fn encode(&self, mut writer: impl Write) -> Result<(), CoreError> {
        match writer.write_all(&self.header.to_bytes()) {
            Ok(_) => Ok(()),
            Err(e) => Err(CoreError::Block(format!("unable to encode block {e}"))),
        }
    }

    pub fn decode(bytes: &[u8]) -> Result<Self, CoreError> {
        todo!()
    }

    // ---
    // Private Methods
    // ---

    // get sequential bytes of all transactions
    fn txs_bytes(&self) -> Vec<u8> {
        let mut txs_bytes = vec![];
        for tx in self.transactions.iter() {
            let bytes = tx.to_bytes();
            txs_bytes.extend_from_slice(&bytes);
        }
        txs_bytes
    }

    // get data to be hashed for the block
    fn hashable_data(&self) -> Vec<u8> {
        let mut data = vec![];
        data.extend_from_slice(&self.header_data());
        data.extend_from_slice(&self.txs_bytes());
        data
    }

    // ---
    // Static methods
    // ---
    // TODO: implement merkle root
    pub fn generate_block_hash(txs: &Vec<Transaction>) -> Hash {
        let mut hash: Hash = match txs.len() {
            0 => Hash::sha256(&[]).unwrap(),
            2 => {
                let mut buf: Vec<u8> = vec![];
                buf.extend_from_slice(&txs[0].hash.to_bytes());
                buf.extend_from_slice(&txs[1].hash.to_bytes());
                return Hash::sha256(&buf).unwrap();
            }
            _ => return Hash::sha256(&txs[0].hash.to_bytes()).unwrap(),
        };

        for (i, tx) in txs.iter().skip(2).enumerate() {
            let prev_tx = &txs[i - 1];
            let mut buf: Vec<u8> = vec![];
            buf.extend_from_slice(&hash.to_bytes());
            buf.extend_from_slice(&prev_tx.hash.to_bytes());
            buf.extend_from_slice(&tx.hash.to_bytes());
            hash = Hash::sha256(&buf).unwrap();
        }

        Hash::new(&hash.to_bytes()).unwrap()
    }
}

impl ByteEncoding for Block {
    fn to_bytes(&self) -> Vec<u8> {
        let mut buf: Vec<u8> = vec![];

        let header_bytes = self.header.to_bytes();
        buf.extend_from_slice(&header_bytes);

        if let Some(signature) = &self.signature {
            buf.extend_from_slice(&[1_u8]);
            buf.extend_from_slice(&signature.to_bytes());
        } else {
            buf.extend_from_slice(&[0_u8]);
        };

        if let Some(signer) = &self.signer {
            buf.extend_from_slice(&[1_u8]);
            buf.extend_from_slice(&signer.to_bytes());
        } else {
            buf.extend_from_slice(&[0_u8]);
        };

        for tx in &self.transactions {
            buf.extend_from_slice(&tx.to_bytes());
        }

        buf
    }
}

impl ByteDecoding for Block {
    type Target = Block;
    type Error = CoreError;

    fn from_bytes(data: &[u8]) -> Result<Self::Target, Self::Error> {
        let mut offset = 0;
        let data_len = data.len();
        // let header = random_header(1, random_hash());

        let header = Header::from_bytes(data)?;
        offset += header.to_bytes().len();

        let has_sig = u8::from_be_bytes(data[offset..offset + 1].try_into().unwrap());
        offset += 1;

        let mut signature = None;
        if has_sig > 0 {
            signature = Some(Signature::from_bytes(&data[offset..offset + 64])?);
            offset += 64;
        }

        let has_signer = u8::from_be_bytes(data[offset..offset + 1].try_into().unwrap());
        offset += 1;

        let mut signer = None;
        if has_signer > 0 {
            signer = Some(PublicKey::from_bytes(&data[offset..offset + 33])?);
            offset += 33;
        }

        // get transaction from rest of bytes
        let mut txs: Vec<Transaction> = vec![];
        while offset < data_len {
            let tx = Transaction::from_bytes(&data[offset..])?;
            offset += tx.to_bytes().len();
            txs.push(tx);
        }

        let hash = Self::generate_block_hash(&txs);
        Ok(Self {
            header,
            transactions: txs,
            signer,
            signature,
            hash,
        })
    }
}

impl HexEncoding for Block {
    fn to_hex(&self) -> String {
        let bytes = &self.to_bytes();
        hex::encode(bytes)
    }
}

impl HexDecoding for Block {
    type Target = Self;
    type Error = CoreError;

    fn from_hex(data: &str) -> Result<Block, CoreError> {
        let bytes = hex::decode(data);
        match bytes {
            Ok(bytes) => Self::from_bytes(&bytes),
            Err(e) => Err(CoreError::Parsing(format!(
                "unable to parse hex from bytes {e}"
            ))),
        }
    }
}

// TODO: Not using Hasher trait
impl Hasher<Block> for Block {
    fn hash(&self) -> Hash {
        Hash::sha256(&self.hashable_data()).unwrap()
    }
}

#[cfg(test)]
mod test {
    use crate::core::{
        header::random_header,
        transaction::{random_signed_tx, random_tx},
    };

    use super::*;

    #[test]
    fn test_sign_block() {
        let header = random_header(0, random_hash());
        let private_key = PrivateKey::new();

        let mut block = Block::new(header, vec![]);

        assert!(block.sign(private_key).is_ok());

        assert!(block.signature.is_some());
        assert!(block.signer.is_some());
    }

    #[test]
    fn add_transaction() {
        let header = random_header(0, random_hash());
        // let private_key = PrivateKey::new();

        let mut block = Block::new(header, vec![]);

        // assert error adding unsigned transactions
        let tx = random_tx();
        let msg = "transaction has no signature".to_string();
        let res = match block.add_transaction(tx) {
            Ok(_) => "wrong".to_string(),
            Err(e) => e.to_string(),
        };
        assert_eq!(res, msg);

        // assert no error adding signed transaction
        let tx = random_signed_tx();
        assert!(block.add_transaction(tx).is_ok());
    }

    #[test]
    fn test_verify_block() {
        let header = random_header(0, random_hash());
        let private_key = PrivateKey::new();

        let mut block = Block::new(header, vec![]);

        assert!(block.sign(private_key).is_ok());

        let private_key = PrivateKey::new();

        assert!(matches!(block.sign(private_key), Err(_)));

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

        let mut block = Block::new(header, vec![]);

        assert!(block.sign(private_key).is_ok());

        let private_key = PrivateKey::new();

        assert!(matches!(block.sign(private_key), Err(_)));

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

        let block_bytes = block.to_bytes();

        assert!(Block::from_bytes(&block_bytes).is_ok());

        let decoded_block = Block::from_bytes(&block_bytes).unwrap();
        assert_eq!(format!("{:?}", block), format!("{:?}", decoded_block));
    }
}

pub fn random_block(header: Header) -> Block {
    Block::new(header, vec![])
}

pub fn random_signed_block(header: Header) -> Block {
    let mut block = Block::new(header, vec![]);
    let pvt_key = PrivateKey::new();

    block.sign(pvt_key).unwrap();
    block
}
