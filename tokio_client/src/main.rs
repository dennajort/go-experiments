extern crate tokio;

use tokio::io::{self, AsyncWriteExt, BufReader, BufWriter};
use tokio::net::TcpStream;

#[tokio::main]
async fn main() {
    let stream = TcpStream::connect("127.0.0.1:4242")
        .await
        .expect("cannot connect");

    let (reader, writer) = io::split(stream);

    let write_thr = tokio::spawn(async move {
        let mut stdin = BufReader::new(io::stdin());
        let mut writer = BufWriter::new(writer);
        io::copy(&mut stdin, &mut writer).await.ok();
        writer.shutdown().await.ok();
    });
    let read_thr = tokio::spawn(async move {
        let mut stdout = BufWriter::new(io::stdout());
        let mut reader = BufReader::new(reader);
        io::copy(&mut reader, &mut stdout).await.ok();
        stdout.shutdown().await.ok();
    });

    read_thr.await.ok();
    write_thr.await.ok();
}
