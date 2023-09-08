use bytes::Bytes;
use hyper::{body::Incoming as IncomingBody, Method, Request, Response};

pub type GenericError = Box<dyn std::error::Error + Send + Sync>;
pub type Result<T> = std::result::Result<T, GenericError>;
pub type BoxBody = http_body_util::combinators::BoxBody<Bytes, hyper::Error>;

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
