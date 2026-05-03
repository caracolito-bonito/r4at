use std::fmt::Display;
use std::io::Write;
use std::net::TcpListener;
use std::sync::atomic::{AtomicBool, Ordering};

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
            writeln!(f, "{}", self.0)
        }
    }
}

fn main() -> Result<()> {
    let addr = "127.0.0.1:6969";
    set_sensitive_mode(true);
    let listener = TcpListener::bind(addr)
        .map_err(|err| eprintln!("ERROR: cound not bind {addr}: {}", Sensitive(err)))?;
    set_sensitive_mode(false);
    println!("Listening to {addr}");
    for stream in listener.incoming() {
        match stream {
            Ok(mut stream) => {
                let _ = writeln!(stream, "Hola, cabroncito");
            }
            Err(e) => {
                eprintln!("ERROR: could not accept connection: {e}")
            }
        }
    }

    Ok(())
}
