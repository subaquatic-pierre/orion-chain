use std::net::SocketAddr;
use std::sync::{Arc, Mutex as StdMutex};

use std::thread;
use std::time;

use bytes::Bytes;
use hyper::server::conn::http1;
use hyper::service::service_fn;
use log::{error, info, warn};
use tokio::net::TcpListener;
use tokio::sync::Mutex;

use crate::rpc::controller::RpcController;

use super::types::ArcRcpHandler;

use super::tokio_util::TokioIo;

pub type GenericError = Box<dyn std::error::Error + Send + Sync>;
pub type Result<T> = std::result::Result<T, GenericError>;
pub type BoxBody = http_body_util::combinators::BoxBody<Bytes, hyper::Error>;

// pub static INDEX: &[u8] = b"<a href=\"test.html\">test.html</a>";
// pub static INTERNAL_SERVER_ERROR: &[u8] = b"Internal Server Error";
// pub static NOTFOUND: &[u8] = b"Not Found";
// pub static POST_DATA: &str = r#"{"original": "data"}"#;
// pub static URL: &str = "http://127.0.0.1:1337/json_api";

use super::router::HttpRouter;

pub struct ApiServer {
    // router: Arc<Mutex<HttpRouter>>,
    router: Arc<Mutex<HttpRouter>>,
}

impl ApiServer {
    pub fn new(rpc_handler: Arc<RpcController>) -> Self {
        let router = Arc::new(Mutex::new(HttpRouter::new(rpc_handler)));

        Self { router }
    }

    pub async fn start(&self) -> Result<()> {
        let addr: SocketAddr = "127.0.0.1:1337".parse().unwrap();

        let listener = TcpListener::bind(&addr).await?;
        info!("client RPC server listening on http://{}", addr);

        loop {
            let (stream, _) = listener.accept().await?;
            let io = TokioIo::new(stream);

            let router = self.router.clone();

            tokio::task::spawn(async move {
                let router = router.lock().await;
                let service = service_fn(|req| router.route_handler(req));

                if let Err(err) = http1::Builder::new().serve_connection(io, service).await {
                    error!("failed to serve connection: {:?}", err);
                }
            });
        }
    }
}
