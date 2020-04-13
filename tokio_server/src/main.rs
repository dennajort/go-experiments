extern crate log;
extern crate pretty_env_logger;
extern crate tokio;

use std::collections::HashMap;
use std::error::Error;
use std::net::SocketAddr;
use std::sync::Arc;

use log::info;

use tokio::io::{self, AsyncRead, AsyncWrite};
use tokio::net::TcpListener;
use tokio::prelude::*;
use tokio::sync::mpsc::{self, Receiver, Sender};

const BUFFER_SIZE: usize = 1 << 16;

enum Event {
    AddClient(SocketAddr, Sender<Arc<Vec<u8>>>),
    Broadcast(Arc<Vec<u8>>),
    RmClient(SocketAddr),
}

async fn handle_events(rx: &mut Receiver<Event>) {
    let mut clients = HashMap::new();
    while let Some(ev) = rx.recv().await {
        match ev {
            Event::AddClient(addr, tx) => {
                clients.insert(addr, tx);
                info!("Client connected, currently {}", clients.len())
            }
            Event::RmClient(addr) => {
                if let Some(_) = clients.remove(&addr) {
                    info!("Client disconnected, remaining {}", clients.len())
                }
            }
            Event::Broadcast(msg) => {
                for tx in clients.values_mut() {
                    tx.send(msg.clone()).await.ok();
                }
            }
        }
    }
}

async fn handle_reader<R>(addr: SocketAddr, reader: &mut R, tx: &mut Sender<Event>)
where
    R: AsyncRead + Unpin,
{
    let mut buf = [0; BUFFER_SIZE];
    loop {
        match reader.read(&mut buf).await {
            Ok(0) => break,
            Ok(n) => match tx
                .send(Event::Broadcast(Arc::new(buf[0..n].to_vec())))
                .await
            {
                Ok(_) => (),
                Err(_) => break,
            },
            Err(_) => break,
        }
    }

    tx.send(Event::RmClient(addr)).await.ok();
}

async fn handle_writer<W>(
    addr: SocketAddr,
    writer: &mut W,
    tx: &mut Sender<Event>,
    rx: &mut Receiver<Arc<Vec<u8>>>,
) where
    W: AsyncWrite + Unpin,
{
    while let Some(msg) = rx.recv().await {
        match writer.write_all(msg.as_slice()).await {
            Ok(_) => (),
            Err(_) => break,
        }
    }
    rx.close();

    tx.send(Event::RmClient(addr)).await.ok();
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    pretty_env_logger::init();
    let mut listener = TcpListener::bind("127.0.0.1:4242").await?;
    let (mut tx, mut rx) = mpsc::channel(1);
    tokio::spawn(async move { handle_events(&mut rx).await });
    loop {
        let (stream, addr) = listener.accept().await?;
        let (mut reader, mut writer) = io::split(stream);
        let (tx2, mut rx2) = mpsc::channel(1);

        tx.send(Event::AddClient(addr, tx2)).await.ok();

        {
            let mut tx = tx.clone();
            tokio::spawn(async move { handle_writer(addr, &mut writer, &mut tx, &mut rx2).await });
        }

        {
            let mut tx = tx.clone();
            tokio::spawn(async move { handle_reader(addr, &mut reader, &mut tx).await });
        }
    }
}
