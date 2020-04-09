use std::collections::HashMap;
use std::io::prelude::*;
use std::net::{TcpListener, TcpStream};
use std::sync::{mpsc, Arc};
use std::thread;

const BUFFER_SIZE: usize = 1 << 16;

enum Event {
    RemoveClient(String),
    NewClient(String, mpsc::SyncSender<Arc<Vec<u8>>>),
    Broadcast(Arc<Vec<u8>>),
}

fn handle_connection(sender: mpsc::SyncSender<Event>, mut stream: TcpStream) {
    let addr = match stream.peer_addr() {
        Err(_) => return,
        Ok(addr) => addr.to_string(),
    };
    let event_addr = addr.clone();
    let write_addr = addr.clone();
    let mut write_stream = match stream.try_clone() {
        Err(_) => return,
        Ok(s) => s,
    };

    let (client_sender, client_receiver) = mpsc::sync_channel(0);
    let write_sender = sender.clone();
    match sender.send(Event::NewClient(event_addr, client_sender)) {
        Err(_) => return,
        Ok(_) => (),
    }
    thread::spawn(move || {
        for msg in client_receiver {
            match write_stream.write_all(msg.as_slice()) {
                Ok(_) => match write_stream.flush() {
                    Ok(_) => (),
                    Err(_) => break,
                },
                Err(_) => break,
            }
        }
        write_sender.send(Event::RemoveClient(write_addr)).ok();
    });
    thread::spawn(move || {
        let mut buf = [0; BUFFER_SIZE];
        loop {
            match stream.read(&mut buf) {
                Ok(0) => break,
                Ok(n) => {
                    let msg = Vec::from(&buf[0..n]);
                    match sender.send(Event::Broadcast(Arc::new(msg))) {
                        Ok(_) => (),
                        Err(_) => break,
                    }
                }
                Err(_) => break,
            }
        }
    });
}

fn handle_events(receiver: mpsc::Receiver<Event>) {
    let mut clients = HashMap::new();

    for event in receiver {
        match event {
            Event::NewClient(addr, sender) => {
                clients.insert(addr, sender);
                println!("{} clients connected", clients.len());
            }
            Event::RemoveClient(addr) => {
                clients.remove(&addr);
                println!("{} clients left", clients.len());
            }
            Event::Broadcast(msg) => {
                let mut deads: Vec<String> = Vec::new();
                for (addr, sender) in clients.iter_mut() {
                    match sender.send(msg.clone()) {
                        Ok(_) => (),
                        Err(_) => deads.push(addr.clone()),
                    }
                }

                for addr in deads {
                    clients.remove(&addr);
                }
            }
        }
    }
}

fn main() {
    let (sender, receiver) = mpsc::sync_channel(0);

    thread::spawn(move || {
        handle_events(receiver);
    });

    let listener = TcpListener::bind("127.0.0.1:4242").unwrap();

    for stream in listener.incoming() {
        let stream = stream.unwrap();
        let sender = sender.clone();
        handle_connection(sender, stream);
    }
}
