use std::{
    env,
    io::{self, Read, Write},
    net::TcpStream,
    sync::mpsc,
    thread, usize,
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
    widgets::{Block, List, ListState, Paragraph},
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
    chat_state: ListState,
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
                    self.push_message(message);
                }
                Event::Disconnect => self.status = Status::Disconnected,
                Event::Terminal(_) => {}
            }
        }
        Ok(())
    }

    fn draw(&mut self, frame: &mut ratatui::prelude::Frame<'_>) {
        let input_width = frame.area().width.saturating_sub(2).max(1);

        let input_lines = wrap_text(&self.user_message, input_width as usize);

        let input_height = (input_lines.len().max(1) + 2).min(6);

        let vertical_layout = Layout::vertical([
            Constraint::Fill(1),
            Constraint::Length(1),
            Constraint::Length(input_height as u16),
        ]);
        let [chat_area, status_area, input_area] = vertical_layout.areas(frame.area());

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

        let input = Paragraph::new(Text::from(input_lines)).block(input_block);

        frame.render_widget(input, input_area);

        let (status_text, status_color) = match self.status {
            Status::Connected => ("CONNECTED", Color::LightGreen),
            Status::Disconnected => ("DISCONNECTED", Color::Gray),
        };
        let status = Paragraph::new(status_text)
            .centered()
            .style(Style::new().bg(status_color));

        frame.render_widget(status, status_area);

        let chat_block = Block::bordered().border_set(border::THICK);
        let chat_width = chat_area.width.saturating_sub(2).max(1);

        let message_list: Vec<Text> = self
            .messages
            .iter()
            .map(|m| wrap_text(m, chat_width as usize))
            .map(|wm| Text::from(wm))
            .collect();

        let chat = List::new(message_list).block(chat_block);

        frame.render_stateful_widget(chat, chat_area, &mut self.chat_state);
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
                self.push_message(self.user_message.clone());
                self.user_message.clear();
            }
            (KeyEventKind::Press, KeyCode::Char(c), modifier)
                if modifier == KeyModifiers::NONE || modifier == KeyModifiers::SHIFT =>
            {
                self.user_message.push(c);
            }
            (KeyEventKind::Press, KeyCode::Up, KeyModifiers::NONE) => {
                self.chat_state.scroll_up_by(5);
            }
            (KeyEventKind::Press, KeyCode::Down, KeyModifiers::NONE) => {
                self.chat_state.scroll_down_by(5);
            }
            _ => {}
        }
        Ok(())
    }

    fn push_message(&mut self, message: String) {
        self.messages.push(message);
        self.chat_state.select(Some(self.messages.len() - 1));
    }
}

fn wrap_text(message: &str, width: usize) -> Vec<Line<'static>> {
    let wrapped = message
        .chars()
        .collect::<Vec<char>>()
        .chunks(width)
        .map(|chunk| Line::from(chunk.iter().collect::<String>()))
        .collect();
    wrapped
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
        chat_state: ListState::default(),
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
