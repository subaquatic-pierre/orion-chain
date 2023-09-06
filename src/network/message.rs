use crate::network::error::NetworkError;
use std::borrow::{BorrowMut, Cow};
use std::net::{IpAddr, Ipv4Addr, Ipv6Addr, SocketAddr, TcpListener, TcpStream};

use super::types::{ArcMut, NetAddr, Payload};

#[derive(Debug)]
pub enum PeerMessage {
    RPC(SocketAddr, Vec<u8>),
    Error(SocketAddr, String),
    Disconnect(SocketAddr, String),
    Ping(SocketAddr, Vec<u8>),
    Pong(SocketAddr, Vec<u8>),
}

#[derive(Debug)]
pub enum MessageCodeMap {
    RPC,
    Error,
    Disconnect,
    Ping,
    Pong,
    Unknown,
}

impl MessageCodeMap {
    fn from_byte(value: u8) -> Self {
        match value {
            1 => MessageCodeMap::RPC,
            200 => MessageCodeMap::Error,
            201 => MessageCodeMap::Disconnect,
            100 => MessageCodeMap::Ping,
            101 => MessageCodeMap::Pong,
            _ => MessageCodeMap::Unknown,
        }
    }

    fn to_byte(self) -> u8 {
        match self {
            MessageCodeMap::RPC => 1,
            MessageCodeMap::Error => 200,
            MessageCodeMap::Disconnect => 201,
            MessageCodeMap::Ping => 100,
            MessageCodeMap::Pong => 101,
            MessageCodeMap::Unknown => 255,
        }
    }
}

impl PeerMessage {
    pub fn from_payload(addr: SocketAddr, data: &[u8]) -> Result<Self, NetworkError> {
        let first_byte = match data.first() {
            Some(byte) => *byte,
            None => {
                return Err(NetworkError::Decoding(
                    "unable to get first byte from peer message decoding.".to_string(),
                ))
            }
        };

        // simple helper to change to string
        let data_str = String::from_utf8_lossy(data).to_string();
        let drop_first_byte = data[1..].to_vec();

        // get message code from first byte
        let code: MessageCodeMap = MessageCodeMap::from_byte(first_byte);

        // get message type from code
        let val = match code {
            MessageCodeMap::RPC => PeerMessage::RPC(addr, drop_first_byte),
            MessageCodeMap::Error => PeerMessage::Error(addr, data_str),
            MessageCodeMap::Disconnect => PeerMessage::Disconnect(addr, data_str),
            MessageCodeMap::Ping => PeerMessage::Ping(addr, drop_first_byte),
            MessageCodeMap::Pong => PeerMessage::Pong(addr, drop_first_byte),
            MessageCodeMap::Unknown => PeerMessage::Error(addr, data_str),
        };

        Ok(val)
    }

    pub fn payload(&self) -> Vec<u8> {
        let mut buf = vec![];
        match self {
            Self::Disconnect(_, msg) => {
                buf.extend_from_slice(&[MessageCodeMap::Disconnect.to_byte()]);
                buf.extend_from_slice(msg.as_bytes());
                buf
            }
            Self::Error(_, msg) => {
                buf.extend_from_slice(&[MessageCodeMap::Error.to_byte()]);
                buf.extend_from_slice(msg.as_bytes());
                buf
            }
            Self::Ping(_, msg) => {
                buf.extend_from_slice(&[MessageCodeMap::Ping.to_byte()]);
                buf.extend_from_slice(msg);
                buf
            }
            Self::Pong(_, msg) => {
                buf.extend_from_slice(&[MessageCodeMap::Pong.to_byte()]);
                buf.extend_from_slice(msg);
                buf
            }
            Self::RPC(_, msg) => {
                buf.extend_from_slice(&[MessageCodeMap::RPC.to_byte()]);
                buf.extend_from_slice(msg);
                buf
            }
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    #[test]
    fn test_message_code() {
        let code = MessageCodeMap::RPC;

        let num = code.to_byte();

        assert_eq!(num, 1);

        let num = 255_u8;

        let val = MessageCodeMap::from_byte(num);
        assert_eq!(format!("{:?}", MessageCodeMap::Unknown), format!("{val:?}"));

        let num = 100_u8;

        let val = MessageCodeMap::from_byte(num);
        assert_eq!(format!("{:?}", MessageCodeMap::Ping), format!("{val:?}"));
    }

    #[test]
    fn test_message() {
        let ip = IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1));
        let addr = SocketAddr::new(ip, 5000);
        let payload = b"Hello world";
        let message = PeerMessage::RPC(addr, payload.to_vec());

        let res = [1, 72, 101, 108, 108, 111, 32, 119, 111, 114, 108, 100];

        assert_eq!(format!("{:?}", res), format!("{:?}", message.payload()));

        let decoded = PeerMessage::from_payload(addr, &res).unwrap();

        assert_eq!(format!("{:?}", message), format!("{:?}", decoded));
    }
}
