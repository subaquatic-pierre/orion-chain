use std::time;

use orion_chain::api::server::ApiServer;
use orion_chain::core::block::random_block;
use orion_chain::core::blockchain::Blockchain;
use orion_chain::core::header::random_header;
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
    let config = NodeConfig {
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
    let server = ApiServer::new(chain_node);
    server.start().await
}
