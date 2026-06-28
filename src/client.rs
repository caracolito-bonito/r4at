use std::{
    env,
    io::{self, ErrorKind, Read, Write, stdout},
    net::TcpStream,
    thread,
    time::Duration,
};

use crossterm::{
    QueueableCommand,
    cursor::MoveTo,
    event::{Event, KeyCode, KeyModifiers, poll, read},
    style::Print,
    terminal::{self, Clear},
};

struct Rect {
    x: u16,
    y: u16,
    w: u16,
    h: u16,
}

struct RawMode;

impl RawMode {
    fn enable() -> io::Result<Self> {
        terminal::enable_raw_mode()?;
        Ok(Self)
    }
}

impl Drop for RawMode {
    fn drop(&mut self) {
        let _ =
            terminal::disable_raw_mode().map_err(|err| eprintln!("ERROR: disable raw mode: {err}"));
    }
}

fn chat_window(
    qc: &mut impl QueueableCommand,
    messages: &[String],
    boundary: Rect,
) -> io::Result<()> {
    let len = messages.len();
    let extra = len.saturating_sub(boundary.h as usize);
    for (dy, line) in messages.iter().skip(extra).enumerate() {
        qc.queue(MoveTo(boundary.x, boundary.y + dy as u16))?;
        qc.queue(Print(line.get(0..boundary.w as usize).unwrap_or(&line)))?;
    }
    Ok(())
}

fn main() -> io::Result<()> {
    let address = env::args().skip(1).next().expect("Provide IP please");

    let mut stream = TcpStream::connect(format!("{address}:6969"))?;

    let _ = stream.set_nonblocking(true);

    let mut buffer = [0; 64];

    let mut stdout = stdout();

    let _raw_mode = RawMode::enable()?;

    let (mut w, mut h) = terminal::size()?;
    let bar = "─";
    let mut border_line = bar.repeat(w as usize);
    let mut prompt = String::new();
    let mut exit = false;
    let mut chat = Vec::new();
    while !exit {
        while poll(Duration::ZERO)? {
            match read()? {
                Event::Resize(nw, nh) => {
                    w = nw;
                    h = nh;
                    border_line = bar.repeat(w as usize);
                }
                Event::Key(event) => match event.code {
                    KeyCode::Char(c) => {
                        if c == 'c' && event.modifiers.contains(KeyModifiers::CONTROL) {
                            exit = true;
                        } else {
                            prompt.push(c);
                        }
                    }
                    KeyCode::Backspace => {
                        prompt.pop();
                    }
                    KeyCode::Enter => {
                        stream.write_all(prompt.as_bytes())?;
                        chat.push(prompt.clone());
                        prompt.clear();
                    }
                    _ => {}
                },
                _ => {}
            }
        }

        match stream.read(&mut buffer) {
            Ok(0) => break,
            Ok(n) => chat.push(String::from_utf8_lossy(&buffer[0..n]).into_owned()),
            Err(e) => {
                if e.kind() != ErrorKind::WouldBlock {
                    return Err(e);
                }
            }
        }

        // rendering chat window
        stdout.queue(Clear(terminal::ClearType::All))?;

        chat_window(
            &mut stdout,
            &chat,
            Rect {
                x: 0,
                y: 0,
                w,
                h: h - 2,
            },
        )?;

        stdout.queue(MoveTo(0, h - 2))?;
        stdout.queue(Print(&border_line))?;
        stdout.queue(MoveTo(0, h - 1))?;
        stdout.queue(Print(prompt.get(0..w as usize).unwrap_or(&prompt)))?;

        stdout.flush()?;
        thread::sleep(Duration::from_millis(16));
    }
    Ok(())
}
