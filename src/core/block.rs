use crate::crypto::{hash::Hash, public_key::PublicKey, signature::Signature};

use super::transaction::Transaction;

pub struct Header {
    version: u32,
    data_hash: Hash,
    prev_hash: Hash,
    height: u32,
    timestamp: i64,
}

pub struct Block<'a> {
    header: &'a Header,
    transactions: Vec<Transaction>,
    validator: Option<PublicKey>,
    signature: Option<Signature>,
    hash: Option<Hash>,
}

impl<'a> Block<'a> {
    pub fn new(header: &'a Header, txs: Vec<Transaction>) -> Self {
        Self {
            header,
            transactions: txs,
            validator: None,
            signature: None,
            hash: None,
        }
    }

    pub fn header_data(&self) -> Vec<u8> {
        vec![]
    }
}
