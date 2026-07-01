use std::{
    env,
    io::{self, Read, Write},
    net::TcpStream,
    sync::mpsc,
    thread,
};

use crossterm::event::{
    self,
    Event::{self as CtEvent},
    KeyCode, KeyEvent, KeyEventKind, KeyModifiers,
};
use ratatui::{
    layout::{Constraint, Layout},
    style::{Color, Style},
    symbols::border,
    text::{Line, Span, Text},
    widgets::{Block, Paragraph, Widget},
};

#[allow(unused)]
enum Event {
    Terminal(CtEvent),
    Chat(String),
    Disconnect,
}

enum Status {
    Connected,
    Disconnected,
}

struct App {
    exit: bool,
    messages: Vec<String>,
    user_message: String,
    status: Status,
    stream: TcpStream,
}
impl App {
    fn run(
        &mut self,
        terminal: &mut ratatui::DefaultTerminal,
        rx: mpsc::Receiver<Event>,
    ) -> io::Result<()> {
        while !self.exit {
            terminal.draw(|frame| self.draw(frame))?;

            match rx.recv().unwrap() {
                Event::Terminal(CtEvent::Key(k)) => self.handle_key_events(k)?,
                Event::Chat(message) => {
                    self.messages.push(message);
                }
                Event::Disconnect => self.status = Status::Disconnected,
                Event::Terminal(_) => {}
            }
        }
        Ok(())
    }

    fn draw(&self, frame: &mut ratatui::prelude::Frame<'_>) {
        frame.render_widget(self, frame.area());
    }

    fn handle_key_events(&mut self, event: KeyEvent) -> std::io::Result<()> {
        match (event.kind, event.code, event.modifiers) {
            (KeyEventKind::Press, KeyCode::Char('q'), KeyModifiers::CONTROL) => self.exit = true,
            (KeyEventKind::Press, KeyCode::Backspace, KeyModifiers::NONE) => {
                let _ = self.user_message.pop();
            }
            (KeyEventKind::Press, KeyCode::Esc, KeyModifiers::NONE) => {
                self.user_message.clear();
            }
            (KeyEventKind::Press, KeyCode::Enter, KeyModifiers::NONE) => {
                let _ = self.stream.write_all(self.user_message.as_bytes());
                self.messages.push(self.user_message.clone());
                self.user_message.clear();
            }
            (KeyEventKind::Press, KeyCode::Char(c), modifier)
                if modifier == KeyModifiers::NONE || modifier == KeyModifiers::SHIFT =>
            {
                self.user_message.push(c);
            }

            _ => {}
        }
        Ok(())
    }
}

impl Widget for &App {
    fn render(self, area: ratatui::prelude::Rect, buf: &mut ratatui::prelude::Buffer)
    where
        Self: Sized,
    {
        let vertical_layout = Layout::vertical([
            Constraint::Fill(1),
            Constraint::Length(1),
            Constraint::Length(3),
        ]);
        let [messages_area, status_area, input_area] = vertical_layout.areas(area);

        let instructions_for_input = Line::from(vec![
            Span::styled(" Clear all ", Style::new().italic()),
            Span::styled(" <ESC> ", Style::new().italic()),
            Span::styled(" Exit ", Style::new().italic()),
            Span::styled(" <Control + Q> ", Style::new().italic()),
        ])
        .centered();

        let input_block = Block::bordered()
            .title_bottom(instructions_for_input)
            .border_set(border::THICK);

        Paragraph::new(self.user_message.as_str())
            .block(input_block)
            .render(input_area, buf);

        let (status_text, status_color) = match self.status {
            Status::Connected => ("CONNECTED", Color::LightGreen),
            Status::Disconnected => ("DISCONNECTED", Color::Gray),
        };

        Paragraph::new(status_text)
            .centered()
            .style(Style::new().bg(status_color))
            .render(status_area, buf);

        let messages_block = Block::bordered().border_set(border::THICK);

        let messages_as_lines: Vec<Line> = self
            .messages
            .iter()
            .map(|message| Line::from(message.as_str()))
            .collect();

        Paragraph::new(Text::from(messages_as_lines))
            .block(messages_block)
            .render(messages_area, buf);
    }
}

fn main() -> io::Result<()> {
    let mut terminal = ratatui::init();
    let addr = env::args().nth(1).expect("provide ip address");

    let stream_read = TcpStream::connect(format!("{addr}:6969"))?;

    let stream_write = stream_read.try_clone()?;

    let mut app = App {
        exit: false,
        messages: vec![],
        user_message: "".to_string(),
        status: Status::Connected,
        stream: stream_write,
    };

    let (tx_input, event_rx) = mpsc::channel::<Event>();
    let tx_reader = tx_input.clone();

    thread::spawn(move || handle_input_events(tx_input));
    thread::spawn(move || handle_chat_events(tx_reader, stream_read));

    let app_result = app.run(&mut terminal, event_rx);
    app_result
}

fn handle_chat_events(tx_reader: mpsc::Sender<Event>, mut stream: TcpStream) {
    let mut buffer = [0; 64];

    loop {
        match stream.read(&mut buffer) {
            Ok(0) => {
                tx_reader.send(Event::Disconnect).unwrap();
                break;
            }
            Ok(n) => tx_reader
                .send(Event::Chat(
                    String::from_utf8_lossy(&buffer[0..n]).into_owned(),
                ))
                .unwrap(),
            Err(_) => {
                tx_reader.send(Event::Disconnect).unwrap();
                break;
            }
        }
    }
}

fn handle_input_events(tx: mpsc::Sender<Event>) {
    loop {
        tx.send(Event::Terminal(event::read().unwrap())).unwrap()
    }
}
