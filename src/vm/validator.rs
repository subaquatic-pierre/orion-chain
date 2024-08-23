use core::time;
use std::sync::{Arc, Mutex, MutexGuard};
use std::time::Instant;

use log::{error, info, warn};

use crate::core::blockchain::Blockchain;
use crate::core::encoding::ByteEncoding;
use crate::core::error::CoreError;

use crate::core::header::random_header;
use crate::core::transaction::{BlockRewardData, TxType};
use crate::crypto::hash::Hash;
use crate::lock;
use crate::network::types::ArcMut;
use crate::{
    core::{block::Block, header::Header, transaction::Transaction},
    crypto::private_key::PrivateKey,
    GenericError,
};

use super::runtime::ValidatorRuntime;

pub struct BlockValidator {
    private_key: PrivateKey,
    runtime: ValidatorRuntime,
    pub pool_size: usize,
}

impl BlockValidator {
    pub fn new(private_key: PrivateKey, pool_size: usize) -> Self {
        Self {
            private_key,
            pool_size,
            runtime: ValidatorRuntime::new(),
        }
    }

    pub fn validate_block(
        &self,
        chain: &MutexGuard<Blockchain>,
        block: &Block,
    ) -> Result<(), CoreError> {
        // Check if the block is already in the blockchain
        if chain.has_block(block.height()) {
            return Err(CoreError::Block(
                "Blockchain already contains block".to_string(),
            ));
        }

        // Check if the block height is correct
        if block.height() != chain.height() + 1 {
            return Err(CoreError::Block("Block height is incorrect".to_string()));
        }

        // Get the last block in the chain
        let last_block = chain.last_block().ok_or_else(|| {
            CoreError::Block("Unable to retrieve last block from the chain".to_string())
        })?;

        // Verify the previous block hash
        if block.header().prev_hash() != last_block.header().hash() {
            return Err(CoreError::Block(
                "Previous block hash is incorrect".to_string(),
            ));
        }

        // Verify the proof of history (PoH) if applicable
        if block.header().poh != Header::gen_poh(block.txs())? {
            return Err(CoreError::Block(
                "Proof of history (PoH) is invalid".to_string(),
            ));
        }

        // Verify the transaction root
        if block.header().tx_root != Header::gen_tx_root(block.txs())? {
            return Err(CoreError::Block("Transaction root is invalid".to_string()));
        }

        // Execute and validate all transactions in the block
        let state = chain.state();
        for tx in block.txs() {
            self.runtime.execute(tx, state)?;
        }

        // Verify the state root after applying all transactions
        let state_root = state.gen_state_root()?;
        if block.header().state_root != state_root {
            return Err(CoreError::Block("State root is invalid".to_string()));
        }

        // Revert the state after validation
        state.rollback()?;

        block.verify()
    }

    pub fn propose_block(
        &self,
        chain: &MutexGuard<Blockchain>,
        mut txs: Vec<Transaction>,
    ) -> Result<Block, CoreError> {
        let last_block = chain.last_block().ok_or(CoreError::Block(
            "unable to get last block from chain".to_string(),
        ))?;
        let last_header = last_block.header();
        let prev_blockhash = last_header.hash();

        self.insert_reward_txs(prev_blockhash, &mut txs)?;

        let height = last_header.height() + 1;
        let poh = Header::gen_poh(&txs)?;
        let tx_root = Header::gen_tx_root(&txs)?;

        // get state
        let state = chain.state();
        // execute each tx and backup each account
        for tx in &txs {
            // TODO: handle tx error case
            self.runtime.execute(tx, state)?
        }
        // calc new state_root after txs are applied
        let state_root = state.gen_state_root()?;
        // revert state after calculating state_root
        state.rollback()?;

        let blockhash = Header::gen_blockhash(height, prev_blockhash, poh, tx_root, state_root)?;

        let header = Header::new(height, blockhash, poh, tx_root, state_root, prev_blockhash);

        let mut block = Block::new(header, txs)?;

        info!(
            "create new block in BlockValidator {:}, num txs: {}, with height: {}",
            block.header().hash(),
            block.num_txs(),
            block.height()
        );

        if let Err(e) = block.sign(&self.private_key) {
            warn!("unable to sign block in miner: {e}")
        }

        Ok(block)
    }

    fn insert_reward_txs(
        &self,
        prev_blockhash: Hash,
        txs: &mut Vec<Transaction>,
    ) -> Result<(), CoreError> {
        // Calculate the block reward and gas fees
        let block_reward = self.calculate_block_reward();
        let gas_fees = self.collect_gas_fees(&txs);

        // Create reward and fee transactions
        let reward_tx =
            self.create_reward_transaction(TxType::BlockReward, prev_blockhash, block_reward)?;
        let fee_tx = self.create_reward_transaction(TxType::GasReward, prev_blockhash, gas_fees)?;

        // Prepend the reward and fee transactions to the tx list
        txs.insert(0, reward_tx);
        txs.insert(1, fee_tx);

        Ok(())
    }

    fn calculate_block_reward(&self) -> u64 {
        // Define how to calculate the block reward
        50 // Example reward value
    }

    fn collect_gas_fees(&self, txs: &[Transaction]) -> u64 {
        let mut total_fees = 0;
        for tx in txs {
            total_fees += tx.gas_limit; // Assuming Transaction struct has a `gas_fee` field
        }
        total_fees
    }

    fn create_reward_transaction(
        &self,
        tx_type: TxType,
        prev_blockhash: Hash,
        amount: u64,
    ) -> Result<Transaction, CoreError> {
        let data = BlockRewardData {
            to: self.private_key.address(),
            amount,
        }
        .to_bytes()?;
        // Create a transaction for the block reward
        let mut tx = Transaction::new(
            tx_type,
            prev_blockhash,
            self.private_key.address(),
            self.private_key.address(),
            &data,
            0,
        )?;
        tx.sign(&self.private_key)?;
        Ok(tx)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::block::Block;
    use crate::core::blockchain::Blockchain;
    use crate::core::header::Header;
    use crate::core::transaction::{random_signed_tx, Transaction, TransferData};
    use crate::crypto::address::Address;
    use crate::crypto::hash::Hash;
    use crate::crypto::private_key::{self, PrivateKey};
    use crate::crypto::utils::random_hash;
    use crate::state::account::Account;
    use std::sync::{Arc, Mutex};
    use std::time::Duration;

    fn setup_blockchain() -> Arc<Mutex<Blockchain>> {
        let chain = Blockchain::new_with_genesis_in_memory().unwrap();
        Arc::new(Mutex::new(chain))
    }

    fn build_tx(pvt_key: &PrivateKey) -> Transaction {
        let receiver = PrivateKey::new().address();
        let sender = pvt_key.address();
        let r_hash = random_hash();
        let bytes = TransferData {
            to: receiver.clone(),
            from: sender.clone(),
            amount: 42,
        }
        .to_bytes()
        .unwrap();
        let mut tx = Transaction::new_transfer(sender, receiver, r_hash, &bytes, 3).unwrap();
        tx.sign(&pvt_key).unwrap();
        tx
    }

    #[test]
    fn test_validate_block_success() {
        let blockchain = setup_blockchain();
        let private_key = PrivateKey::new();
        let validator = BlockValidator::new(private_key.clone(), 10);

        let chain = blockchain.lock().unwrap();

        let state = chain.state();
        state
            .set_account(&private_key.address(), &Account { balance: 100 })
            .unwrap();
        state.commit().unwrap();

        let txs = vec![build_tx(&private_key)];
        let block = validator.propose_block(&chain, txs).unwrap();

        let result = validator.validate_block(&chain, &block);

        if let Err(e) = &result {
            println!("{e}")
        }
        assert!(result.is_ok(), "Block should be valid");
    }

    #[test]
    fn test_validate_block_failure_duplicate() {
        let blockchain = setup_blockchain();
        let private_key = PrivateKey::new();
        let validator = BlockValidator::new(private_key.clone(), 10);

        let mut chain = blockchain.lock().unwrap();

        let state = chain.state();
        state
            .set_account(&private_key.address(), &Account { balance: 100 })
            .unwrap();
        state.commit().unwrap();

        let txs = vec![build_tx(&private_key)];

        let block = validator.propose_block(&chain, txs).unwrap();

        let _ = chain.add_block(block.clone());

        let result = validator.validate_block(&chain, &block);
        assert!(result.is_err(), "Block should be rejected as duplicate");
    }

    #[test]
    fn test_propose_block_success() {
        let blockchain = setup_blockchain();
        let private_key = PrivateKey::new();
        let validator = BlockValidator::new(private_key.clone(), 10);

        let chain = blockchain.lock().unwrap();

        let state = chain.state();
        state
            .set_account(&private_key.address(), &Account { balance: 100 })
            .unwrap();
        state.commit().unwrap();

        let txs = vec![build_tx(&private_key)];
        let result = validator.propose_block(&chain, txs);
        assert!(result.is_ok(), "Block should be proposed successfully");

        let block = result.unwrap();
        assert_eq!(block.height(), 1, "Block height should be 1");
    }

    #[test]
    fn test_propose_block_with_signature() {
        let blockchain = setup_blockchain();
        let private_key = PrivateKey::new();
        let validator = BlockValidator::new(private_key.clone(), 10);

        let chain = blockchain.lock().unwrap();

        let state = chain.state();
        state
            .set_account(&private_key.address(), &Account { balance: 100 })
            .unwrap();
        state.commit().unwrap();

        let txs = vec![build_tx(&private_key)];
        let result = validator.propose_block(&chain, txs);
        assert!(result.is_ok(), "Block should be proposed successfully");

        let block = result.unwrap();
        assert!(block.verify().is_ok(), "Block signature should be valid");
    }

    // TODO: implement validate blocks tests
    // #[test]
    // fn test_validate_block_valid_block() {
    //     let blockchain = Arc::new(Mutex::new(create_mock_blockchain()));
    //     let validator = create_mock_block_validator();
    //     let chain = blockchain.lock().unwrap();

    //     // Create a valid block to add to the blockchain
    //     let last_block = chain.last_block().unwrap();
    //     let valid_block = create_mock_block(1, last_block.header().hash().clone());

    //     // Validate the block
    //     let result = validator.validate_block(&chain, &valid_block);

    //     assert!(result.is_ok(), "Expected block to be valid");
    // }

    // #[test]
    // fn test_validate_block_duplicate_block() {
    //     let blockchain = Arc::new(Mutex::new(create_mock_blockchain()));
    //     let validator = create_mock_block_validator();
    //     let chain = blockchain.lock().unwrap();

    //     // Create a valid block and add it to the blockchain
    //     let last_block = chain.last_block().unwrap();
    //     let valid_block = create_mock_block(1, last_block.header().hash().clone());
    //     chain.add_block(valid_block.clone()).unwrap();

    //     // Validate the same block again (should be duplicate)
    //     let result = validator.validate_block(&chain, &valid_block);

    //     assert!(result.is_err(), "Expected block to be invalid due to duplication");
    //     assert_eq!(result.unwrap_err().to_string(), "Blockchain already contains block");
    // }

    // #[test]
    // fn test_validate_block_incorrect_height() {
    //     let blockchain = Arc::new(Mutex::new(create_mock_blockchain()));
    //     let validator = create_mock_block_validator();
    //     let chain = blockchain.lock().unwrap();

    //     // Create a block with incorrect height
    //     let last_block = chain.last_block().unwrap();
    //     let invalid_block = create_mock_block(2, last_block.header().hash().clone()); // Incorrect height

    //     // Validate the block
    //     let result = validator.validate_block(&chain, &invalid_block);

    //     assert!(result.is_err(), "Expected block to be invalid due to incorrect height");
    //     assert_eq!(result.unwrap_err().to_string(), "Block height is incorrect");
    // }

    // #[test]
    // fn test_validate_block_incorrect_prev_hash() {
    //     let blockchain = Arc::new(Mutex::new(create_mock_blockchain()));
    //     let validator = create_mock_block_validator();
    //     let chain = blockchain.lock().unwrap();

    //     // Create a block with incorrect previous hash
    //     let last_block = chain.last_block().unwrap();
    //     let invalid_block = create_mock_block(1, Hash::default()); // Incorrect previous hash

    //     // Validate the block
    //     let result = validator.validate_block(&chain, &invalid_block);

    //     assert!(result.is_err(), "Expected block to be invalid due to incorrect previous hash");
    //     assert_eq!(result.unwrap_err().to_string(), "Previous block hash is incorrect");
    // }

    // #[test]
    // fn test_validate_block_incorrect_tx_root() {
    //     let blockchain = Arc::new(Mutex::new(create_mock_blockchain()));
    //     let validator = create_mock_block_validator();
    //     let chain = blockchain.lock().unwrap();

    //     // Create a block with correct height and previous hash but incorrect tx_root
    //     let last_block = chain.last_block().unwrap();
    //     let mut invalid_block = create_mock_block(1, last_block.header().hash().clone());
    //     invalid_block.header_mut().tx_root = Hash::default(); // Incorrect tx_root

    //     // Validate the block
    //     let result = validator.validate_block(&chain, &invalid_block);

    //     assert!(result.is_err(), "Expected block to be invalid due to incorrect tx_root");
    //     assert_eq!(result.unwrap_err().to_string(), "Transaction root is invalid");
    // }

    // #[test]
    // fn test_validate_block_incorrect_state_root() {
    //     let blockchain = Arc::new(Mutex::new(create_mock_blockchain()));
    //     let validator = create_mock_block_validator();
    //     let chain = blockchain.lock().unwrap();

    //     // Create a block with correct height, prev_hash, and tx_root but incorrect state_root
    //     let last_block = chain.last_block().unwrap();
    //     let mut invalid_block = create_mock_block(1, last_block.header().hash().clone());
    //     invalid_block.header_mut().state_root = Hash::default(); // Incorrect state_root

    //     // Validate the block
    //     let result = validator.validate_block(&chain, &invalid_block);

    //     assert!(result.is_err(), "Expected block to be invalid due to incorrect state_root");
    //     assert_eq!(result.unwrap_err().to_string(), "State root is invalid");
    // }
}
