use std::io::Write;
use std::net::TcpListener;

type Result<T> = std::result::Result<T, ()>;

fn main() -> Result<()> {
    let addr = "127.0.0.1:6969";
    let listener =
        TcpListener::bind(addr).map_err(|err| eprintln!("ERROR: cound not bind {addr}: {err}"))?;

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
