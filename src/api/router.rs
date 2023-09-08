use std::net::SocketAddr;
use std::sync::{Arc, Mutex as StdMutex};

use bytes::{Buf, Bytes};
use http_body_util::{BodyExt, Full};
use hyper::server::conn::http1;
use hyper::service::service_fn;
use hyper::{body::Incoming as IncomingBody, header, Method, Request, Response, StatusCode};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::{Mutex, MutexGuard};

use crate::api::util::TokioIo;
use crate::core::blockchain::Blockchain;
use crate::network::node::ChainNode;
use crate::network::rpc::{RpcHandler, RpcHeader, RPC};
use crate::network::{transport, types::ArcMut};

pub type GenericError = Box<dyn std::error::Error + Send + Sync>;
pub type Result<T> = std::result::Result<T, GenericError>;
pub type BoxBody = http_body_util::combinators::BoxBody<Bytes, hyper::Error>;

pub static INDEX: &[u8] = b"<a href=\"test.html\">test.html</a>";
pub static INTERNAL_SERVER_ERROR: &[u8] = b"Internal Server Error";
pub static NOTFOUND: &[u8] = b"Not Found";
pub static POST_DATA: &str = r#"{"original": "data"}"#;
pub static URL: &str = "http://127.0.0.1:1337/json_api";

use super::types::ArcRcpHandler;

use super::handlers::{create_tx, get_block, get_tx, not_found};

pub struct HttpRouter {
    rpc_handler: ArcRcpHandler,
}

impl HttpRouter {
    pub fn new(rpc_handler: ArcRcpHandler) -> Self {
        Self { rpc_handler }
    }

    pub async fn route_handler(&self, req: Request<IncomingBody>) -> Result<Response<BoxBody>> {
        let rpc_handler = &self.rpc_handler.clone();
        // let chain = &self.node.lock().await.chain;
        match (req.method(), req.uri().path()) {
            (&Method::POST, "/create-tx") => create_tx(rpc_handler, req).await,
            (&Method::POST, "/get-tx") => get_tx(rpc_handler, req).await,
            (&Method::POST, "/get-block") => get_block(rpc_handler, req).await,
            _ => not_found().await,
        }
    }
}
