use std::collections::VecDeque;

use crate::core::transaction::Transaction;

pub struct TxPool {
    transactions: VecDeque<Transaction>,
}

impl TxPool {
    pub fn new() -> Self {
        Self {
            transactions: VecDeque::new(),
        }
    }

    pub fn take(&mut self, len: usize) -> Vec<Transaction> {
        let mut txs = vec![];
        let self_len = self.transactions.len();
        for i in 0..len {
            if i < self_len {
                // SAFETY: checked length of transactions above
                // guaranteed to have at least one element
                txs.push(self.transactions.pop_front().unwrap());
            }
        }
        txs
    }

    pub fn add(&mut self, tx: Transaction) {
        self.transactions.push_back(tx);
    }

    pub fn has(&self, tx: &Transaction) -> bool {
        self.transactions.contains(tx)
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
        tx_pool.add(tx);

        assert_eq!(tx_pool.len(), 1)
    }

    #[test]
    fn test_flush() {
        let mut tx_pool = TxPool::new();

        let txs: Vec<Transaction> = (0..20).map(|i| Transaction::new(&[i]).unwrap()).collect();

        for tx in txs {
            tx_pool.add(tx);
        }

        assert_eq!(tx_pool.len(), 20);

        tx_pool.flush();

        assert_eq!(tx_pool.len(), 0)
    }

    #[test]
    fn test_take_txs() {
        let mut tx_pool = TxPool::new();

        let txs: Vec<Transaction> = (0..20).map(|i| Transaction::new(&[i]).unwrap()).collect();

        for tx in txs {
            tx_pool.add(tx);
        }

        let txs = tx_pool.take(3);

        assert_eq!(txs.len(), 3);

        let tx = Transaction::new(&[1]).unwrap();
        assert_eq!(txs.contains(&tx), true);
        let tx = Transaction::new(&[4]).unwrap();
        assert_eq!(txs.contains(&tx), false);

        let tx = Transaction::new(&[1]).unwrap();
        assert_eq!(tx_pool.len(), 17);
        assert_eq!(tx_pool.has(&tx), false);
    }
}
