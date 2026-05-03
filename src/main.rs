use std::fmt::Display;
use std::io::Read;
use std::net::{TcpListener, TcpStream};
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
    ClientConnected,
    ClientDisconnected,
    NewMessage(Vec<u8>),
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
                thread::spawn(|| client(stream, message_sender));
            }
            Err(e) => {
                eprintln!("ERROR: could not accept connection: {e}")
            }
        }
    }

    Ok(())
}

fn server(_messages: Receiver<Message>) -> Result<()> {
    todo!()
}

fn client(mut stream: TcpStream, messages: Sender<Message>) -> Result<()> {
    messages
        .send(Message::ClientConnected)
        .map_err(|err| eprintln!("ERROR: Could not send message to the server thread: {err}"))?;

    let mut buffer = Vec::with_capacity(64);
    loop {
        let bytes_read = stream.read(&mut buffer).map_err(|err| {
            eprintln!("ERROR: Could not read message from client {err}");
            let _ = messages.send(Message::ClientDisconnected);
        })?;
        messages
            .send(Message::NewMessage(buffer[0..bytes_read].to_vec()))
            .map_err(|err| {
                eprintln!("ERROR: Failed to send a message to the server thread: {err}");
            })?;
    }
}
