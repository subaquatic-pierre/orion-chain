use std::error::Error;

use std::net::SocketAddr;
use std::sync::Arc;

use bytes::{Buf, Bytes};
use http_body_util::{BodyExt, Full};
use hyper::server::conn::{http1, http2};
use hyper::service::service_fn;
use hyper::{body::Incoming as IncomingBody, header, Method, Request, Response, StatusCode};
use orion_chain::api::router::Router;
use tokio::net::{TcpListener, TcpStream};

use orion_chain::{
    api::util::TokioIo,
    build_full_node,
    crypto::private_key::PrivateKey,
    logger_init,
    network::{
        node::{ChainNode, NodeConfig},
        transport::{ArcMut, LocalTransport, Transport, TransportManager},
    },
    send_tx_loop, Result,
};
use tokio::sync::Mutex;

// fn main() -> Result<(), Box<dyn Error>> {
//     logger_init();

//     let server = build_full_node()?;

//     let handle = send_tx_loop(server);
//     handle.join().unwrap();

//     Ok(())
// }

#[tokio::main]
async fn main() -> Result<()> {
    pretty_env_logger::init();

    let mut chain_node = build_full_node()?;
    chain_node.start();

    let arc = Arc::new(Mutex::new(chain_node));

    let router = Arc::new(Router::new(arc.clone()));

    let addr: SocketAddr = "127.0.0.1:1337".parse().unwrap();

    let listener = TcpListener::bind(&addr).await?;
    println!("Listening on http://{}", addr);
    loop {
        let (stream, _) = listener.accept().await?;
        let io = TokioIo::new(stream);

        let router = router.clone();

        tokio::task::spawn(async move {
            let service = service_fn(|req| router.route_handler(req));

            if let Err(err) = http1::Builder::new().serve_connection(io, service).await {
                println!("Failed to serve connection: {:?}", err);
            }
        });
    }
}
