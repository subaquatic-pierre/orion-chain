use crate::{core::transaction::Transaction, state::manager::StateManager};

pub struct RuntimeExecData<'a> {
    pub tx: &'a Transaction,
    pub state: &'a StateManager,
    pub backup: bool,
}

impl<'a> RuntimeExecData<'a> {
    pub fn new(tx: &'a Transaction, state: &'a StateManager) -> Self {
        Self {
            tx,
            state,
            backup: false,
        }
    }

    pub fn new_with_backup(tx: &'a Transaction, state: &'a StateManager) -> Self {
        Self {
            tx,
            state,
            backup: true,
        }
    }
}
