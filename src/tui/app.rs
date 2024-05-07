use color_eyre::eyre::{Context, Result};
use crossterm::event::{self, Event, KeyEventKind};
use ratatui::{
    buffer::Buffer,
    layout::{Alignment, Constraint, Direction, Layout},
    style::{Modifier, Style},
    text::Line,
    widgets::{block::Title, Block, BorderType, Borders, Paragraph, Widget, Wrap},
    Frame,
};
use sqlx::{Pool, Sqlite};
use tui_textarea::{Input, Key, TextArea};

use crate::db::db_mac::NotesMac;

use super::{user_messages::UserMessage, utils::Tui};

#[derive(PartialEq, Debug, Default, Clone, Copy)]
pub(crate) enum AppState {
    #[default]
    Running,
    Exit,
}

#[derive(Debug, Clone)]
pub(crate) struct App<'a> {
    pub(crate) state: AppState,
    pub(crate) db: Pool<Sqlite>,
    pub(crate) note_editor: TextArea<'a>,
    pub(crate) user_message: UserMessage,
}

impl<'a> App<'a> {
    pub fn new(db: Pool<Sqlite>) -> Self {
        Self {
            state: AppState::default(),
            db,
            note_editor: TextArea::default(),
            user_message: UserMessage::welcome(),
        }
    }
    pub async fn run(&mut self, terminal: &mut Tui) -> Result<()> {
        // MAIN PROGRAM LOOP
        while self.state != AppState::Exit {
            terminal.draw(|frame| self.clone().render_frame(frame))?;
            let result = self.handle_events().await;

            result.wrap_err("handle events failed")?;
        }

        Ok(())
    }

    fn render_frame(self, frame: &mut Frame) {
        frame.render_widget(self, frame.size());
    }

    async fn handle_events(&mut self) -> Result<()> {
        match event::read()? {
            Event::Key(key_event) if key_event.kind == KeyEventKind::Press => {
                let result = self.handle_key_event(key_event.into()).await;
                result.wrap_err_with(|| format!("handling key event failed:\n{key_event:#?}"))
            }
            _ => Ok(()),
        }
    }

    async fn handle_key_event(&mut self, input: Input) -> Result<()> {
        match input {
            Input { key: Key::Esc, .. } => {
                self.exit();
            }
            Input {
                key: Key::Char('s'),
                alt: true,
                ..
            } => {
                self.save_note().await?;
            }
            input => {
                self.note_editor.input(input);
            }
        };
        Ok(())
    }

    async fn save_note(&self) -> Result<()> {
        let note = self.note_editor.lines().join("\n");
        let result = NotesMac::save_note(&self.db, note).await;

        return result;
    }

    fn exit(&mut self) {
        self.state = AppState::Exit;
    }
}

impl<'a> Widget for App<'a> {
    fn render(mut self, area: ratatui::prelude::Rect, buf: &mut Buffer)
    where
        Self: Sized,
    {
        let layout = Layout::default()
            .direction(Direction::Horizontal)
            .constraints(&[Constraint::Percentage(80), Constraint::Percentage(20)])
            .split(area);

        let note_block = Block::default()
            .title(Title::from("Note Editor").alignment(Alignment::Center))
            .title_style(Style::default().add_modifier(Modifier::BOLD))
            .borders(Borders::ALL)
            .border_type(BorderType::Thick)
            .title_bottom(Line::from(" <Esc> Quit "));

        self.note_editor.set_block(note_block);

        let note_editor_widget = self.note_editor.widget();

        note_editor_widget.render(layout[0], buf);

        let files_block = Block::default()
            .title(Title::from("File Explorer").alignment(Alignment::Center))
            .title_style(Style::default().add_modifier(Modifier::BOLD))
            .borders(Borders::ALL)
            .border_type(BorderType::Thick);

        let files_text =
            vec![Line::from("This is the file explorer sidebar").style(Style::default())];

        Paragraph::new(files_text)
            .block(files_block)
            .wrap(Wrap { trim: true })
            .render(layout[1], buf);
    }
}
