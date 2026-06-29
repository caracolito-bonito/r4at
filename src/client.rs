use std::io;

use crossterm::{event::KeyEvent, style::Stylize};
use ratatui::{
    layout::{Constraint, Layout},
    style::Style,
    symbols::border,
    text::{Line, Span},
    widgets::{Block, Paragraph, Widget},
};

#[allow(unused)]
enum Event {
    Input(KeyEvent),
    Chat(String),
    Disconnect,
}

struct App {
    exit: bool,
}
impl App {
    fn run(&mut self, terminal: &mut ratatui::DefaultTerminal) -> io::Result<()> {
        while !self.exit {
            terminal.draw(|frame| self.draw(frame))?;
        }
        Ok(())
    }

    fn draw(&self, frame: &mut ratatui::prelude::Frame<'_>) {
        frame.render_widget(self, frame.area());
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
            Span::styled(" Clear all ", Style::new()),
            Span::styled(" <ESC> ", Style::new()),
            Span::styled(" Exit ", Style::new()),
            Span::styled(" <Control + Q> ", Style::new()),
        ])
        .centered();

        let block = Block::bordered()
            .title_bottom(instructions_for_input)
            .border_set(border::THICK);

        let input_par = Paragraph::new("Type your message here").block(block);

        input_par.render(input_area, buf);

        // let line = Line::from(vec![
        //     Span::styled("Hello", Style::new().blue()),
        //     Span::raw(" world!"),
        // ]);
    }
}

fn main() -> io::Result<()> {
    let mut terminal = ratatui::init();

    let mut app = App { exit: false };
    let app_result = app.run(&mut terminal);
    ratatui::restore();
    app_result
}
