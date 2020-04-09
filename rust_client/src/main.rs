use std::io;
use std::io::prelude::*;
use std::net::{Shutdown, TcpStream};
use std::thread;

const BUFFER_SIZE: usize = 1 << 16;

fn main() {
    let mut stream = TcpStream::connect("127.0.0.1:4242").expect("cannot connect");
    let mut write_stream = stream.try_clone().expect("cannot clone stream");

    let thr = thread::spawn(move || {
        let mut buf = [0; BUFFER_SIZE];

        loop {
            match io::stdin().read(&mut buf) {
                Ok(0) => break,
                Ok(n) => match write_stream.write_all(&buf[0..n]) {
                    Err(_) => break,
                    _ => (),
                },
                Err(_) => break,
            }
        }

        write_stream.shutdown(Shutdown::Write).ok();
    });

    {
        let mut buf = [0; BUFFER_SIZE];
        loop {
            match stream.read(&mut buf) {
                Ok(0) => break,
                Ok(n) => match io::stdout().write_all(&buf[0..n]) {
                    Err(_) => break,
                    _ => (),
                },
                Err(_) => break,
            }
        }
    }

    stream.shutdown(Shutdown::Read).ok();
    thr.join().expect("cannot wait thread");
}
