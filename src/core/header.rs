use std::time::SystemTime;

use serde::{Deserialize, Serialize};

use super::{
    block::Block,
    encoding::{ByteEncoding, HexEncoding},
    error::CoreError,
    util::timestamp,
};
use crate::crypto::{hash::Hash, utils::random_hash};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Header {
    pub version: u8,
    hash: Hash,
    prev_hash: Hash,
    pub height: usize,
    pub timestamp: u64,
}

impl Header {
    pub fn new(height: usize, hash: Hash, prev_hash: Hash) -> Self {
        let now = SystemTime::now();
        let timestamp = timestamp(now);
        Self {
            version: 1,
            hash,
            prev_hash,
            height,
            timestamp,
        }
    }

    pub fn height(&self) -> usize {
        self.height
    }

    pub fn prev_hash(&self) -> Hash {
        self.prev_hash.clone()
    }

    pub fn hash(&self) -> Hash {
        // TODO: Handle error checking
        Hash::sha256(&self.to_bytes().unwrap()).unwrap()
    }
}

#[derive(Clone, Debug)]
pub struct HeaderManager {
    headers: Vec<Header>,
}

impl HeaderManager {
    pub fn new() -> Self {
        Self { headers: vec![] }
    }

    pub fn add(&mut self, header: Header) {
        self.headers.push(header)
    }

    pub fn get(&self, index: usize) -> Option<&Header> {
        if let Some(h) = self.headers.get(index) {
            Some(h)
        } else {
            None
        }
    }

    pub fn last(&self) -> Option<&Header> {
        if let Some(h) = self.headers.last() {
            Some(h)
        } else {
            None
        }
    }

    pub fn has_block(&self, height: usize) -> bool {
        height <= self.height()
    }

    pub fn height(&self) -> usize {
        self.headers.len() - 1
    }
}

impl ByteEncoding<Header> for Header {
    fn to_bytes(&self) -> Result<Vec<u8>, CoreError> {
        Ok(bincode::serialize(&self)?)
    }

    fn from_bytes(data: &[u8]) -> Result<Header, CoreError> {
        Ok(bincode::deserialize(data)?)
    }
}

impl ByteEncoding<Header> for &Header {
    fn to_bytes(&self) -> Result<Vec<u8>, CoreError> {
        Ok(bincode::serialize(&self)?)
    }

    fn from_bytes(data: &[u8]) -> Result<Header, CoreError> {
        Ok(bincode::deserialize(data)?)
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

        assert_eq!(header.hash.to_string(), header_2.hash.to_string());
        assert_eq!(header.prev_hash.to_string(), header_2.prev_hash.to_string());
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

        assert_eq!(header.hash.to_string(), header_2.hash.to_string());
        assert_eq!(header.prev_hash.to_string(), header_2.prev_hash.to_string());
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

    Header {
        version,
        hash,
        prev_hash,
        height,
        timestamp,
    }
}
