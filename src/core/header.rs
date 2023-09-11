use std::time::SystemTime;

use super::{
    block::Block,
    encoding::{ByteDecoding, ByteEncoding, HexDecoding, HexEncoding},
    error::CoreError,
    hasher::Hasher,
    util::timestamp,
};
use crate::crypto::{hash::Hash, utils::random_hash};

#[derive(Clone, Debug)]
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

// TODO: Not using Hasher trait
impl Hasher<Header> for Header {
    fn hash(&self) -> Hash {
        Hash::sha256(&self.to_bytes()).unwrap()
    }
}

impl ByteDecoding for Header {
    type Target = Self;
    type Error = CoreError;

    fn from_bytes(data: &[u8]) -> Result<Header, CoreError> {
        let mut offset = 0;
        let version = u8::from_be_bytes(data[offset..1].try_into().unwrap());
        offset += 1;

        let hash = match Hash::new(&data[offset..offset + 32]) {
            Ok(hash) => hash,
            Err(e) => return Err(CoreError::Parsing(format!("unable to parse hash {e}"))),
        };

        offset += 32;

        let prev_hash = match Hash::new(&data[offset..offset + 32]) {
            Ok(hash) => hash,
            Err(e) => return Err(CoreError::Parsing(format!("unable to parse hash {e}"))),
        };

        offset += 32;

        let height = usize::from_be_bytes(data[offset..offset + 8].try_into().unwrap());
        offset += 8;

        let timestamp = u64::from_be_bytes(data[offset..offset + 8].try_into().unwrap());

        Ok(Self {
            version,
            hash,
            prev_hash,
            height,
            timestamp,
        })
    }
}

impl ByteEncoding for Header {
    fn to_bytes(&self) -> Vec<u8> {
        let mut buf: Vec<u8> = vec![];

        // append version
        buf.extend_from_slice(&self.version.to_be_bytes());

        // append data hash
        buf.extend_from_slice(&self.hash.to_bytes());

        // append prev hash
        buf.extend_from_slice(&self.prev_hash.to_bytes());

        // append height
        buf.extend_from_slice(&self.height.to_be_bytes());

        // append timestamp
        buf.extend_from_slice(&self.timestamp.to_be_bytes());

        buf
    }
}

impl ByteEncoding for &Header {
    fn to_bytes(&self) -> Vec<u8> {
        let mut buf: Vec<u8> = vec![];

        // append version
        buf.extend_from_slice(&self.version.to_be_bytes());

        // append data hash
        buf.extend_from_slice(&self.hash.to_bytes());

        // append prev hash
        buf.extend_from_slice(&self.prev_hash.to_bytes());

        // append height
        buf.extend_from_slice(&self.height.to_be_bytes());

        // append timestamp
        buf.extend_from_slice(&self.timestamp.to_be_bytes());

        buf
    }
}

impl HexEncoding for Header {
    fn to_hex(&self) -> String {
        let bytes = &self.to_bytes();
        hex::encode(bytes)
    }
}

impl HexDecoding for Header {
    type Target = Self;
    type Error = CoreError;

    fn from_hex(data: &str) -> Result<Header, CoreError> {
        let bytes = hex::decode(data);
        match bytes {
            Ok(bytes) => Self::from_bytes(&bytes),
            Err(e) => Err(CoreError::Parsing(format!(
                "unable to parse hex from bytes {e}"
            ))),
        }
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

        let bytes = header.to_bytes();

        assert_eq!(bytes.len(), 81);

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

        let hex_str = header.to_hex();

        assert_eq!(hex_str.len(), 154);

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
