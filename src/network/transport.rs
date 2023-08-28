use log::{info, warn};

use crate::network::error::NetworkError;
use std::borrow::{BorrowMut, Cow};

use std::error::Error;
use std::ops::Deref;
use std::sync::mpsc::{channel, Receiver, Sender};
use std::sync::{Arc, Mutex};
use std::thread::{self, JoinHandle};

pub type NetAddr = String;
pub type Payload = Vec<u8>;

pub struct ArcMut<T> {
    inner: Arc<Mutex<T>>,
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

#[derive(Debug, Clone)]
pub struct RPC {
    pub sender: NetAddr,
    pub receiver: NetAddr,
    pub payload: Payload,
}

pub trait Transport {
    fn address(&self) -> NetAddr;
    fn send_msg(&self, from_addr: NetAddr, payload: Payload) -> Result<(), NetworkError>;
    fn receiver(&self) -> Arc<Mutex<Receiver<RPC>>>;
}

pub struct HttpTransport {
    addr: NetAddr,
    rx: ArcMut<Receiver<RPC>>,
    tx: ArcMut<Sender<RPC>>,
}

impl Transport for HttpTransport {
    fn address(&self) -> NetAddr {
        self.addr.to_string()
    }

    fn receiver(&self) -> Arc<Mutex<Receiver<RPC>>> {
        self.rx.clone()
    }

    fn send_msg(&self, from_addr: NetAddr, payload: Payload) -> Result<(), NetworkError> {
        Ok(())
    }
}

pub struct LocalTransport {
    addr: NetAddr,
    rx: ArcMut<Receiver<RPC>>,
    tx: ArcMut<Sender<RPC>>,
}

impl LocalTransport {
    pub fn new(addr: &str) -> Self {
        let (tx, rx) = channel::<RPC>();
        let (tx, rx) = (ArcMut::new(tx), ArcMut::new(rx));

        LocalTransport {
            addr: addr.to_string(),
            tx,
            rx,
        }
    }
}

impl Transport for LocalTransport {
    fn address(&self) -> NetAddr {
        self.addr.to_string()
    }

    fn send_msg(&self, from_addr: NetAddr, payload: Payload) -> Result<(), NetworkError> {
        let rpc = RPC {
            sender: from_addr.to_string(),
            receiver: self.address().to_string(),
            payload,
        };

        if let Ok(tx) = self.tx.lock() {
            if tx.send(rpc.clone()).is_ok() {
                return Ok(());
            } else {
                return Err(NetworkError::Message(format!(
                    "unable to send message: {rpc:?}"
                )));
            }
        }
        return Err(NetworkError::Message(format!(
            "unable to send message: {rpc:?}"
        )));
    }

    fn receiver(&self) -> Arc<Mutex<Receiver<RPC>>> {
        self.rx.clone()
    }
}

pub struct TransportManager<T>
where
    T: Transport,
{
    peers: Vec<T>,
    threads: Vec<JoinHandle<()>>,
}

impl TransportManager<LocalTransport> {
    pub fn new() -> Self {
        Self {
            peers: vec![],
            threads: vec![],
        }
    }

    pub fn send_msg(
        &self,
        from_addr: NetAddr,
        to_addr: NetAddr,
        payload: Payload,
    ) -> Result<(), NetworkError> {
        let from_ts = self.peers.iter().find(|&ts| ts.address() == from_addr);

        if from_addr == to_addr {
            let msg = format!(
                "cannot send rpc message to self, from address: {from_addr}, to address: {to_addr}"
            );
            warn!("{msg}");
            return Err(NetworkError::NotFound(msg));
        }

        if from_ts.is_none() {
            let msg = format!("to transport address not found: {from_addr}");
            warn!("{msg}");
            return Err(NetworkError::NotFound(msg));
        }

        let to_ts = self.peers.iter().find(|&ts| ts.address() == to_addr);

        if let Some(to_ts) = to_ts {
            to_ts.send_msg(from_addr, payload)?;
            Ok(())
        } else {
            let msg = format!("to transport address not found: {to_addr}");
            warn!("{msg}");
            Err(NetworkError::NotFound(msg))
        }
    }

    pub fn peers(&self) -> &Vec<LocalTransport> {
        &self.peers
    }

    pub fn connect(&mut self, ts: LocalTransport) -> Result<(), NetworkError> {
        self.peers.push(ts);
        Ok(())
    }

    pub fn init(&mut self, server_tx: Arc<Mutex<Sender<RPC>>>) -> Result<(), NetworkError> {
        let mut txs = vec![];
        for ts in self.peers().iter() {
            txs.push((ts.receiver(), server_tx.clone()));
        }

        // let srv_clone = Arc::new(server_tx);
        for (rx, tx) in txs {
            // srv_clone.clone();
            let th = thread::spawn(move || {
                if let Ok(rx) = rx.lock() {
                    while let Ok(msg) = rx.recv() {
                        if let Ok(tx) = tx.lock() {
                            if let Err(e) = tx.send(msg.clone()) {
                                warn!("there was an error sending message to sever: {msg:?}, {e}")
                            }
                        }
                    }
                }
            });
            self.threads.push(th);
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use core::time;

    use super::*;

    #[test]
    fn test_connect() {
        let mut ts_manager = TransportManager::new();

        let ts1 = LocalTransport::new("local");
        let ts2 = LocalTransport::new("remote");

        ts_manager.connect(ts1).unwrap();
        ts_manager.connect(ts2).unwrap();

        assert_eq!(ts_manager.peers().len(), 2);

        let found = ts_manager
            .peers()
            .iter()
            .find(|&t| t.address() == "remote".to_string());

        assert_eq!(found.is_some(), true);
        let found = ts_manager
            .peers()
            .iter()
            .find(|&t| t.address() == "local".to_string());

        assert_eq!(found.is_some(), true);
    }
    #[test]
    fn test_send_msg() {
        let (server_tx, server_rx) = channel::<RPC>();
        let mut ts_manager = TransportManager::new();

        let ts1 = LocalTransport::new("local");
        let ts2 = LocalTransport::new("remote");

        ts_manager.connect(ts1).unwrap();
        ts_manager.connect(ts2).unwrap();

        let server_tx = Arc::new(Mutex::new(server_tx));
        ts_manager.init(server_tx).unwrap();

        // ensure error if transport not found in manager
        assert_eq!(
            ts_manager
                .send_msg("from_addr".to_string(), "to_addr".to_string(), vec![])
                .is_err(),
            true
        );

        let clone = Arc::new(ts_manager);

        let handle = thread::spawn(move || {
            clone
                .send_msg("local".to_string(), "remote".to_string(), vec![])
                .unwrap();
            clone
                .send_msg("local".to_string(), "remote".to_string(), vec![])
                .unwrap();
            thread::sleep(time::Duration::from_millis(1))
        });

        let mut msgs = vec![];

        handle.join();

        for msg in server_rx.try_iter() {
            msgs.push(msg)
        }

        // assert messages are in msg array
        assert_eq!(msgs.len(), 2);
    }
}
