#[macro_use]
extern crate log;
extern crate env_logger;
extern crate tokio;

use std::collections::LinkedList;
use std::error::Error;

use bytes::{Bytes, BytesMut};
use tokio::net::TcpListener;
use tokio::sync::mpsc::{self, Receiver, Sender};
use tokio::io::{self, AsyncWriteExt, AsyncReadExt};

const BUFFER_SIZE: usize = 1 << 16;

#[derive(Debug)]
enum Action {
    Broadcast(Bytes),
    AddClient(Sender<Bytes>),
}

async fn handle_manager(mut rx: Receiver<Action>) {
    let mut clients = LinkedList::new();
    loop {
        match rx.recv().await {
            Some(Action::AddClient(tx)) => {
                clients.push_back(tx);
                info!("Client connected, currently {}", clients.len());
            }
            Some(Action::Broadcast(msg)) => {
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
            None => {
                return;
            }
        }
    }
}

async fn handle_reader<R>(
    reader: &mut R,
    tx: &mut Sender<Action>,
) where
    R: AsyncReadExt + Unpin,
{
    let mut buf = BytesMut::with_capacity(BUFFER_SIZE);
    loop {
        match reader.read_buf(&mut buf).await {
            Ok(0) => {
                info!("connection closed");
                return;
            }
            Ok(n) => {
                debug!("read {} bytes", n);
                if let Err(e) = tx.send(Action::Broadcast(buf.split().freeze())).await {
                    error!("sending to manager {:?}", e);
                    return
                }
            }
            Err(e) => {
                error!("reading {:?}", e);
                return;
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
        if let Err(_e) = writer.write_all(&mut msg).await{
            // error!("writing {:?}", e);
            return;
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    env_logger::init();
    let (stx, srx) = mpsc::channel(1);

    tokio::spawn(async move { handle_manager(srx).await });

    let listener = TcpListener::bind("127.0.0.1:4242").await?;

    loop {
        let (stream, _) = listener.accept().await?;
        let (mut reader, mut writer) = io::split(stream);
        let (tx, mut rx) = mpsc::channel(1);
        stx.send(Action::AddClient(tx)).await?;

        tokio::spawn(async move { handle_writer(&mut writer, &mut rx).await });

        {
            let mut tx = stx.clone();
            tokio::spawn(async move { handle_reader(&mut reader, &mut tx).await });
        }
    }
}
