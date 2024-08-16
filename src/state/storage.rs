use rocksdb::{Options, DB};

use crate::{core::encoding::ByteEncoding, crypto::address::Address};

use crate::state::account::Account;

pub struct StateStorage {
    db: DB,
}

impl StateStorage {
    pub fn new(path: &str) -> Self {
        let mut options = Options::default();
        options.create_if_missing(true);
        let db = DB::open(&options, path).unwrap();

        StateStorage { db }
    }

    pub fn get_account(&self, address: &Address) -> Option<Account> {
        match self.db.get(address) {
            Ok(Some(value)) => match Account::from_bytes(&value) {
                Ok(acc) => Some(acc),
                Err(e) => None,
            },
            Ok(None) => None,
            Err(_) => None,
        }
    }

    pub fn set_account(&self, address: &Address, account: &Account) {
        let serialized = account.to_bytes().unwrap();
        self.db.put(address, serialized).unwrap();
    }

    pub fn delete_account(&self, address: &Address) {
        self.db.delete(address).unwrap();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::crypto::address::Address;
    use crate::state::account::Account;
    use tempfile::tempdir;

    #[test]
    fn test_state_storage_get_set_account() {
        // Create a temporary directory for RocksDB
        let temp_dir = tempdir().unwrap();
        let storage = StateStorage::new(temp_dir.path().to_str().unwrap());

        // Create an account and an address
        let address_data = [1u8; 20];
        let address = Address::new(&address_data);
        let account = Account {
            balance: 1000,
            nonce: 0,
        };

        // Store the account
        storage.set_account(&address, &account);

        // Retrieve the account and check if it matches
        let retrieved_account = storage.get_account(&address);
        assert!(retrieved_account.is_some());
        let retrieved_account = retrieved_account.unwrap();
        assert_eq!(retrieved_account.balance, account.balance);
        assert_eq!(retrieved_account.nonce, account.nonce);
    }

    #[test]
    fn test_state_storage_get_nonexistent_account() {
        // Create a temporary directory for RocksDB
        let temp_dir = tempdir().unwrap();
        let storage = StateStorage::new(temp_dir.path().to_str().unwrap());

        // Create an address
        let address_data = [1u8; 20];
        let address = Address::new(&address_data);

        // Attempt to retrieve a non-existent account
        let retrieved_account = storage.get_account(&address);
        assert!(retrieved_account.is_none());
    }

    #[test]
    fn test_state_storage_delete_account() {
        // Create a temporary directory for RocksDB
        let temp_dir = tempdir().unwrap();
        let storage = StateStorage::new(temp_dir.path().to_str().unwrap());

        // Create an account and an address
        let address_data = [1u8; 20];
        let address = Address::new(&address_data);
        let account = Account {
            balance: 1000,
            nonce: 0,
        };

        // Store the account
        storage.set_account(&address, &account);

        // Delete the account
        storage.delete_account(&address);

        // Ensure the account is no longer in the storage
        let retrieved_account = storage.get_account(&address);
        assert!(retrieved_account.is_none());
    }
}
