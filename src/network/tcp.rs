use log::{debug, error, info, warn};

use std::io::{BufReader, BufWriter};

use crate::core::encoding::{ByteDecoding, ByteEncoding};
use crate::core::util::timestamp;
use crate::lock;
use crate::network::error::NetworkError;
use std::collections::HashMap;
use std::net::{SocketAddr, TcpListener, TcpStream};
use std::sync::mpsc::{channel, Receiver, Sender};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time;

use super::types::RpcChanMsg;
use super::{
    message::PeerMessage,
    peer::{PeerStreamDirection, TcpPeer},
    rpc::RPC,
    types::ArcMut,
};

pub struct TcpController {
    pub node_addr: SocketAddr,
    hb_interval: u64,
    _hb_threshhold: u64,
    listener: ArcMut<TcpListener>,
    peers: ArcMut<HashMap<SocketAddr, TcpPeer>>,

    // channel used to send messages to ChainNode
    rpc_tx: Arc<Mutex<Sender<RpcChanMsg>>>,

    // channel used to communicate with peer
    peer_msg_rx: ArcMut<Receiver<PeerMessage>>,
    peer_msg_tx: ArcMut<Sender<PeerMessage>>,
}

impl TcpController {
    pub fn new(
        node_addr: SocketAddr,
        rpc_tx: Arc<Mutex<Sender<RpcChanMsg>>>,
    ) -> Result<Self, NetworkError> {
        let listener = match TcpListener::bind(node_addr) {
            Ok(listener) => listener,
            Err(e) => return Err(NetworkError::Connect(e.to_string())),
        };

        // create channels to be used to communicate with remote peers
        let (tx, rx) = channel::<PeerMessage>();
        let (peer_msg_tx, peer_msg_rx) = (ArcMut::new(tx), ArcMut::new(rx));

        Ok(Self {
            node_addr,
            listener: ArcMut::new(listener),
            peers: ArcMut::new(HashMap::new()),
            rpc_tx,
            peer_msg_rx,
            peer_msg_tx,

            // TODO: CONFIG, get heartbeat interval from config, get heartbeat threshhold
            // from config
            hb_interval: 5,
            _hb_threshhold: 600,
        })
    }

    // Main method used to start TcpController
    // calls private methods to initialize each phase
    pub fn start(&mut self, known_peers: Vec<SocketAddr>) {
        self.init_message_receiver();
        self.init_outgoing_peers(known_peers);
        self.init_heartbeats();
        self.init_listener();
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

    // ---
    // Private Methods
    // ---

    // Spawn thread to handle all incoming messages from
    // peers
    fn init_message_receiver(&self) {
        // get data to be used in thread below
        let _node_addr = self.node_addr;
        let peers = self.peers.clone();
        let rpc_tx = self.rpc_tx.clone();
        let peer_msg_rx = self.peer_msg_rx.clone();

        // spawn main thread to handle messages from peers
        thread::spawn(move || {
            if let Ok(peer_msg_rx) = peer_msg_rx.lock() {
                for msg in peer_msg_rx.iter() {
                    match msg {
                        PeerMessage::Disconnect(addr, _msg) => {
                            info!(
                                "disconnect message received, removing peer from peer list {addr}"
                            );
                            peers.lock().unwrap().remove(&addr);
                        }
                        PeerMessage::Error(addr, msg) => {
                            warn!("error received from peer: {addr} with message: {msg}");
                            peers.lock().unwrap().remove(&addr);
                        }
                        PeerMessage::RPC(addr, rpc_bytes) => {
                            match RPC::from_bytes(&rpc_bytes) {
                                Ok(rpc) => {
                                    // Send message back to ChainNode
                                    if let Err(e) = lock!(rpc_tx).send((addr, rpc)) {
                                        error!("error sending message on RPC chanel from TCPController: {e}");
                                    };
                                }
                                Err(e) => {
                                    error!("unable to decode RPC from peer message: {e}")
                                }
                            }
                        }
                        PeerMessage::Ping(addr, _) => {
                            // return pong message to peer
                            if let Some(peer) = peers.lock().unwrap().get_mut(&addr) {
                                let ts = timestamp(time::SystemTime::now());
                                peer.set_last_hb(ts);
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
                    };
                }
            }
        });
    }

    // Spawn main Tcp listener thread
    // for peers to connect to
    fn init_listener(&self) {
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
    }

    // Create peer for each know peer, known peers
    // is passed from start method
    fn init_outgoing_peers(&self, known_peers: Vec<SocketAddr>) {
        // spawn outgoing peer connections
        for addr in known_peers {
            match TcpStream::connect(addr) {
                Ok(stream) => {
                    let (reader, writer) = split_stream(stream);

                    // create new peer
                    let mut peer = TcpPeer::new(
                        addr,
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

    // Initialize heartbeat thread to check status
    // of all peers determined by heartbeat interval set
    // on main struct
    fn init_heartbeats(&self) {
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
    }
}

type ThreadBufReader = ArcMut<BufReader<TcpStream>>;
type ThreadBufWriter = ArcMut<BufWriter<TcpStream>>;

pub fn split_stream(stream: TcpStream) -> (ThreadBufReader, ThreadBufWriter) {
    let input = BufReader::new(stream.try_clone().unwrap());
    let output = BufWriter::new(stream);
    (ArcMut::new(input), ArcMut::new(output))
}
