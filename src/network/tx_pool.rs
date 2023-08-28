use std::collections::HashMap;

use crate::{core::transaction::Transaction, crypto::hash::Hash};

pub struct TxPool {
    transactions: HashMap<Hash, Transaction>,
}

impl TxPool {
    pub fn new() -> Self {
        Self {
            transactions: HashMap::new(),
        }
    }

    pub fn take(&mut self, len: usize) -> Vec<Transaction> {
        let mut keys: Vec<Hash> = vec![];
        for (idx, key) in self.transactions.keys().enumerate() {
            if idx < len {
                keys.push(key.clone());
            }
        }

        let mut txs = vec![];
        for key in &keys {
            // SAFETY: known to have key as taken from iterator above
            let tx = self.transactions.get(key).unwrap();
            txs.push(tx.clone())
        }

        for key in keys {
            self.transactions.remove(&key);
        }

        txs
    }

    pub fn add(&mut self, tx: &Transaction) {
        self.transactions.insert(tx.hash(), tx.clone());
    }

    pub fn has(&self, tx: &Transaction) -> bool {
        self.transactions.contains_key(&tx.hash())
    }

    pub fn len(&self) -> usize {
        self.transactions.len()
    }

    pub fn flush(&mut self) {
        self.transactions.clear()
    }
}

#[cfg(test)]
mod test {
    use crate::core::transaction::random_tx;

    use super::*;
    #[test]
    fn test_add_tx() {
        let mut tx_pool = TxPool::new();

        let tx = random_tx();
        tx_pool.add(&tx);

        assert_eq!(tx_pool.len(), 1)
    }

    #[test]
    fn test_flush() {
        let mut tx_pool = TxPool::new();

        let txs: Vec<Transaction> = (0..20).map(|i| Transaction::new(&[i])).collect();

        for tx in &txs {
            tx_pool.add(tx);
        }

        assert_eq!(tx_pool.len(), 20);

        tx_pool.flush();

        assert_eq!(tx_pool.len(), 0)
    }

    #[test]
    fn test_take_txs() {
        let mut tx_pool = TxPool::new();

        let txs: Vec<Transaction> = (0..20).map(|i| Transaction::new(&[i])).collect();

        for tx in &txs {
            tx_pool.add(tx);
        }

        let txs = tx_pool.take(3);

        assert_eq!(txs.len(), 3);
        assert_eq!(tx_pool.len(), 17);
    }
}
