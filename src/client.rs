use std::{
    error::Error,
    io::{Write, stdout},
    thread,
    time::Duration,
};

use crossterm::{
    QueueableCommand,
    cursor::MoveTo,
    event::{Event, KeyCode, KeyModifiers, poll, read},
    terminal::{self, Clear},
};

struct Rect {
    x: u16,
    y: u16,
    w: u16,
    h: u16,
}

fn chat_window(
    stdout: &mut impl Write,
    messages: &[String],
    boundary: Rect,
) -> Result<(), Box<dyn Error>> {
    let len = messages.len();

    let extra = len.checked_sub(boundary.h as usize).unwrap_or(0);
    for (dy, line) in messages.iter().skip(extra).enumerate() {
        stdout.queue(MoveTo(boundary.x, boundary.y + dy as u16))?;
        let bytes = line.as_bytes();
        stdout.write(bytes.get(0..boundary.w as usize).unwrap_or(bytes))?;
    }
    Ok(())
}

fn main() -> Result<(), Box<dyn Error>> {
    println!("Hi buddy");
    let mut stdout = stdout();
    let _ = terminal::enable_raw_mode()?;
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
                        chat.push(prompt.clone());
                        prompt.clear();
                    }
                    _ => {}
                },
                _ => {}
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
        stdout.write(border_line.as_bytes())?;
        stdout.write(prompt.as_bytes())?;
        stdout.queue(MoveTo(0, h - 1))?;
        stdout.flush()?;
        thread::sleep(Duration::from_millis(16));
    }
    terminal::disable_raw_mode()?;
    Ok(())
}
