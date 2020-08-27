#[macro_use]
extern crate log;
extern crate env_logger;
extern crate tokio;

use std::collections::HashMap;
use std::error::Error;
use std::net::SocketAddr;
use std::sync::Arc;

use bytes::{Buf, Bytes, BytesMut};
use tokio::net::TcpListener;
use tokio::prelude::*;
use tokio::sync::mpsc::{self, Receiver, Sender};
use tokio::sync::Mutex;

const BUFFER_SIZE: usize = 1 << 16;

struct Server {
    clients: Mutex<HashMap<SocketAddr, Sender<Bytes>>>,
}

impl Server {
    pub fn new() -> Server {
        Server {
            clients: Mutex::new(HashMap::new()),
        }
    }

    pub async fn broadcast(&self, msg: Bytes) {
        let mut clients = self.clients.lock().await;
        for tx in clients.values_mut() {
            tx.send(msg.clone()).await.ok();
        }
    }

    pub async fn add_client(&self, addr: SocketAddr, tx: Sender<Bytes>) {
        let mut clients = self.clients.lock().await;
        clients.insert(addr, tx);
        info!("Client connected, currently {}", clients.len());
    }

    pub async fn remove_client(&self, addr: SocketAddr) {
        let mut clients = self.clients.lock().await;
        if let Some(_) = clients.remove(&addr) {
            info!("Client disconnected, remaining {}", clients.len());
        }
    }
}

async fn handle_reader<R>(reader: &mut R, server: Arc<Server>)
where
    R: AsyncRead + Unpin,
{
    let mut buf = BytesMut::with_capacity(BUFFER_SIZE);
    loop {
        match reader.read_buf(&mut buf).await {
            Ok(0) => {
                info!("connection closed");
                break;
            }
            Ok(n) => {
                debug!("read {} bytes", n);
                server.broadcast(buf.split().freeze()).await;
            }
            Err(e) => {
                error!("reading {:?}", e);
                break;
            }
        }
    }
}

async fn handle_writer<W>(
    addr: SocketAddr,
    writer: &mut W,
    server: Arc<Server>,
    rx: &mut Receiver<Bytes>,
) where
    W: AsyncWrite + Unpin,
{
    while let Some(mut msg) = rx.recv().await {
        while msg.has_remaining() {
            match writer.write_buf(&mut msg).await {
                Ok(_) => (),
                Err(e) => {
                    error!("writing {:?}", e);
                    break;
                }
            }
        }
    }
    rx.close();
    server.remove_client(addr).await;
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    env_logger::init();
    let server = Arc::new(Server::new());
    let mut listener = TcpListener::bind("127.0.0.1:4242").await?;
    loop {
        let (stream, addr) = listener.accept().await?;
        let (mut reader, mut writer) = io::split(stream);
        let (tx, mut rx) = mpsc::channel(1);
        server.add_client(addr, tx).await;

        {
            let server = server.clone();
            tokio::spawn(async move { handle_writer(addr, &mut writer, server, &mut rx).await });
        }

        {
            let server = server.clone();
            tokio::spawn(async move { handle_reader(&mut reader, server).await });
        }
    }
}
