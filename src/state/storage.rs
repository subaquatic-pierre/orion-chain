use log::{error, warn};
use rocksdb::{ColumnFamilyDescriptor, Options, WriteBatch, DB};

use crate::core::encoding::HexEncoding;
use crate::core::error::CoreError;
use crate::{core::encoding::ByteEncoding, crypto::address::Address};

use crate::state::account::Account;

pub struct StateStorage {
    db: DB,
    account_cf: String,
    backup_account_cf: String,
}

impl StateStorage {
    pub fn new(path: &str) -> Self {
        let account_cf = "account_cf".to_string();
        let backup_account_cf = "backup_account_cf".to_string();

        let mut options = Options::default();
        options.create_if_missing(true);
        options.create_missing_column_families(true);

        let account_cf_descriptor = ColumnFamilyDescriptor::new(&account_cf, Options::default());
        let backup_account_cf_descriptor =
            ColumnFamilyDescriptor::new(&backup_account_cf, Options::default());

        let db = DB::open_cf_descriptors(
            &options,
            path,
            vec![account_cf_descriptor, backup_account_cf_descriptor],
        )
        .expect("Unable to open DB with column families");

        Self {
            db,
            account_cf,
            backup_account_cf,
        }
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

    pub fn backup_account(&self, address: &Address, account: &Account) -> Result<(), CoreError> {
        let addr_str = address.to_hex()?;
        match self.db.cf_handle(&self.backup_account_cf) {
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
                "unable to get ColumnFamily handle in StateStorage.backup_account".to_string(),
            )),
        }
    }

    pub fn rollback_accounts(&self) -> Result<(), CoreError> {
        // Get the handle for the backup column family
        let backup_handle = match self.db.cf_handle(&self.backup_account_cf) {
            Some(handle) => handle,
            None => {
                return Err(CoreError::State(
                    "unable to get ColumnFamily handle in rollback_account_backups".to_string(),
                ))
            }
        };

        // Iterate over all key-value pairs in the backup column family
        let backup_iter = self
            .db
            .iterator_cf(backup_handle, rocksdb::IteratorMode::Start);

        let mut batch = WriteBatch::default();

        for iter in backup_iter {
            match iter {
                Ok((key, value)) => {
                    let addr_str = String::from_utf8(key.to_vec()).map_err(|e| {
                        CoreError::State(format!("failed to convert key to string: {}", e))
                    })?;
                    let address = Address::from_hex(&addr_str)?;

                    // Convert the value bytes back to Account
                    let account = Account::from_bytes(&value)?;

                    // Restore the account to the state storage
                    self.set_account(&address, &account)?;

                    // add key to batch delete which will clear all account backups at end
                    batch.delete_cf(backup_handle, &key);
                }
                Err(e) => {
                    error!("unable to iterate through account_backup_cf in StateStorage.rollback_accounts, {e}")
                }
            }
        }

        // Clear all entries in the backup column family
        // Apply the batch delete operations
        self.db.write(batch).map_err(|e| {
            CoreError::State(format!(
                "failed to apply delete all backup accounts batch operations to backup column family: {e}"
            ))
        })?;

        Ok(())
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
        let account = Account { balance: 1000 };

        // Store the account
        storage.set_account(&address, &account).unwrap();

        // Retrieve the account and check if it matches
        let retrieved_account = storage.get_account(&address);
        assert!(retrieved_account.is_some());
        let retrieved_account = retrieved_account.unwrap();
        assert_eq!(retrieved_account.balance, account.balance);
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
        let account = Account { balance: 1000 };

        // Store the account
        storage.set_account(&address, &account).unwrap();

        // Delete the account
        storage.delete_account(&address).unwrap();

        // Ensure the account is no longer in the storage
        let retrieved_account = storage.get_account(&address);
        assert!(retrieved_account.is_none());
    }

    #[test]
    fn test_backup_account() {
        let dir = tempdir().unwrap();
        let path = dir.path().to_str().unwrap();
        let state_storage = StateStorage::new(path);

        // Create an account and an address
        let address_data = [1u8; 20];
        let address = Address::new(&address_data);
        let account = Account { balance: 100 };

        // Backup the account
        state_storage.backup_account(&address, &account).unwrap();

        // Verify that the account is backed up
        let backup_handle = state_storage
            .db
            .cf_handle(&state_storage.backup_account_cf)
            .unwrap();
        let backup_value = state_storage
            .db
            .get_cf(backup_handle, &address.to_hex().unwrap())
            .unwrap()
            .unwrap();
        let backed_up_account = Account::from_bytes(&backup_value).unwrap();

        assert_eq!(backed_up_account.balance, 100);
    }

    #[test]
    fn test_rollback_accounts() {
        let dir = tempdir().unwrap();
        let path = dir.path().to_str().unwrap();
        let state_storage = StateStorage::new(path);

        let address_data = [1u8; 20];
        let address1 = Address::new(&address_data);
        let account1 = Account { balance: 100 };

        let address_data = [2u8; 20];
        let address2 = Address::new(&address_data);
        let account2 = Account { balance: 200 };

        // Backup the accounts
        state_storage.backup_account(&address1, &account1).unwrap();
        state_storage.backup_account(&address2, &account2).unwrap();

        // Apply some changes to the state (simulate updates)
        state_storage
            .set_account(
                &address1,
                &Account {
                    balance: 50, /* other fields... */
                },
            )
            .unwrap();
        state_storage
            .set_account(
                &address2,
                &Account {
                    balance: 150, /* other fields... */
                },
            )
            .unwrap();

        // Rollback accounts
        state_storage.rollback_accounts().unwrap();

        // Verify that accounts are restored
        let restored_account1 = state_storage.get_account(&address1).unwrap();
        let restored_account2 = state_storage.get_account(&address2).unwrap();

        assert_eq!(restored_account1.balance, 100);
        assert_eq!(restored_account2.balance, 200);

        // Verify that backup column family is empty
        let backup_handle = state_storage
            .db
            .cf_handle(&state_storage.backup_account_cf)
            .unwrap();
        let mut backup_iter = state_storage
            .db
            .iterator_cf(backup_handle, rocksdb::IteratorMode::Start);
        assert!(backup_iter.next().is_none()); // Backup column family should be empty
    }
}
