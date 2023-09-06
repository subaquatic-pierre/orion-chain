use bytes::Buf;
use log::{debug, error, info, warn};

use std::io::{BufReader, BufWriter, ErrorKind, Read, Result as IoResult, Write};

use crate::core::encoding::{ByteDecoding, ByteEncoding};
use crate::core::util::timestamp;
use crate::network::error::NetworkError;
use std::borrow::{BorrowMut, Cow};
use std::collections::HashMap;
use std::error::Error;
use std::net::{IpAddr, Ipv4Addr, Ipv6Addr, SocketAddr, TcpListener, TcpStream};
use std::ops::Deref;
use std::sync::mpsc::{channel, Receiver, Sender};
use std::sync::{Arc, Mutex};
use std::thread::{self, JoinHandle};
use std::time;

use super::{
    message::PeerMessage,
    peer::{PeerStreamDirection, TcpPeer},
    rpc::RPC,
    transport::Transport,
    types::{ArcMut, NetAddr, Payload},
};

pub struct TcpController {
    node_addr: SocketAddr,
    hb_interval: u64,
    listener: ArcMut<TcpListener>,
    peers: ArcMut<HashMap<SocketAddr, TcpPeer>>,

    // channel used to send messages to ChainNode
    rpc_tx: Arc<Mutex<Sender<RPC>>>,

    // channel used to communicate with peer
    peer_msg_rx: ArcMut<Receiver<PeerMessage>>,
    peer_msg_tx: ArcMut<Sender<PeerMessage>>,
}

impl TcpController {
    pub fn new(
        node_addr: SocketAddr,
        rpc_tx: Arc<Mutex<Sender<RPC>>>,
    ) -> Result<Self, NetworkError> {
        let listener = match TcpListener::bind(node_addr) {
            Ok(listener) => listener,
            Err(e) => return Err(NetworkError::Connect(e.to_string())),
        };

        // create channels to be used to communicate with remote peers
        let (tx, rx) = channel::<PeerMessage>();
        let (peer_msg_tx, peer_msg_rx) = (ArcMut::new(tx), ArcMut::new(rx));

        // TODO: CONFIG, get heartbeat interval from config

        Ok(Self {
            node_addr,
            listener: ArcMut::new(listener),
            peers: ArcMut::new(HashMap::new()),
            rpc_tx,
            peer_msg_rx,
            peer_msg_tx,
            hb_interval: 5,
        })
    }

    pub fn init(&mut self, known_peers: Vec<SocketAddr>) {
        let peers = self.peers.clone();
        let listener = self.listener.clone();
        let peer_msg_tx = self.peer_msg_tx.clone();
        let node_addr = self.node_addr;

        // spawn main thread to listen to incoming connections
        // create new peer and add to peer set on each
        // new stream established
        thread::spawn(move || {
            info!("initialized new TCP controller for ChainNode at address: {node_addr}");

            if let Ok(listener) = listener.lock() {
                for stream in listener.incoming().flatten() {
                    let remote_addr = stream.peer_addr().unwrap();
                    info!("new peer connected with remote address: {remote_addr}");

                    // split tcp stream, used for incoming and outgoing messages
                    let (reader, writer) = split_stream(stream);

                    let mut peer = TcpPeer::new(
                        remote_addr,
                        node_addr,
                        PeerStreamDirection::Incoming,
                        reader,
                        writer,
                        peer_msg_tx.clone(),
                    );

                    // start handler for incoming messages on peer
                    peer.spawn_incoming_handler();

                    // insert peer into peer set
                    peers.lock().unwrap().insert(remote_addr, peer);
                }
            } else {
                error!("unable to get lock on listener in TCP controller");
            }
        });

        let peers = self.peers.clone();
        let hb_interval = self.hb_interval;
        // spawn thread to send heartbeat messages to peers
        thread::spawn(move || loop {
            debug!(
                "trying to send to all peers {:?}",
                peers.lock().unwrap().keys()
            );
            for (addr, peer) in peers.lock().as_mut().unwrap().iter_mut() {
                let msg = PeerMessage::Ping(*addr, b"PING".to_vec());
                peer.send_msg(&msg);
            }
            thread::sleep(time::Duration::from_secs(hb_interval));

            // TODO: check peer last heartbeat, remove if older than last
            // heartbeat threshold
        });

        // get data to be used in thread below
        let peers = self.peers.clone();
        let rpc_tx = self.rpc_tx.clone();
        let peer_msg_rx = self.peer_msg_rx.clone();

        // spawn main thread to handle messages from peers
        thread::spawn(move || {
            if let Ok(peer_msg_rx) = peer_msg_rx.lock() {
                for msg in peer_msg_rx.iter() {
                    match msg {
                        PeerMessage::Disconnect(addr, msg) => {
                            info!(
                                "disconnect message received, removing peer from peer list {addr}, message: {msg}"
                            );
                            peers.lock().unwrap().remove(&addr);
                            debug!("DISCONNECT message received from: {addr}");
                        }
                        PeerMessage::Error(addr, msg) => {
                            warn!("error received from peer: {addr} with message: {msg}");
                            peers.lock().unwrap().remove(&addr);
                            debug!("ERROR message received from: {addr}");
                        }
                        PeerMessage::RPC(addr, rpc_payload) => {
                            // Send message back to ChainNode
                            let rpc = RPC {
                                sender: addr.to_string(),
                                receiver: node_addr.to_string(),
                                payload: rpc_payload,
                            };
                            rpc_tx.lock().unwrap().send(rpc).unwrap();
                            debug!("RPC message received from: {addr}");
                        }
                        PeerMessage::Ping(addr, _) => {
                            // return pong message to peer
                            if let Some(peer) = peers.lock().unwrap().get_mut(&addr) {
                                let ts = timestamp(time::SystemTime::now());
                                let pong_msg = PeerMessage::Pong(addr, vec![]);
                                peer.send_msg(&pong_msg);
                                debug!("PING message received from: {addr}");
                            }
                        }
                        PeerMessage::Pong(addr, _) => {
                            // update last heartbeat on peer
                            if let Some(peer) = peers.lock().unwrap().get_mut(&addr) {
                                let ts = timestamp(time::SystemTime::now());
                                peer.set_last_hb(ts);
                                debug!("PONG message received from: {addr}");
                            }
                        }
                        msg => {
                            warn!("unknown message received from peer: {msg:?}")
                        }
                    };
                }
            }
        });

        // spawn outgoing peer connections
        for addr in known_peers {
            match TcpStream::connect(addr) {
                Ok(stream) => {
                    let (reader, writer) = split_stream(stream);

                    // create new peer
                    let mut peer = TcpPeer::new(
                        addr,
                        self.node_addr,
                        PeerStreamDirection::Outgoing,
                        reader,
                        writer,
                        self.peer_msg_tx.clone(),
                    );

                    // start incoming message handler
                    peer.spawn_incoming_handler();

                    // add new peer to self peer set
                    self.peers.lock().unwrap().insert(addr, peer);
                }
                Err(e) => {
                    error!("{e}")
                }
            }
        }
    }

    pub fn get_peer_addrs(&self) -> Vec<SocketAddr> {
        self.peers.lock().unwrap().keys().cloned().collect()
    }

    // pub fn send_rpc(&self, addr: SocketAddr, rpc: RPC) {
    pub fn send_rpc(&self, rpc: RPC) {
        for (_, peer) in self.peers.lock().as_mut().unwrap().iter_mut() {
            // if let Some(peer) = self.peers.lock().unwrap().get_(&addr) {
            let msg = PeerMessage::RPC(self.node_addr, rpc.to_bytes());
            peer.send_msg(&msg);
            break;
        }
    }

    pub fn broadcast(&self, msg: &PeerMessage) {
        for (_, peer) in self.peers.lock().as_mut().unwrap().iter_mut() {
            peer.send_msg(msg);
        }
    }
}

type ThreadBufReader = ArcMut<BufReader<TcpStream>>;
type ThreadBufWriter = ArcMut<BufWriter<TcpStream>>;

pub fn split_stream(stream: TcpStream) -> (ThreadBufReader, ThreadBufWriter) {
    let input = BufReader::new(stream.try_clone().unwrap());
    let output = BufWriter::new(stream);
    (ArcMut::new(input), ArcMut::new(output))
}
