use super::transport::{NetAddr, Payload};

#[derive(Debug, Clone)]
pub struct RPC {
    pub sender: NetAddr,
    pub receiver: NetAddr,
    pub payload: Payload,
}
