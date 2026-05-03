use std::collections::HashMap;
use std::fmt::Display;
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{Receiver, Sender, channel};
use std::thread;

type Result<T> = std::result::Result<T, ()>;

static SENSITIVE_MODE: AtomicBool = AtomicBool::new(false);

fn set_sensitive_mode(enabled: bool) {
    SENSITIVE_MODE.store(enabled, Ordering::Relaxed);
}

struct Sensitive<T>(T);

impl<T: Display> Display for Sensitive<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if SENSITIVE_MODE.load(Ordering::Relaxed) {
            writeln!(f, "[REDACTED]")
        } else {
            self.0.fmt(f)
        }
    }
}

enum Message {
    ClientConnected {
        author: Arc<TcpStream>,
    },
    ClientDisconnected {
        author: Arc<TcpStream>,
    },
    NewMessage {
        author: Arc<TcpStream>,
        bytes: Vec<u8>,
    },
}

struct Client {
    conn: Arc<TcpStream>,
}

fn main() -> Result<()> {
    let addr = "127.0.0.1:6969";
    set_sensitive_mode(true);
    let listener = TcpListener::bind(addr)
        .map_err(|err| eprintln!("ERROR: cound not bind {addr}: {}", Sensitive(err)))?;
    set_sensitive_mode(false);

    println!("Listening to {}", Sensitive(addr));

    let (message_sender, message_recevier): (Sender<Message>, Receiver<Message>) = channel();

    thread::spawn(|| server(message_recevier));

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                let message_sender = message_sender.clone();
                thread::spawn(|| client(Arc::new(stream), message_sender));
            }
            Err(e) => {
                eprintln!("ERROR: could not accept connection: {e}")
            }
        }
    }

    Ok(())
}

fn server(messages: Receiver<Message>) -> Result<()> {
    let mut clients = HashMap::new();

    loop {
        let msg = messages.recv().expect("The server receiver is not hung up");

        match msg {
            Message::ClientConnected { author } => {
                let addr = author
                    .peer_addr()
                    .expect("TODO: cache the peer address of the connection");
                clients.insert(
                    addr.clone(),
                    Client {
                        conn: author.clone(),
                    },
                );
            }
            Message::ClientDisconnected { author } => {
                let addr = author
                    .peer_addr()
                    .expect("TODO: cache the peer address of the connection");

                clients.remove(&addr);
            }
            Message::NewMessage { author, bytes } => {
                let author_addr = author
                    .peer_addr()
                    .expect("TODO: cache the peer address of the connection");

                for (addr, client) in clients.iter() {
                    if *addr != author_addr {
                        let _ = client.conn.as_ref().write(&bytes);
                    }
                }
            }
        }
    }
}

fn client(stream: Arc<TcpStream>, messages: Sender<Message>) -> Result<()> {
    messages
        .send(Message::ClientConnected {
            author: stream.clone(),
        })
        .map_err(|err| eprintln!("ERROR: Could not send message to the server thread: {err}"))?;

    let mut buffer = [0u8; 64];

    loop {
        let bytes_read = stream.as_ref().read(&mut buffer).map_err(|err| {
            eprintln!("ERROR: Could not read message from client {err}");
            let _ = messages.send(Message::ClientDisconnected {
                author: stream.clone(),
            });
        })?;

        if bytes_read == 0 {
            let _ = messages.send(Message::ClientDisconnected {
                author: stream.clone(),
            });
            return Ok(());
        }

        messages
            .send(Message::NewMessage {
                author: stream.clone(),
                bytes: buffer[0..bytes_read].to_vec(),
            })
            .map_err(|err| {
                eprintln!("ERROR: Failed to send a message to the server thread: {err}");
            })?;
    }
}
