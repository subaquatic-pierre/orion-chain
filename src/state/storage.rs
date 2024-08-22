use log::{error, warn};
use rocksdb::{ColumnFamilyDescriptor, Options, DB};

use crate::core::encoding::HexEncoding;
use crate::core::error::CoreError;
use crate::{core::encoding::ByteEncoding, crypto::address::Address};

use crate::state::account::Account;

pub struct StateStorage {
    db: DB,
    account_cf: String,
}

impl StateStorage {
    pub fn new(path: &str) -> Self {
        let account_cf = "account_cf".to_string();

        let mut options = Options::default();
        options.create_if_missing(true);
        options.create_missing_column_families(true);

        let account_cf_descriptor = ColumnFamilyDescriptor::new(&account_cf, Options::default());

        let db = DB::open_cf_descriptors(&options, path, vec![account_cf_descriptor])
            .expect("Unable to open DB with column families");

        Self { db, account_cf }
    }

    pub fn get_account(&self, address: &Address) -> Option<Account> {
        let addr_str = match address.to_hex() {
            Ok(str) => str,
            Err(e) => {
                error!("unable to convert address to hex in StateStorage.get_account, {e}");
                return None;
            }
        };

        let account = match self.db.cf_handle(&self.account_cf) {
            Some(handle) => match self.db.get_cf(handle, &addr_str) {
                Ok(Some(value)) => {
                    match Account::from_bytes(&value) {
                        Ok(acc) => Some(acc),
                        Err(e) => {
                            error!("unable to convert account from bytes in StateStorage.get_account, {e}");
                            None
                        }
                    }
                }
                Ok(None) => {
                    warn!("no account found for address: {addr_str} in StateStorage.get_account");
                    None
                }
                Err(e) => {
                    error!("unable to get account data from ColumnFamily in StateStorage.get_account, {e}");
                    None
                }
            },
            None => {
                warn!("unable to get account ColumnFamily in StateStorage");
                None
            }
        };
        account
    }

    pub fn set_account(&self, address: &Address, account: &Account) -> Result<(), CoreError> {
        let addr_str = address.to_hex()?;
        match self.db.cf_handle(&self.account_cf) {
            Some(handle) => {
                self.db
                    .put_cf(handle, &addr_str, account.to_bytes()?)
                    .map_err(|e| {
                        CoreError::State(format!(
                            "unable to put address: {} in StateStorage, {e}",
                            addr_str
                        ))
                    })?;
                Ok(())
            }
            None => Err(CoreError::State(
                "unable to get ColumnFamily handle in StateStorage.set_account".to_string(),
            )),
        }
    }

    pub fn delete_account(&self, address: &Address) -> Result<(), CoreError> {
        let addr_str = address.to_hex()?;

        match self.db.cf_handle(&self.account_cf) {
            Some(handle) => {
                self.db.delete_cf(handle, addr_str).unwrap();
            }
            None => error!("unable to get ColumnFamily handle in StateStorage.delete_account"),
        }

        Ok(())
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
        storage.set_account(&address, &account).unwrap();

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
        storage.set_account(&address, &account).unwrap();

        // Delete the account
        storage.delete_account(&address).unwrap();

        // Ensure the account is no longer in the storage
        let retrieved_account = storage.get_account(&address);
        assert!(retrieved_account.is_none());
    }
}
