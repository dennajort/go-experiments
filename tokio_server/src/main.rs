#[macro_use]
extern crate log;
extern crate env_logger;
extern crate tokio;

use std::collections::LinkedList;
use std::error::Error;
use std::sync::Arc;

use bytes::{Buf, Bytes, BytesMut};
use tokio::net::TcpListener;
use tokio::sync::mpsc::{self, Receiver, Sender};
use tokio::sync::Mutex;
use tokio::io::{self, AsyncWriteExt, AsyncReadExt};

const BUFFER_SIZE: usize = 1 << 16;

struct Server {
    clients: Mutex<LinkedList<Sender<Bytes>>>,
}

impl Server {
    pub fn new() -> Server {
        Server {
            clients: Mutex::new(LinkedList::new()),
        }
    }

    pub async fn broadcast(&self, msg: Bytes) {
        let mut clients = self.clients.lock().await;
        let count = clients.len();
        for _ in 0..count {
            if let Some(tx) = clients.pop_front() {
                match tx.send(msg.clone()).await {
                    Ok(_) => {
                        clients.push_back(tx);
                    }
                    Err(_) => {
                        info!("Client disconnected, remaining {}", clients.len());
                    }
                }
            }
        }
    }

    pub async fn add_client(&self, tx: Sender<Bytes>) {
        let mut clients = self.clients.lock().await;
        clients.push_back(tx);
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
        let (stream, _) = listener.accept().await?;
        let (mut reader, mut writer) = io::split(stream);
        let (tx, mut rx) = mpsc::channel(1);
        server.add_client(tx).await;

        tokio::spawn(async move { handle_writer(&mut writer, &mut rx).await });

        {
            let server = server.clone();
            tokio::spawn(async move { handle_reader(&mut reader, server).await });
        }
    }
}
