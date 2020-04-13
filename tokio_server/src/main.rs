extern crate log;
extern crate pretty_env_logger;
extern crate tokio;

use std::collections::HashMap;
use std::error::Error;
use std::net::SocketAddr;
use std::sync::Arc;

use log::info;

use tokio::io::{self, AsyncRead, AsyncWrite, BufReader, BufWriter};
use tokio::net::TcpListener;
use tokio::prelude::*;
use tokio::sync::Mutex;

const BUFFER_SIZE: usize = 1 << 16;

struct State<W> {
    clients: HashMap<SocketAddr, W>,
}

impl<W> State<W>
where
    W: AsyncWrite + Unpin,
{
    pub fn new() -> Self {
        Self {
            clients: HashMap::new(),
        }
    }

    fn add_client(&mut self, addr: SocketAddr, writer: W) {
        self.clients.insert(addr, writer);
        info!("Client connected, currently {}", self.clients.len())
    }

    async fn remove_client(&mut self, addr: SocketAddr) {
        if let Some(mut writer) = self.clients.remove(&addr) {
            writer.flush().await.ok();
        }
        info!("Client disconnected, remaining {}", self.clients.len())
    }

    async fn broadcast(&mut self, msg: &[u8]) {
        let mut deads = Vec::new();

        for (addr, writer) in self.clients.iter_mut() {
            match writer.write_all(msg).await {
                Ok(_) => (),
                Err(_) => {
                    deads.push(addr.clone());
                }
            }
        }

        for dead in deads {
            self.remove_client(dead).await;
        }
    }
}

async fn handle_reader<R, W>(addr: SocketAddr, reader: &mut R, state: Arc<Mutex<State<W>>>)
where
    R: AsyncRead + Unpin,
    W: AsyncWrite + Unpin,
{
    let mut buf = [0; BUFFER_SIZE];
    loop {
        match reader.read(&mut buf).await {
            Ok(0) => break,
            Ok(n) => {
                state.lock().await.broadcast(&buf[0..n]).await;
            }
            Err(_) => break,
        }
    }
    state.lock().await.remove_client(addr).await;
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    pretty_env_logger::init();
    let state = Arc::new(Mutex::new(State::new()));
    let mut listener = TcpListener::bind("127.0.0.1:4242").await?;

    loop {
        let (stream, addr) = listener.accept().await?;

        let (reader, writer) = io::split(stream);
        let mut reader = BufReader::with_capacity(BUFFER_SIZE, reader);
        // let writer = BufWriter::new(writer);

        state.lock().await.add_client(addr, writer);

        {
            let state = state.clone();
            tokio::spawn(async move { handle_reader(addr, &mut reader, state).await });
        }
    }
}
