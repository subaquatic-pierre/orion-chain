use std::io::BufWriter;

use crate::crypto::{hash::Hash, public_key::PublicKey, signature::Signature, utils::random_hash};

use super::{
    encoding::{BlockEncoder, ByteEncoding, Encoder},
    error::CoreError,
    header::Header,
    transaction::Transaction,
    utils::timestamp,
};

pub struct Block<'a> {
    header: &'a Header,
    transactions: Vec<Transaction>,
    signer: Option<PublicKey>,
    signature: Option<Signature>,

    // cached hash
    hash: Option<Hash>,
}

impl<'a> Block<'a> {
    pub fn new(header: &'a Header, txs: Vec<Transaction>) -> Self {
        Self {
            header,
            transactions: txs,
            signer: None,
            signature: None,
            hash: None,
        }
    }

    pub fn encode(&self, enc: impl Encoder<Block<'a>, CoreError>) -> Result<(), CoreError> {
        let mut buf = [0_u8; 1028];
        let buf_writer = BufWriter::new(buf.as_mut());
        enc.encode(buf_writer, self.header)
    }

    pub fn header_data(&self) -> Vec<u8> {
        self.header.to_bytes()
    }
}

#[cfg(test)]
mod test {
    use super::*;
}
