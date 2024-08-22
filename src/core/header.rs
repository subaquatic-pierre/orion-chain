use borsh::{BorshDeserialize, BorshSerialize};
use serde::{Deserialize, Serialize};
use std::time::SystemTime;

use super::{
    block::Block,
    encoding::{ByteEncoding, HexEncoding},
    error::CoreError,
    util::timestamp,
};
use crate::crypto::{hash::Hash, utils::random_hash};

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
        hash: Hash,
        poh: Hash,
        tx_root: Hash,
        state_root: Hash,
        prev_hash: Hash,
    ) -> Self {
        let now = SystemTime::now();
        let timestamp = timestamp(now);
        Self {
            version: 1,
            blockhash: hash,
            prev_blockhash: prev_hash,
            height,
            timestamp,
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
    use crate::crypto::{
        hash::Hash, public_key::PublicKey, signature::Signature, utils::random_hash,
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
