use crate::network::rpc::RpcHandler;
use bytes::Bytes;
use std::sync::{Arc, Mutex};

pub type GenericError = Box<dyn std::error::Error + Send + Sync>;
pub type Result<T> = std::result::Result<T, GenericError>;
pub type BoxBody = http_body_util::combinators::BoxBody<Bytes, hyper::Error>;

pub type ArcRcpHandler = Arc<Mutex<RpcHandler>>;
