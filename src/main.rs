use std::error::Error;

use std::net::SocketAddr;
use std::sync::Arc;
use std::time;

use bytes::{Buf, Bytes};
use http_body_util::{BodyExt, Full};
use hyper::server::conn::{http1, http2};
use hyper::service::service_fn;
use hyper::{body::Incoming as IncomingBody, header, Method, Request, Response, StatusCode};
use orion_chain::api::server::ApiServer;
use orion_chain::core::block::random_block;
use orion_chain::core::blockchain::Blockchain;
use orion_chain::core::header::random_header;
use orion_chain::crypto::utils::random_hash;
use tokio::net::{TcpListener, TcpStream};

use orion_chain::{
    api::util::TokioIo,
    build_full_node,
    crypto::private_key::PrivateKey,
    logger_init,
    network::{
        node::{ChainNode, NodeConfig},
        transport::{LocalTransport, Transport, TransportManager},
        types::ArcMut,
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

    // TODO: Remove transport manager construction here
    let ts1 = LocalTransport::new("local");
    let ts2 = LocalTransport::new("custom");
    let ts3 = LocalTransport::new("remote");
    let mut ts_manager = TransportManager::new();

    ts_manager.connect(ts1)?;
    ts_manager.connect(ts2)?;
    ts_manager.connect(ts3)?;

    // TODO: Get config from file
    let config = NodeConfig {
        ts_manager,
        block_time: time::Duration::from_secs(30),
        private_key: Some(PrivateKey::new()),
    };

    // Create core blockchain data structure. The data structure
    // does no server any function on its own, it needs
    // to be added to the ChainNode to allow for inter peer communication
    // as well as starting the mining/validation loops needed for
    // a functioning blockchain.
    let block = random_block(random_header(0, random_hash()));
    let chain = Blockchain::new_with_genesis(block);

    // Create a ChainNode with newly created blockchain. ChainNode
    // serves the purpose of composing all blockchain functionality together
    // inter peer communication as well as block syncing, transaction processing
    // loops
    let mut chain_node = ChainNode::new(config, chain);
    chain_node.start()?;

    // Create main entry point for HTTP API server for the node,
    // pass in Arc of ChainNode to access blockchain functionality
    // within the Api

    let rpc_handler = chain_node.rpc_handler();
    // let arc_node = Arc::new(Mutex::new(chain_node));
    let server = ApiServer::new(chain_node);
    server.start().await
}
