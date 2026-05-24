use std::{
    error::Error,
    io::{Write, stdout},
    process::exit,
    thread,
    time::Duration,
};

use crossterm::{
    QueueableCommand,
    cursor::MoveTo,
    event::{Event, KeyCode, KeyModifiers, poll, read},
    terminal::{self, Clear},
};

fn main() -> Result<(), Box<dyn Error>> {
    println!("Hi buddy");
    let mut stdout = stdout();
    let _ = terminal::enable_raw_mode()?;
    let (mut w, mut h) = terminal::size()?;
    let bar = "─";
    let mut border_line = bar.repeat(w as usize);
    let mut prompt = String::new();
    let mut exit = false;
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
                        }
                        prompt.push(c);
                    }
                    KeyCode::Backspace => {
                        prompt.pop();
                    }
                    _ => {}
                },
                _ => {}
            }
        }

        // rendering chat window
        stdout.queue(Clear(terminal::ClearType::All))?;
        stdout.queue(MoveTo(0, h - 2))?;
        stdout.write(border_line.as_bytes())?;
        stdout.write(prompt.as_bytes())?;
        stdout.queue(MoveTo(0, h - 1))?;
        stdout.flush()?;
        thread::sleep(Duration::from_millis(16));
    }
    Ok(())
}
