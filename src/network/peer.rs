use log::{error, info};

use std::io::{BufReader, BufWriter, ErrorKind, Read, Write};

use crate::core::util::timestamp;
use std::net::{SocketAddr, TcpStream};
use std::sync::mpsc::Sender;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time;

use super::{message::PeerMessage, types::ArcMut};

#[derive(Debug)]
pub enum PeerStreamDirection {
    Incoming,
    Outgoing,
}

pub struct TcpPeer {
    reader: ArcMut<BufReader<TcpStream>>,
    writer: ArcMut<BufWriter<TcpStream>>,
    _direction: PeerStreamDirection,
    remote_addr: SocketAddr,
    tcp_controller_tx: Arc<Mutex<Sender<PeerMessage>>>,
    pub last_hb: u64,
}

impl TcpPeer {
    pub fn new(
        remote_addr: SocketAddr,
        direction: PeerStreamDirection,
        reader: ArcMut<BufReader<TcpStream>>,
        writer: ArcMut<BufWriter<TcpStream>>,
        tcp_controller_tx: Arc<Mutex<Sender<PeerMessage>>>,
    ) -> Self {
        let last_hb = timestamp(time::SystemTime::now());
        Self {
            remote_addr,
            reader,
            writer,
            _direction: direction,
            tcp_controller_tx,
            last_hb,
        }
    }

    pub fn spawn_incoming_handler(&mut self) {
        // get handle to incoming stream
        let stream = self.reader.clone();

        // get channel to send back to TCP controller
        let tcp_controller_tx = self.tcp_controller_tx.clone();

        // get information of node to be used in messages
        let remote_addr = self.remote_addr;

        // start thread to listen to reads on stream
        thread::spawn(move || {
            // create buffer to handle incoming bytes
            let mut buf = [0u8; 1024];

            if let Ok(reader) = stream.lock().as_mut() {
                loop {
                    match reader.read(&mut buf) {
                        // successful read
                        Ok(bytes_read) => {
                            // if zero bytes read then connection is terminated
                            if bytes_read == 0 {
                                if let Ok(message_tx) = tcp_controller_tx.lock() {
                                    // send error back to TCP controller
                                    message_tx
                                        .send(PeerMessage::Disconnect(
                                            remote_addr,
                                            "disconnected".to_string(),
                                        ))
                                        .ok();
                                    break;
                                }
                            }

                            // get TCP controller tx channel
                            if let Ok(message_tx) = tcp_controller_tx.lock() {
                                // decode message from payload received
                                // MAIN return of PeerMessage
                                if let Ok(msg) = PeerMessage::from_payload(remote_addr, &buf) {
                                    // try send message back to TCP controller
                                    if let Err(e) = message_tx.send(msg) {
                                        let err = PeerMessage::Error(remote_addr, e.to_string());

                                        // try send back to TCP controller again
                                        message_tx.send(err).ok();
                                    }
                                }

                                // clear buffer
                                buf = [0_u8; 1024];
                            }
                        }

                        // connection reset by remote
                        Err(e) if e.kind() == ErrorKind::ConnectionReset => {
                            if let Ok(tcp_controller_tx) = tcp_controller_tx.lock() {
                                tcp_controller_tx
                                    .send(PeerMessage::Disconnect(
                                        remote_addr,
                                        "disconnected".to_string(),
                                    ))
                                    .ok();
                                break;
                            }
                        }

                        // unknown error
                        Err(e) => {
                            if let Ok(message_tx) = tcp_controller_tx.lock() {
                                let err = PeerMessage::Error(remote_addr, e.to_string());
                                message_tx.send(err).ok();
                                break;
                            }
                        }
                    }
                }
            }
        });
    }

    pub fn send_msg(&mut self, msg: &PeerMessage) {
        let remote_addr = self.remote_addr;
        if let Ok(writer) = self.writer.lock().as_mut() {
            // main method to send messages to remote peers
            // always send payload type as defined in PeerMessage payload
            // the receiver will always decode the message with
            // PeerMessage.from_payload()
            if let Ok(n) = writer.write(&msg.payload()) {
                info!("message sent to: {remote_addr:?}, num bytes: {n}",)
            }

            // flush writer to ensure message is sent
            if let Err(e) = writer.flush() {
                error!("unable to send message to: {remote_addr:?}, error: {e}",)
            }
        }
    }

    pub fn set_last_hb(&mut self, ts: u64) {
        self.last_hb = ts;
    }
}
