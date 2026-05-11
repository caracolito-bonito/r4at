use std::collections::HashMap;
use std::fmt::Display;
use std::fmt::Write as FmtWrite;
use std::io::{Read, Write};
use std::net::{IpAddr, SocketAddr, TcpListener, TcpStream};
use std::str;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{Receiver, Sender, channel};
use std::thread;
use std::time::{Duration, SystemTime};

type Result<T> = std::result::Result<T, ()>;

static SENSITIVE_MODE: AtomicBool = AtomicBool::new(false);
const BAN_LIMIT: Duration = Duration::from_secs(10 * 60);
const MESSAGE_RATE: Duration = Duration::from_secs(1);
const STRIKE_LIMIT: u64 = 10;

#[allow(dead_code)]
fn set_sensitive_mode(enabled: bool) {
    SENSITIVE_MODE.store(enabled, Ordering::Relaxed);
}

struct Sensitive<T>(T);

impl<T: Display> Display for Sensitive<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if SENSITIVE_MODE.load(Ordering::Relaxed) {
            write!(f, "[REDACTED]")
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
        author_addr: SocketAddr,
    },
    NewMessage {
        author_addr: SocketAddr,
        bytes: Vec<u8>,
    },
}

struct Client {
    conn: Arc<TcpStream>,
    last_message: SystemTime,
    strike_count: u64,
}

fn main() -> Result<()> {
    let mut token_raw = [0; 16];
    let _ = getrandom::fill(&mut token_raw)
        .map_err(|err| eprintln!("ERROR: Couldn't generate raw token: {err}"));

    let mut token = String::new();

    for b in token_raw.iter() {
        let _ = write!(token, "{b:02X}");
    }

    eprintln!("Auth token is: {token}");

    let addr = "0.0.0.0:6969";
    let listener = TcpListener::bind(addr)
        .map_err(|err| eprintln!("ERROR: cound not bind {addr}: {}", Sensitive(err)))?;

    println!("Listening to {}", Sensitive(addr));

    let (message_sender, message_recevier): (Sender<Message>, Receiver<Message>) = channel();

    thread::spawn(|| server(message_recevier));

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                let message_sender = message_sender.clone();
                let token = token.clone();
                thread::spawn(move || client(Arc::new(stream), message_sender, token.clone()));
            }
            Err(e) => {
                eprintln!("ERROR: could not accept connection: {e}")
            }
        }
    }

    Ok(())
}

fn server(messages: Receiver<Message>) -> Result<()> {
    let mut clients = HashMap::<SocketAddr, Client>::new();
    let mut banned_users = HashMap::<IpAddr, SystemTime>::new();

    loop {
        let msg = messages.recv().expect("The server receiver is not hung up");

        match msg {
            Message::ClientConnected { author } => {
                let author_addr = author
                    .peer_addr()
                    .expect("TODO: cache the peer address of the connection");

                let banned_at = banned_users.remove(&author_addr.ip());
                let now = SystemTime::now();

                let still_banned = banned_at.and_then(|banned_at| {
                    let diff = now
                        .duration_since(banned_at)
                        .expect("TODO: we shouldn't crash if the clock goes backwards");
                    if diff >= BAN_LIMIT {
                        None
                    } else {
                        Some(banned_at)
                    }
                });

                if let Some(banned_at) = still_banned {
                    banned_users.insert(author_addr.ip(), banned_at);

                    let diff = now
                        .duration_since(banned_at)
                        .expect("TODO: we shouldn't crash if the clock goes backwards");

                    let mut author = author.as_ref();
                    let _ = writeln!(
                        author,
                        "You are banned! {secs}s left",
                        secs = (BAN_LIMIT - diff).as_secs_f32()
                    );
                    let _ = author.shutdown(std::net::Shutdown::Both);
                } else {
                    clients.insert(
                        author_addr.clone(),
                        Client {
                            conn: author.clone(),
                            last_message: now,
                            strike_count: 0,
                        },
                    );
                    println!("INFO: Client {author_addr} connected");
                }
            }
            Message::ClientDisconnected { author_addr } => {
                clients.remove(&author_addr);
                println!("INFO: Client {author_addr} disconnected");
            }
            Message::NewMessage { author_addr, bytes } => {
                if let Some(author) = clients.get_mut(&author_addr) {
                    let now = SystemTime::now();

                    let diff = now
                        .duration_since(author.last_message)
                        .expect("TODO: we shouldn't crash if the clock goes backwards");

                    if diff >= MESSAGE_RATE {
                        if str::from_utf8(&bytes).is_ok() {
                            author.last_message = now;
                            println!("Client {author_addr} sent message {bytes:?}");
                            for (addr, client) in clients.iter() {
                                if *addr != author_addr {
                                    let _ = client.conn.as_ref().write(&bytes);
                                }
                            }
                        } else {
                            author.strike_count += 1;
                            if author.strike_count >= STRIKE_LIMIT {
                                banned_users.insert(author_addr.ip(), now);
                                let _ = writeln!(
                                    author.conn.as_ref(),
                                    "You are banned! {secs}s left",
                                    secs = (BAN_LIMIT - diff).as_secs_f32()
                                );
                                let _ = author.conn.shutdown(std::net::Shutdown::Both);
                                println!("INFO: Client {author_addr} banned");
                            }
                        }
                    } else {
                        author.strike_count += 1;
                        if author.strike_count >= STRIKE_LIMIT {
                            banned_users.insert(author_addr.ip(), now);
                            let _ = writeln!(
                                author.conn.as_ref(),
                                "You are banned! {secs}s left",
                                secs = (BAN_LIMIT - diff).as_secs_f32()
                            );
                            let _ = author.conn.shutdown(std::net::Shutdown::Both);
                            println!("INFO: Client {author_addr} disconnected");
                        }
                    }
                }
            }
        }
    }
}

fn authorize(stream: Arc<TcpStream>, addr: &SocketAddr, token: &str) -> Result<()> {
    let mut buffer: [u8; 32] = [0; 32];

    let bytes_read = stream
        .as_ref()
        .read(&mut buffer)
        .map_err(|err| eprintln!("ERROR: Could not read auth token from {}:{}", addr, err))?;

    if bytes_read < buffer.len() {
        eprintln!("ERROR: didn't fully read the auth token: only {bytes_read} bytes");
        return Err(());
    }

    let user_token = str::from_utf8(&buffer)
        .map_err(|err| eprintln!("ERROR: token is not valid utf8: {err}"))?;

    if user_token != token {
        eprintln!("ERROR: User provided invalid token");
        return Err(());
    }
    Ok(())
}

fn client(stream: Arc<TcpStream>, messages: Sender<Message>, expected_token: String) -> Result<()> {
    let author_addr = stream
        .peer_addr()
        .map_err(|err| eprintln!("Could not get peer address: {err}"))?;

    let _ = write!(stream.as_ref(), "Token: ")
        .map_err(|err| eprintln!("Could not send token prompt to {}: {}", author_addr, err));

    stream
        .set_read_timeout(Some(Duration::from_secs(10)))
        .map_err(|err| eprintln!("Could not set read timeout on client stream: {err}"))?;

    authorize(stream.clone(), &author_addr, &expected_token).map_err(|()| {
        let _ = writeln!(stream.as_ref(), "Authorization failed!").map_err(|err| {
            eprintln!(
                "Could not send auth failed prompt to {}: {}",
                author_addr, err
            )
        });
        let _ = stream
            .shutdown(std::net::Shutdown::Both)
            .map_err(|err| eprintln!("ERROR: Could not shutdown {}:{}", author_addr, err));
    })?;

    let _ = writeln!(
        stream.as_ref(),
        "Authorization suceeded, now you can send messages!"
    )
    .map_err(|err| {
        eprintln!(
            "Could not send auth succesfull prompt to {}: {}",
            author_addr, err
        )
    });

    println!("INFO: {} authorized", author_addr);

    stream.set_read_timeout(None).map_err(|err| {
        eprintln!("Couldn't disable read timeout after succesful auth on client stream: {err}")
    })?;

    messages
        .send(Message::ClientConnected {
            author: stream.clone(),
        })
        .map_err(|err| eprintln!("ERROR: Could not send message to the server thread: {err}"))?;

    let mut buffer = [0u8; 64];

    loop {
        let bytes_read = stream.as_ref().read(&mut buffer).map_err(|err| {
            eprintln!("ERROR: Could not read message from client {err}");
            let _ = messages.send(Message::ClientDisconnected { author_addr });
        })?;

        if bytes_read == 0 {
            let _ = messages.send(Message::ClientDisconnected { author_addr });
            return Ok(());
        }
        let mut bytes = Vec::new();

        for b in &buffer[0..bytes_read].to_vec() {
            if *b >= 32 {
                bytes.push(*b);
            }
        }

        messages
            .send(Message::NewMessage { author_addr, bytes })
            .map_err(|err| {
                eprintln!("ERROR: Failed to send a message to the server thread: {err}");
            })?;
    }
}
