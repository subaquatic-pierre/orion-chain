mod network;

use std::{error::Error, sync::Arc, thread, time};

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

fn main() -> Result<(), Box<dyn Error>> {
    let ts1 = LocalTransport::new("local");
    let ts2 = LocalTransport::new("custom");
    let ts3 = LocalTransport::new("remote");
    let mut ts_manager = TransportManager::new();

    ts_manager.connect(ts1)?;
    ts_manager.connect(ts2)?;
    ts_manager.connect(ts3)?;

    let config = ServerConfig { ts_manager };
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

                ts_manager.send_msg(addr, "remote".to_string(), vec![]);
                // .expect("unable to send message");
            }
            thread::sleep(time::Duration::from_secs(3));

            if let Ok(ticker) = ticker.lock().as_mut() {
                ticker.inc()
            }
        }
    });

    handle.join().unwrap();

    Ok(())
}
