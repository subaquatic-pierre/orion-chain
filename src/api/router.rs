use std::sync::Arc;

use bytes::Bytes;
use hyper::{body::Incoming as IncomingBody, Method, Request, Response};

pub type GenericError = Box<dyn std::error::Error + Send + Sync>;
pub type Result<T> = std::result::Result<T, GenericError>;
pub type BoxBody = http_body_util::combinators::BoxBody<Bytes, hyper::Error>;

use crate::rpc::controller::RpcController;

use super::types::ArcRcpHandler;

use super::handlers::{
    get_block, get_block_header, get_chain_height, get_last_block, get_tx, new_tx, not_found,
};

pub struct HttpRouter {
    rpc_controller: Arc<RpcController>,
}

impl HttpRouter {
    pub fn new(rpc_controller: Arc<RpcController>) -> Self {
        Self { rpc_controller }
    }

    pub async fn route_handler(&self, req: Request<IncomingBody>) -> Result<Response<BoxBody>> {
        let rpc_controller = self.rpc_controller.clone();
        // let chain = &self.node.lock().await.chain;
        match (req.method(), req.uri().path()) {
            (&Method::POST, "/get-chain-height") => get_chain_height(rpc_controller, req).await,
            (&Method::POST, "/get-last-block") => get_last_block(rpc_controller, req).await,
            (&Method::POST, "/new-tx") => new_tx(rpc_controller, req).await,
            (&Method::POST, "/get-tx") => get_tx(rpc_controller, req).await,
            (&Method::POST, "/get-block") => get_block(rpc_controller, req).await,
            (&Method::POST, "/get-block-header") => get_block_header(rpc_controller, req).await,
            _ => not_found().await,
        }
    }
}
