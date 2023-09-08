use crate::network::error::NetworkError;
use std::net::SocketAddr;

#[derive(Debug)]
pub enum PeerMessage {
    RPC(SocketAddr, Vec<u8>),
    Error(SocketAddr, String),
    Disconnect(SocketAddr, String),
    Ping(SocketAddr, Vec<u8>),
    Pong(SocketAddr, Vec<u8>),
}

#[derive(Debug, Copy, Clone)]
#[repr(u8)]
pub enum MessageCodeMap {
    RPC = 1,
    Ping = 100,
    Pong = 101,
    Error = 200,
    Disconnect = 201,
    Unknown = 255,
}

impl From<u8> for MessageCodeMap {
    fn from(value: u8) -> Self {
        unsafe { ::std::mem::transmute(value) }
    }
}

impl From<MessageCodeMap> for u8 {
    fn from(value: MessageCodeMap) -> u8 {
        value as u8
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
        let code: MessageCodeMap = first_byte.into();
        // let code: MessageCodeMap = MessageCodeMap::from_byte(first_byte);

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
                buf.extend_from_slice(&[MessageCodeMap::Disconnect.into()]);
                buf.extend_from_slice(msg.as_bytes());
                buf
            }
            Self::Error(_, msg) => {
                buf.extend_from_slice(&[MessageCodeMap::Error.into()]);
                buf.extend_from_slice(msg.as_bytes());
                buf
            }
            Self::Ping(_, msg) => {
                buf.extend_from_slice(&[MessageCodeMap::Ping.into()]);
                buf.extend_from_slice(msg);
                buf
            }
            Self::Pong(_, msg) => {
                buf.extend_from_slice(&[MessageCodeMap::Pong.into()]);
                buf.extend_from_slice(msg);
                buf
            }
            Self::RPC(_, msg) => {
                buf.extend_from_slice(&[MessageCodeMap::RPC.into()]);
                buf.extend_from_slice(msg);
                buf
            }
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use std::net::{IpAddr, Ipv4Addr};
    #[test]
    fn test_message_code() {
        let code = MessageCodeMap::RPC;

        let num: u8 = code.into();

        assert_eq!(num, 1);

        let num = 255_u8;

        let val = MessageCodeMap::from(num);
        assert_eq!(format!("{:?}", MessageCodeMap::Unknown), format!("{val:?}"));

        let num = 100_u8;

        let val = MessageCodeMap::from(num);
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
