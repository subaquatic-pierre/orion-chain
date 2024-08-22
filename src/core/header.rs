use borsh::{BorshDeserialize, BorshSerialize};
use log::debug;
use serde::{Deserialize, Serialize};
use std::time::SystemTime;

use super::{
    block::Block,
    encoding::{ByteEncoding, HexEncoding},
    error::CoreError,
    transaction::Transaction,
    util::timestamp,
};
use crate::crypto::{
    hash::{Hash, Hasher},
    utils::random_hash,
};

#[derive(Clone, Debug, BorshSerialize, BorshDeserialize, PartialEq)]
pub struct Header {
    pub version: u8,
    pub blockhash: Hash,
    pub prev_blockhash: Hash,
    pub height: usize,
    pub timestamp: u64,
    pub tx_root: Hash,
    pub state_root: Hash,
    pub poh: Hash,
}

impl Header {
    pub fn new(
        height: usize,
        blockhash: Hash,
        poh: Hash,
        tx_root: Hash,
        state_root: Hash,
        prev_blockhash: Hash,
    ) -> Self {
        let now = SystemTime::now();
        let timestamp = timestamp(now);
        Self {
            version: 1,
            blockhash,
            timestamp,
            // Below fields are used to determine blockhash
            height,
            prev_blockhash,
            poh,
            tx_root,
            state_root,
        }
    }

    pub fn height(&self) -> usize {
        self.height
    }

    pub fn prev_hash(&self) -> Hash {
        self.prev_blockhash.clone()
    }

    pub fn hash(&self) -> Hash {
        self.blockhash.clone()
    }

    pub fn hashable_data(&self) -> Vec<u8> {
        vec![]
    }

    // Static Hashing Methods
    pub fn gen_blockhash(
        block_height: usize,
        prev_blockhash: Hash,
        poh: Hash,
        tx_root: Hash,
        state_root: Hash,
    ) -> Result<Hash, CoreError> {
        let mut buf = vec![];

        buf.extend_from_slice(&block_height.to_le_bytes().to_vec());
        buf.extend_from_slice(&prev_blockhash.to_bytes()?);
        buf.extend_from_slice(&poh.to_bytes()?);
        buf.extend_from_slice(&tx_root.to_bytes()?);
        buf.extend_from_slice(&state_root.to_bytes()?);

        Ok(Hash::sha256(&buf)?)
    }

    pub fn gen_tx_root(txs: &[Transaction]) -> Result<Hash, CoreError> {
        let hash: Hash = match txs.len() {
            0 => Hash::sha256(&[])?,
            1 => {
                let mut buf: Vec<u8> = vec![];
                let tx1_bytes = &txs[0].hash()?.to_bytes()?;
                buf.extend_from_slice(&tx1_bytes);
                buf.extend_from_slice(&tx1_bytes);
                Hash::sha256(&buf).unwrap()
            }
            2 => {
                let mut buf: Vec<u8> = vec![];
                let tx1_bytes = &txs[0].hash()?.to_bytes()?;
                let tx2_bytes = &txs[1].hash()?.to_bytes()?;

                buf.extend_from_slice(&tx1_bytes);
                buf.extend_from_slice(&tx2_bytes);
                return Ok(Hash::sha256(&buf)?);
            }
            _ => return Self::gen_tx_root(&txs[..txs.len() - 2]),
        };

        Ok(hash)
    }

    pub fn gen_poh(txs: &[Transaction]) -> Result<Hash, CoreError> {
        let mut hasher = Hasher::new();

        for tx in txs {
            hasher.update(&tx.to_bytes()?)?;
        }

        Ok(hasher.finalize()?)
    }

    pub fn gen_state_root() -> Result<Hash, CoreError> {
        debug!("NEED TO IMPLEMENT Header::gen_state_root!!!");
        Ok(random_hash())
    }
}

impl ByteEncoding<Header> for Header {
    fn to_bytes(&self) -> Result<Vec<u8>, CoreError> {
        match borsh::to_vec(self) {
            Ok(b) => Ok(b),
            Err(e) => Err(CoreError::Parsing(e.to_string())),
        }
    }

    fn from_bytes(data: &[u8]) -> Result<Header, CoreError> {
        match borsh::from_slice(data) {
            Ok(t) => Ok(t),
            Err(e) => Err(CoreError::Parsing(e.to_string())),
        }
    }
}

impl ByteEncoding<Header> for &Header {
    fn to_bytes(&self) -> Result<Vec<u8>, CoreError> {
        match borsh::to_vec(self) {
            Ok(b) => Ok(b),
            Err(e) => Err(CoreError::Parsing(e.to_string())),
        }
    }

    fn from_bytes(data: &[u8]) -> Result<Header, CoreError> {
        match borsh::from_slice(data) {
            Ok(t) => Ok(t),
            Err(e) => Err(CoreError::Parsing(e.to_string())),
        }
    }
}

impl HexEncoding<Header> for Header {
    fn to_hex(&self) -> Result<String, CoreError> {
        Ok(hex::encode(&self.to_bytes()?))
    }

    fn from_hex(data: &str) -> Result<Header, CoreError> {
        let bytes = hex::decode(data)?;

        Self::from_bytes(&bytes)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::{
        core::transaction::random_signed_tx,
        crypto::{hash::Hash, public_key::PublicKey, signature::Signature, utils::random_hash},
    };

    #[test]
    fn test_header_parse_bytes() {
        let header = random_header(0, random_hash());

        let bytes = header.to_bytes().unwrap();

        // assert_eq!(bytes.len(), 81);

        let header_2 = Header::from_bytes(&bytes);

        assert!(header_2.is_ok());

        let header_2 = header_2.unwrap();

        assert_eq!(header.hash().to_string(), header_2.hash().to_string());
        assert_eq!(
            header.prev_hash().to_string(),
            header_2.prev_hash().to_string()
        );
        assert_eq!(header.version, header_2.version);
        assert_eq!(header.timestamp, header_2.timestamp);
        assert_eq!(header.height, header_2.height);
    }

    #[test]
    fn test_header_parse_hex() {
        let header = random_header(0, random_hash());

        let hex_str = header.to_hex().unwrap();

        // assert_eq!(hex_str.len(), 154);

        let header_2 = Header::from_hex(&hex_str);

        assert!(header_2.is_ok());

        let header_2 = header_2.unwrap();

        assert_eq!(header.hash().to_string(), header_2.hash().to_string());
        assert_eq!(
            header.prev_hash().to_string(),
            header_2.prev_hash().to_string()
        );
        assert_eq!(header.version, header_2.version);
        assert_eq!(header.timestamp, header_2.timestamp);
        assert_eq!(header.height, header_2.height);
    }

    #[test]
    fn test_gen_tx_root_empty() {
        let txs: Vec<Transaction> = vec![];
        let result = Header::gen_tx_root(&txs);
        assert!(result.is_ok());

        let expected_hash = Hash::sha256(&[]).unwrap();
        assert_eq!(result.unwrap(), expected_hash);
    }

    #[test]
    fn test_gen_tx_root_single() {
        let tx1 = random_signed_tx();
        let txs = vec![tx1.clone()];
        let result = Header::gen_tx_root(&txs);

        if let Err(e) = &result {
            println!("{e}");
        }

        assert!(result.is_ok());

        let mut buf = vec![];
        let tx1_bytes = tx1.hash().unwrap().to_bytes().unwrap();
        buf.extend_from_slice(&tx1_bytes);
        buf.extend_from_slice(&tx1_bytes);

        let expected_hash = Hash::sha256(&buf).unwrap();
        assert_eq!(result.unwrap(), expected_hash);
    }

    #[test]
    fn test_gen_tx_root_two() {
        let tx1 = random_signed_tx();
        let tx2 = random_signed_tx();
        let txs = vec![tx1.clone(), tx2.clone()];
        let result = Header::gen_tx_root(&txs);
        assert!(result.is_ok());

        let mut buf = vec![];
        let tx1_bytes = tx1.hash().unwrap().to_bytes().unwrap();
        let tx2_bytes = tx2.hash().unwrap().to_bytes().unwrap();

        buf.extend_from_slice(&tx1_bytes);
        buf.extend_from_slice(&tx2_bytes);

        let expected_hash = Hash::sha256(&buf).unwrap();
        assert_eq!(result.unwrap(), expected_hash);
    }

    #[test]
    fn test_gen_tx_root_multiple() {
        let tx1 = random_signed_tx();
        let tx2 = random_signed_tx();
        let tx3 = random_signed_tx();
        let txs = vec![tx1.clone(), tx2.clone(), tx3.clone()];
        let result = Header::gen_tx_root(&txs);
        assert!(result.is_ok());

        // Since gen_tx_root returns the root hash of tx1 and tx2, we can validate against that.
        let expected_root = Header::gen_tx_root(&txs[..txs.len() - 2]).unwrap();
        assert_eq!(result.unwrap(), expected_root);
    }

    #[test]
    fn test_gen_poh_empty() {
        let txs: Vec<Transaction> = vec![];
        let result = Header::gen_poh(&txs);
        assert!(result.is_ok());

        let expected_hash = Hasher::new().finalize().unwrap();
        assert_eq!(result.unwrap(), expected_hash);
    }

    #[test]
    fn test_gen_poh_single() {
        let tx1 = random_signed_tx();
        let txs = vec![tx1.clone()];
        let result = Header::gen_poh(&txs);
        assert!(result.is_ok());

        let mut hasher = Hasher::new();
        hasher.update(&tx1.to_bytes().unwrap()).unwrap();
        let expected_hash = hasher.finalize().unwrap();
        assert_eq!(result.unwrap(), expected_hash);
    }

    #[test]
    fn test_gen_poh_multiple() {
        let tx1 = random_signed_tx();
        let tx2 = random_signed_tx();
        let tx3 = random_signed_tx();
        let txs = vec![tx1.clone(), tx2.clone(), tx3.clone()];
        let result = Header::gen_poh(&txs);
        assert!(result.is_ok());

        let mut hasher = Hasher::new();
        for tx in &txs {
            hasher.update(&tx.to_bytes().unwrap()).unwrap();
        }
        let expected_hash = hasher.finalize().unwrap();
        assert_eq!(result.unwrap(), expected_hash);
    }
}

pub fn random_header(height: usize, prev_hash: Hash) -> Header {
    let hash = random_hash();
    let prev_hash = prev_hash;
    let timestamp = timestamp(SystemTime::now());
    let version = 1;
    let random_hash = random_hash();

    Header {
        version,
        blockhash: hash,
        prev_blockhash: prev_hash,
        height,
        timestamp,
        tx_root: random_hash,
        state_root: random_hash,
        poh: random_hash,
    }
}
