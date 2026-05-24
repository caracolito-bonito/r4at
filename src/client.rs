use std::{
    error::Error,
    io::{Write, stdout},
    thread,
    time::Duration,
};

use crossterm::{
    QueueableCommand,
    cursor::MoveTo,
    event::{Event, poll, read},
    terminal::{self, Clear},
};

fn main() -> Result<(), Box<dyn Error>> {
    println!("Hi buddy");
    let mut stdout = stdout();
    let (mut w, mut h) = terminal::size()?;
    let bar = "-";
    let mut border_line = bar.repeat(w as usize);
    loop {
        while poll(Duration::ZERO)? {
            match read()? {
                Event::Resize(nw, nh) => {
                    w = nw;
                    h = nh;
                    border_line = bar.repeat(w as usize);
                }
                Event::Key(event) => todo!(),
                _ => {}
            }
        }

        // rendering chat window
        stdout.queue(Clear(terminal::ClearType::All))?;
        stdout.queue(MoveTo(0, h - 2))?;
        stdout.write(border_line.as_bytes())?;
        stdout.queue(MoveTo(0, h - 1))?;
        stdout.flush()?;
        thread::sleep(Duration::from_millis(16));
    }
}
