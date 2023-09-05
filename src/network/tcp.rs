use bytes::Buf;
use log::{info, warn};

use std::io::{BufReader, BufWriter, ErrorKind, Read, Result as IoResult, Write};

use crate::network::error::NetworkError;
use std::borrow::{BorrowMut, Cow};
use std::collections::HashMap;
use std::error::Error;
use std::net::{SocketAddr, TcpListener, TcpStream};
use std::ops::Deref;
use std::sync::mpsc::{channel, Receiver, Sender};
use std::sync::{Arc, Mutex};
use std::thread::{self, JoinHandle};
use std::time;

pub struct BufTcpStream {
    input: BufReader<TcpStream>,
    output: BufWriter<TcpStream>,
}

impl BufTcpStream {
    fn new(stream: TcpStream) -> IoResult<Self> {
        let input = BufReader::new(stream.try_clone()?);
        let output = BufWriter::new(stream);

        Ok(Self { input, output })
    }

    pub fn reader(&mut self) -> &mut BufReader<TcpStream> {
        &mut self.input
    }

    pub fn writer(&mut self) -> &mut BufWriter<TcpStream> {
        &mut self.output
    }
}

use super::{
    rpc::RPC,
    transport::Transport,
    types::{ArcMut, NetAddr, Payload},
};

#[derive(Debug)]
pub enum PeerStreamDirection {
    Incoming,
    Outgoing,
}

#[derive(Debug)]
pub enum PeerHealth {
    Error(String, SocketAddr),
    Message(String),
    Disconnect(SocketAddr),
}

// #[derive(Debug)]
pub struct TcpPeer {
    reader: ArcMut<BufReader<TcpStream>>,
    writer: ArcMut<BufWriter<TcpStream>>,
    direction: PeerStreamDirection,
    remote_addr: SocketAddr,
    node_addr: SocketAddr,
    rpc_tx: Arc<Mutex<Sender<RPC>>>,
    health_tx: Arc<Mutex<Sender<PeerHealth>>>,
}

impl TcpPeer {
    pub fn new(
        remote_addr: SocketAddr,
        node_addr: SocketAddr,
        reader: ArcMut<BufReader<TcpStream>>,
        writer: ArcMut<BufWriter<TcpStream>>,
        direction: PeerStreamDirection,
        rpc_tx: Arc<Mutex<Sender<RPC>>>,
        health_tx: Arc<Mutex<Sender<PeerHealth>>>,
    ) -> Self {
        Self {
            remote_addr,
            node_addr,
            reader,
            writer,
            direction,
            rpc_tx,
            health_tx,
        }
    }

    pub fn handle_incoming(&mut self) {
        let stream = self.reader.clone();
        let health_tx = self.health_tx.clone();
        let mut buf = [0u8; 1024];

        let remote_addr = self.remote_addr;
        let node_addr = self.node_addr;
        let rpc_tx = self.rpc_tx.clone();

        thread::spawn(move || {
            if let Ok(reader) = stream.lock().as_mut() {
                loop {
                    // let reader = stream.reader();
                    match reader.read(&mut buf) {
                        Ok(bytes_read) => {
                            if bytes_read == 0 {
                                if let Ok(tx) = health_tx.lock() {
                                    tx.send(PeerHealth::Disconnect(remote_addr)).ok();
                                    break;
                                }
                            }
                            if let Ok(tx) = rpc_tx.lock() {
                                let rpc = RPC {
                                    sender: remote_addr.to_string(),
                                    receiver: node_addr.to_string(),
                                    payload: buf.to_vec(),
                                };
                                tx.send(rpc).ok();
                                buf = [0_u8; 1024];
                            }
                        }
                        Err(e) if e.kind() == ErrorKind::ConnectionAborted => {
                            if let Ok(health_tx) = health_tx.lock() {
                                health_tx
                                    .send(PeerHealth::Error(e.to_string(), remote_addr))
                                    .ok();
                                break;
                            }
                        }
                        Err(e) => {
                            if let Ok(health_tx) = health_tx.lock() {
                                health_tx
                                    .send(PeerHealth::Error(e.to_string(), remote_addr))
                                    .ok();
                                break;
                            }
                        }
                    }
                }
            }
        });
    }

    pub fn send_msg(&mut self, msg: &[u8]) {
        let remote_addr = self.remote_addr;
        info!("trying to send message");
        if let Ok(writer) = self.writer.lock().as_mut() {
            if let Ok(n) = writer.write(msg) {
                info!("message sent to: {remote_addr:?}, num bytes: {n}",)
            }
            writer.flush().unwrap();
        }
    }
}

pub struct TcpTransport {
    node_addr: SocketAddr,
    listener: ArcMut<TcpListener>,
    peers: ArcMut<HashMap<SocketAddr, TcpPeer>>,

    // channel used to send messages to node
    rpc_rx: Arc<Mutex<Receiver<RPC>>>,
    rpc_tx: Arc<Mutex<Sender<RPC>>>,

    // channel used for peer health
    health_rx: ArcMut<Receiver<PeerHealth>>,
    health_tx: ArcMut<Sender<PeerHealth>>,
}

impl TcpTransport {
    pub fn new(
        node_addr: SocketAddr,
        rpc_tx: Arc<Mutex<Sender<RPC>>>,
        rpc_rx: Arc<Mutex<Receiver<RPC>>>,
    ) -> Self {
        let listener = TcpListener::bind(node_addr).unwrap();

        let (tx, rx) = channel::<PeerHealth>();
        let (health_tx, health_rx) = (ArcMut::new(tx), ArcMut::new(rx));

        Self {
            node_addr,
            listener: ArcMut::new(listener),
            peers: ArcMut::new(HashMap::new()),
            rpc_rx,
            rpc_tx,
            health_rx,
            health_tx,
        }
    }

    pub fn init(&mut self) {
        let peers = self.peers.clone();
        let listener = self.listener.clone();
        let tx = self.rpc_tx.clone();
        let health_tx = self.health_tx.clone();

        let node_addr = self.node_addr.clone();
        thread::spawn(move || {
            info!("initialized new tcp transport {:?}", node_addr);

            for stream in listener.lock().unwrap().incoming().flatten() {
                let remote_addr = stream.peer_addr().unwrap();
                info!("new peer connected with remote address {:?}", remote_addr);

                let input = BufReader::new(stream.try_clone().unwrap());
                let output = BufWriter::new(stream);

                let (reader, writer) = (ArcMut::new(input), ArcMut::new(output));

                let mut peer = TcpPeer::new(
                    remote_addr,
                    node_addr,
                    reader,
                    writer,
                    PeerStreamDirection::Incoming,
                    tx.clone(),
                    health_tx.clone(),
                );

                peer.handle_incoming();
                peers.lock().unwrap().insert(remote_addr, peer);
            }
        });

        // spawn thread to send periodic messages to peers
        let peers = self.peers.clone();
        thread::spawn(move || loop {
            info!(
                "trying to send to all peers {:?}",
                peers.lock().unwrap().keys()
            );
            for (_, peer) in peers.lock().as_mut().unwrap().iter_mut() {
                peer.send_msg(b"Hello from the other side");
            }
            thread::sleep(time::Duration::from_secs(5));
        });

        // spawn thread to get health messages from peers
        let peers = self.peers.clone();
        let health_rx = self.health_rx.clone();
        thread::spawn(move || {
            if let Ok(rx) = health_rx.lock() {
                for msg in rx.iter() {
                    match msg {
                        PeerHealth::Disconnect(addr) => {
                            info!("removing peer from peer list {addr:?}");
                            peers.lock().unwrap().remove(&addr);
                        }
                        PeerHealth::Error(msg, addr) => {
                            info!("error message {msg}");
                            peers.lock().unwrap().remove(&addr);
                        }
                        msg => {
                            info!("message received from peer in health checkers {msg:?}")
                        }
                    };
                }
            }
        });
    }
}

impl Transport for TcpTransport {
    fn address(&self) -> NetAddr {
        self.node_addr.to_string()
    }

    fn receiver(&self) -> Arc<Mutex<Receiver<RPC>>> {
        self.rpc_rx.clone()
    }

    fn send_msg(&self, from_addr: NetAddr, payload: Payload) -> Result<(), NetworkError> {
        Ok(())
    }
}
