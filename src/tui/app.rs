use color_eyre::eyre::{Context, Result};
use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind};
use ratatui::{
    layout::Alignment,
    style::Stylize,
    symbols::border,
    text::{Line, Text},
    widgets::{
        block::{Position::Bottom, Title},
        Block, Borders, Paragraph, Widget,
    },
    Frame,
};

use super::utils::Tui;

#[derive(Debug, Default, Clone, Copy)]
pub(crate) struct App {
    pub(crate) exit: bool,
}

impl App {
    pub fn run(&mut self, terminal: &mut Tui) -> Result<()> {
        while !self.exit {
            terminal.draw(|frame| self.render_frame(frame))?;
            self.handle_events().wrap_err("handle events failed")?;
        }

        Ok(())
    }

    fn render_frame(self, frame: &mut Frame) {
        frame.render_widget(self, frame.size());
    }

    fn handle_events(&mut self) -> Result<()> {
        match event::read()? {
            Event::Key(key_event) if key_event.kind == KeyEventKind::Press => self
                .handle_key_event(key_event)
                .wrap_err_with(|| format!("handling key event failed:\n{key_event:#?}")),
            _ => Ok(()),
        }
    }

    pub(crate) fn handle_key_event(&mut self, key_event: KeyEvent) -> Result<()> {
        match key_event.code {
            KeyCode::Char('q') => self.exit(),
            _ => {}
        }
        Ok(())
    }

    fn exit(&mut self) {
        self.exit = true;
    }
}

impl Widget for App {
    fn render(self, area: ratatui::prelude::Rect, buf: &mut ratatui::prelude::Buffer)
    where
        Self: Sized,
    {
        let title = Title::from(" Counter App Tutorial ".bold());
        let instructions = Title::from(Line::from(vec![" Quit ".into(), "<Q> ".blue().bold()]));

        let block = Block::default()
            .title(title.alignment(Alignment::Center))
            .title(instructions.alignment(Alignment::Center).position(Bottom))
            .borders(Borders::all())
            .border_set(border::THICK);

        let welcome_txt = Text::from(vec![Line::from(vec!["Welcome to tuipaz!".into()])]);

        Paragraph::new(welcome_txt)
            .centered()
            .block(block)
            .render(area, buf);
    }
}
