use std::collections::HashMap;

use crate::{core::transaction::Transaction, crypto::hash::Hash};

pub struct TxPool<'a> {
    transactions: HashMap<Hash, &'a Transaction>,
}

impl<'a> TxPool<'a> {
    pub fn new() -> Self {
        Self {
            transactions: HashMap::new(),
        }
    }

    pub fn add(&mut self, tx: &'a Transaction) {
        self.transactions.insert(tx.hash(), tx);
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
}
