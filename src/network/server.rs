use core::time;
use std::{
    sync::{
        mpsc::{channel, Receiver, Sender},
        Arc,
    },
    thread,
};

use super::transport::{ArcMut, HttpTransport, LocalTransport, Transport, TransportManager, RPC};

pub struct ServerConfig<T>
where
    T: Transport,
{
    pub ts_manager: TransportManager<T>,
}

pub struct Server<T>
where
    T: Transport,
{
    pub transport_manager: ArcMut<TransportManager<T>>,
    rx: ArcMut<Receiver<RPC>>,
    tx: ArcMut<Sender<RPC>>,
}

impl Server<LocalTransport> {
    pub fn new(config: ServerConfig<LocalTransport>) -> Self {
        let (tx, rx) = channel::<RPC>();
        let (tx, rx) = (ArcMut::new(tx), ArcMut::new(rx));
        let ts_manager = ArcMut::new(config.ts_manager);

        Self {
            transport_manager: ts_manager,
            rx,
            tx,
        }
    }

    pub fn start(&mut self) {
        let ts_manager = self.transport_manager.clone();
        let tx = self.tx.clone();
        let rx = self.rx.clone();

        if let Ok(ts_manager) = ts_manager.lock().as_mut() {
            ts_manager
                .init(tx)
                .expect("unable to initialize transport manager");
        }

        // Start infinite loop to handler transport manager messages
        thread::spawn(move || loop {
            if let Ok(rx) = rx.lock() {
                for msg in rx.try_iter() {
                    println!("{msg:#?}")
                }
            }

            thread::sleep(time::Duration::from_secs(1));
        });
    }
}

// impl Server<HttpTransport> {
//     pub fn new(config: ServerConfig<HttpTransport>) -> Self {
//         let (tx, rx) = channel::<RPC>();
//         let (tx, rx) = (ArcMut::new(tx), ArcMut::new(rx));
//         let ts_manager = Arc::new(config.ts_manager);

//         Self {
//             transport_manager: ts_manager,
//             rx,
//             tx,
//         }
//     }

//     pub fn start(&self) {
//         let ts_manager = self.transport_manager.clone();
//         let tx = self.tx.clone();
//         thread::spawn(|| loop {
//             println!("something");

//             thread::sleep(time::Duration::from_secs(5));
//         });
//     }
// }
