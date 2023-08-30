#![allow(clippy::needless_range_loop)]
use crypto::private_key::PrivateKey;
use log::{info, trace, warn};
use rand::Rng;

mod core;
mod crypto;
mod network;

use std::{
    error::Error,
    sync::{Arc, Once},
    thread, time,
};

use network::{
    server::{Server, ServerConfig},
    transport::{ArcMut, LocalTransport, Transport, TransportManager},
};

use crate::network::error::NetworkError;

struct Ticker {
    pub val: i32,
}

impl Ticker {
    pub fn inc(&mut self) {
        self.val += 1
    }
}

static INIT: Once = Once::new();

/// Setup function that is only run once, even if called multiple times.
fn setup() {
    INIT.call_once(|| {
        env_logger::init();
    });
}

fn main() -> Result<(), Box<dyn Error>> {
    setup();
    let ts1 = LocalTransport::new("local");
    let ts2 = LocalTransport::new("custom");
    let ts3 = LocalTransport::new("remote");
    let mut ts_manager = TransportManager::new();

    ts_manager.connect(ts1)?;
    ts_manager.connect(ts2)?;
    ts_manager.connect(ts3)?;

    let config = ServerConfig {
        ts_manager,
        block_time: time::Duration::from_secs(20),
        private_key: Some(PrivateKey::new()),
    };

    let mut server = Server::new(config);

    let ticker = ArcMut::new(Ticker { val: 0 });

    let handle = thread::spawn(move || {
        server.start();
        loop {
            if let Ok(ts_manager) = server.transport_manager.lock() {
                let addr = match ticker.lock() {
                    Ok(tick) => {
                        if tick.val % 4 == 0 {
                            "local".to_string()
                        } else if tick.val % 3 == 1 {
                            "remote".to_string()
                        } else if tick.val % 3 == 2 {
                            "custom".to_string()
                        } else {
                            "street".to_string()
                        }
                    }
                    _ => "local".to_string(),
                };

                let random_number: Vec<u8> = (0..1024).map(|_| rand::random::<u8>()).collect();
                ts_manager
                    .send_msg(addr, "remote".to_string(), random_number)
                    .ok();
            }
            thread::sleep(time::Duration::from_secs(1));

            if let Ok(ticker) = ticker.lock().as_mut() {
                ticker.inc()
            }
        }
    });

    handle.join().unwrap();

    Ok(())
}
