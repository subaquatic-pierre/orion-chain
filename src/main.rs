use std::error::Error;

use orion_chain::{
    build_full_node,
    crypto::private_key::PrivateKey,
    logger_init,
    network::{
        node::{ChainNode, NodeConfig},
        transport::{ArcMut, LocalTransport, Transport, TransportManager},
    },
    send_tx_loop,
};

fn main() -> Result<(), Box<dyn Error>> {
    logger_init();

    let server = build_full_node()?;

    let handle = send_tx_loop(server);
    handle.join().unwrap();

    Ok(())
}
