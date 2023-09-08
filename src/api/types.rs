use crate::network::rpc::{RpcHandler, RpcHeader, RPC};
use std::sync::{Arc, Mutex};

pub type ArcRcpHandler = Arc<Mutex<RpcHandler>>;
