use bytes::Bytes;
use hyper::{body::Incoming as IncomingBody, Method, Request, Response};

pub type GenericError = Box<dyn std::error::Error + Send + Sync>;
pub type Result<T> = std::result::Result<T, GenericError>;
pub type BoxBody = http_body_util::combinators::BoxBody<Bytes, hyper::Error>;

use super::types::ArcRcpHandler;

use super::handlers::{
    get_block, get_block_header, get_chain_height, get_last_block, get_tx, new_tx, not_found,
};

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
            (&Method::POST, "/get-chain-height") => get_chain_height(rpc_handler, req).await,
            (&Method::POST, "/get-last-block") => get_last_block(rpc_handler, req).await,
            (&Method::POST, "/new-tx") => new_tx(rpc_handler, req).await,
            (&Method::POST, "/get-tx") => get_tx(rpc_handler, req).await,
            (&Method::POST, "/get-block") => get_block(rpc_handler, req).await,
            (&Method::POST, "/get-block-header") => get_block_header(rpc_handler, req).await,
            _ => not_found().await,
        }
    }
}
