extern crate tokio;

use tokio::io::{self, AsyncWriteExt, AsyncReadExt};
use tokio::net::TcpStream;
use bytes::{BytesMut, Buf};

const BUFFER_SIZE: usize = 1 << 16;

#[tokio::main]
async fn main() {
    let stream = TcpStream::connect("127.0.0.1:4242")
        .await
        .expect("cannot connect");

    let (mut reader, mut writer) = io::split(stream);

    let write_thr = tokio::spawn(async move {
        let mut buffer = BytesMut::with_capacity(BUFFER_SIZE);
        let mut stdin = io::stdin();
        loop {
            match stdin.read_buf(&mut buffer).await {
                Ok(0) => break,
                Ok(_) => {
                    while buffer.has_remaining() {
                        match writer.write_buf(&mut buffer).await {
                            Ok(_) => (),
                            Err(_) => break,
                        }
                    }
                },
                Err(_) => break,
            }
        }
        writer.shutdown().await.ok();
    });
    let read_thr = tokio::spawn(async move {
        let mut buffer = BytesMut::with_capacity(BUFFER_SIZE);
        let mut stdout = io::stdout();
        loop {
            match reader.read_buf(&mut buffer).await {
                Ok(0) => break,
                Ok(_) => {
                    while buffer.has_remaining() {
                        match stdout.write_buf(&mut buffer).await {
                            Ok(_) => (),
                            Err(_) => break,
                        }
                    }
                },
                Err(_) => break,
            }
        }

        stdout.shutdown().await.ok();
    });

    read_thr.await.ok();
    write_thr.await.ok();
}
