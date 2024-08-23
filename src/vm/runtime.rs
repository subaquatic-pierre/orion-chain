use crate::{
    core::{
        encoding::ByteEncoding,
        error::CoreError,
        transaction::{BlockRewardData, Transaction, TransferData, TxType},
    },
    state::manager::StateManager,
};

use super::types::RuntimeExecData;

pub struct ValidatorRuntime;

impl ValidatorRuntime {
    pub fn new() -> Self {
        Self
    }

    pub fn execute(&self, exec_data: RuntimeExecData) -> Result<(), CoreError> {
        let RuntimeExecData { tx, state, backup } = exec_data;

        match tx.tx_type {
            TxType::BlockReward | TxType::GasReward => {
                let data = BlockRewardData::from_bytes(&tx.data)?;
                self.execute_block_reward(data, state, backup)
            }
            TxType::Transfer => {
                let data = TransferData::from_bytes(&tx.data)?;
                self.execute_transfer(data, state, backup)
            }
            _ => todo!(),
        }
    }

    fn execute_block_reward(
        &self,
        data: BlockRewardData,
        state: &StateManager,
        backup: bool,
    ) -> Result<(), CoreError> {
        if backup {
            state.backup_account(&data.to)?;
        }

        let mut to_account = state
            .get_account(&data.to)
            .ok_or_else(|| CoreError::State("account not found".to_string()))?;

        to_account.balance += data.amount;

        state.set_account(&data.to, &to_account)?;

        Ok(())
    }

    fn execute_transfer(
        &self,
        data: TransferData,
        state: &StateManager,
        backup: bool,
    ) -> Result<(), CoreError> {
        if backup {
            state.backup_account(&data.from)?;
            state.backup_account(&data.to)?;
        }

        let mut from_account = state
            .get_account(&data.from)
            .ok_or_else(|| CoreError::State("account not found".to_string()))?;
        let mut to_account = state
            .get_account(&data.to)
            .ok_or_else(|| CoreError::State("account not found".to_string()))?;

        if from_account.balance < data.amount {
            return Err(CoreError::State("Insufficient balance".to_string()));
        }

        from_account.balance -= data.amount;
        to_account.balance += data.amount;

        state.set_account(&data.from, &from_account)?;
        state.set_account(&data.to, &to_account)?;

        Ok(())
    }
}

// TODO: runtime tests
#[cfg(test)]
mod tests {
    use super::*;
    use std::str;
    use tempfile::tempdir;

    // #[test]
    // fn test_execute_transfer_success() {
    // }

    // #[test]
    // fn test_execute_transfer_insufficient_balance() {
    // }

    // #[test]
    // fn test_execute_transfer_account_not_found() {
    // }
}
