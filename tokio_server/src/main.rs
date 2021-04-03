#[macro_use]
extern crate log;
extern crate env_logger;
extern crate tokio;

use std::collections::{HashMap, LinkedList};
use std::error::Error;
use std::net::SocketAddr;
use std::sync::Arc;

use bytes::{Buf, Bytes, BytesMut};
use tokio::net::TcpListener;
use tokio::sync::mpsc::{self, Receiver, Sender};
use tokio::sync::Mutex;
use tokio::io::{self, AsyncWriteExt, AsyncReadExt};

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
        let mut to_remove = LinkedList::new();
        for (addr, tx) in clients.iter() {
            if let Err(_e) = tx.send(msg.clone()).await{
                // error!("sending to writer {:?}", e);
                to_remove.push_front(addr.clone());
            };
        }

        while let Some(addr) = to_remove.pop_front() {
            clients.remove(&addr);
            info!("Client disconnected, remaining {}", clients.len());
        }
    }

    pub async fn add_client(&self, addr: SocketAddr, tx: Sender<Bytes>) {
        let mut clients = self.clients.lock().await;
        clients.insert(addr, tx);
        info!("Client connected, currently {}", clients.len());
    }
}

async fn handle_reader<R>(reader: &mut R, server: Arc<Server>)
where
    R: AsyncReadExt + Unpin,
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
    writer: &mut W,
    rx: &mut Receiver<Bytes>,
) where
    W: AsyncWriteExt + Unpin,
{
    while let Some(mut msg) = rx.recv().await {
        while msg.has_remaining() {
            if let Err(_e) = writer.write_buf(&mut msg).await{
                // error!("writing {:?}", e);
                return;
            }
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    env_logger::init();
    let server = Arc::new(Server::new());
    let listener = TcpListener::bind("127.0.0.1:4242").await?;
    loop {
        let (stream, addr) = listener.accept().await?;
        let (mut reader, mut writer) = io::split(stream);
        let (tx, mut rx) = mpsc::channel(1);
        server.add_client(addr, tx).await;

        tokio::spawn(async move { handle_writer(&mut writer, &mut rx).await });

        {
            let server = server.clone();
            tokio::spawn(async move { handle_reader(&mut reader, server).await });
        }
    }
}
