use std::path::PathBuf;
use std::{thread, time};

use log::warn;
use orion_chain::api::server::ApiServer;
use orion_chain::core::block::random_block;
use orion_chain::core::blockchain::Blockchain;
use orion_chain::core::encoding::ByteEncoding;
use orion_chain::core::header::random_header;
use orion_chain::core::transaction::random_signed_tx;
use orion_chain::crypto::hash::Hash;
use orion_chain::crypto::utils::random_hash;

use orion_chain::logger_init;
use orion_chain::{
    crypto::private_key::PrivateKey,
    network::node::{ChainNode, NodeConfig},
    Result,
};

#[tokio::main]
async fn main() -> Result<()> {
    logger_init();

    // TODO: Get config from file
    let config = NodeConfig::default();

    // Create a ChainNode with newly created blockchain. ChainNode
    // serves the purpose of composing all blockchain functionality together
    // inter peer communication as well as block syncing, transaction processing
    // loops
    let mut chain_node = ChainNode::new(config);
    chain_node.start()?;

    // Create main entry point for HTTP API server for the node,
    // pass in Arc of ChainNode to access blockchain functionality
    // within the Api
    let server = ApiServer::new(chain_node.rpc_controller());
    server
        .start()
        .await
        .expect("Unable to start server")
        .await?;
    Ok(())
}
