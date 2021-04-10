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
        'wLoop: loop {
            match stdin.read_buf(&mut buffer).await {
                Ok(0) => break 'wLoop,
                Ok(_) => match writer.write_all(&mut buffer).await {
                    Ok(_) => buffer.clear(),
                    Err(_) => break 'wLoop,
                },
                Err(_) => break 'wLoop,
            }
        }
        writer.shutdown().await.ok();
    });

    let read_thr = tokio::spawn(async move {
        let mut buffer = BytesMut::with_capacity(BUFFER_SIZE);
        let mut stdout = io::stdout();
        'rLoop: loop {
            match reader.read_buf(&mut buffer).await {
                Ok(0) => break 'rLoop,
                Ok(_) => match stdout.write_all(&mut buffer).await {
                    Ok(_) => buffer.clear(),
                    Err(_) => break 'rLoop,
                },
                Err(_) => break 'rLoop,
            }
        }

        stdout.shutdown().await.ok();
    });

    read_thr.await.ok();
    write_thr.await.ok();
}
