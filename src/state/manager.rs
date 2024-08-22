use tempfile::tempdir;

use crate::{
    core::{encoding::HexEncoding, error::CoreError},
    crypto::{address::Address, hash::Hash, utils::random_hash},
};

use super::{account::Account, storage::StateStorage};

pub struct StateManager {
    store: StateStorage,
}

impl StateManager {
    pub fn new(storage_path: &str) -> Self {
        Self {
            store: StateStorage::new(storage_path),
        }
    }

    pub fn get_account(&self, address: &Address) -> Option<Account> {
        self.store.get_account(address)
    }

    pub fn set_account(&self, address: &Address, account: &Account) -> Result<(), CoreError> {
        self.store.set_account(address, account)
    }

    pub fn backup_account(&self, address: &Address) -> Result<(), CoreError> {
        match self.get_account(address) {
            Some(acc) => self.store.backup_account(address, &acc),
            None => Err(CoreError::State(format!(
                "account with address: {} not found",
                address.to_hex()?
            ))),
        }
    }

    pub fn rollback_accounts(&self) -> Result<(), CoreError> {
        self.store.rollback_accounts()
    }

    pub fn gen_state_root(&self) -> Result<Hash, CoreError> {
        Ok(random_hash())
    }

    pub fn new_in_memory() -> Self {
        let temp_dir = tempdir().unwrap();
        let db_path = temp_dir.path().to_str().unwrap();
        Self {
            store: StateStorage::new(db_path),
        }
    }
}

impl Default for StateManager {
    fn default() -> Self {
        Self::new("data/state.db")
    }
}
