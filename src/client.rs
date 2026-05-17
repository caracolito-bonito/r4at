use std::{
    error::Error,
    io::{Write, stdout},
    thread,
    time::Duration,
};

use crossterm::{
    QueueableCommand,
    cursor::{self, MoveRight, MoveTo},
    terminal::{self, Clear},
};

fn main() -> Result<(), Box<dyn Error>> {
    println!("Hello from client");
    let mut stdout = stdout();
    let (width, height) = terminal::size()?;
    stdout.queue(Clear(crossterm::terminal::ClearType::All))?;
    stdout.flush()?;
    stdout.queue(MoveTo(width / 2, height / 2))?;
    Ok(())
}
