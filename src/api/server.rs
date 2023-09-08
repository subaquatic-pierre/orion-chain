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
use crate::network::{transport, types::ArcMut};

pub type GenericError = Box<dyn std::error::Error + Send + Sync>;
pub type Result<T> = std::result::Result<T, GenericError>;
pub type BoxBody = http_body_util::combinators::BoxBody<Bytes, hyper::Error>;

pub static INDEX: &[u8] = b"<a href=\"test.html\">test.html</a>";
pub static INTERNAL_SERVER_ERROR: &[u8] = b"Internal Server Error";
pub static NOTFOUND: &[u8] = b"Not Found";
pub static POST_DATA: &str = r#"{"original": "data"}"#;
pub static URL: &str = "http://127.0.0.1:1337/json_api";

use super::router::HttpRouter;
use super::types::ArcRcpHandler;

pub struct ApiServer {
    node: ChainNode,
    router: Arc<Mutex<HttpRouter>>,
}

impl ApiServer {
    pub fn new(node: ChainNode) -> Self {
        let router = Arc::new(Mutex::new(HttpRouter::new(node.rpc_handler())));

        Self { node, router }
    }

    pub async fn start(&self) -> Result<()> {
        let addr: SocketAddr = "127.0.0.1:1337".parse().unwrap();

        let listener = TcpListener::bind(&addr).await?;
        println!("Listening on http://{}", addr);

        loop {
            let (stream, _) = listener.accept().await?;
            let io = TokioIo::new(stream);

            let router = self.router.clone();

            tokio::task::spawn(async move {
                let router = router.lock().await;
                let service = service_fn(|req| router.route_handler(req));

                if let Err(err) = http1::Builder::new().serve_connection(io, service).await {
                    println!("Failed to serve connection: {:?}", err);
                }
            });
        }
    }
}
