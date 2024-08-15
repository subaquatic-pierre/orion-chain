use std::net::SocketAddr;
use std::ops::Deref;
use std::sync::{Arc, Mutex};

use crate::rpc::types::RPC;

pub type NetAddr = String;
pub type Payload = Vec<u8>;

pub type RpcChanMsg = (SocketAddr, RPC);

pub struct ArcMut<T> {
    pub inner: Arc<Mutex<T>>,
}

impl<T> ArcMut<T> {
    pub fn new(data: T) -> Self {
        Self {
            inner: Arc::new(Mutex::new(data)),
        }
    }
}

impl<T> Deref for ArcMut<T> {
    type Target = Arc<Mutex<T>>;
    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}
