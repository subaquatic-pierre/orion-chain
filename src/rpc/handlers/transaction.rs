use std::sync::{Arc, Mutex};

use log::debug;

use crate::{
    core::{encoding::ByteEncoding, transaction::Transaction},
    crypto::private_key::PrivateKey,
    network::{error::NetworkError, tx_pool::TxPool, types::ArcMut},
    rpc::types::RPC,
};

pub fn new_tx(rpc: &RPC, mem_pool: Arc<Mutex<TxPool>>) -> Result<Transaction, NetworkError> {
    let tx = Transaction::from_bytes(&rpc.payload);

    match tx {
        Ok(mut tx) => {
            let key = PrivateKey::new();
            tx.sign(&key)?;
            if let Ok(mut mem_pool) = mem_pool.lock() {
                mem_pool.add(tx.clone());
                debug!(
                    "adding transaction to the mem_pool in RpcController, hash: {}",
                    tx.hash()
                );
                Ok(tx)
            } else {
                Err(NetworkError::RPC(
                    "unable to lock mem_pool in RpcController".to_string(),
                ))
            }
        }
        Err(e) => Err(NetworkError::RPC(format!(
            "unable to handle RpcHeader::NewTx in RpcController, {e}"
        ))),
    }
}
