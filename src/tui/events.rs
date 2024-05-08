use color_eyre::eyre::{Context, Result};
use crossterm::event::{self, Event, KeyEventKind};
use tui_textarea::{Input, Key};

use crate::db::db_mac::DbMac;

use super::{
    app::{App, AppState, CurrentScreen},
    user_messages::{MessageType, UserMessage},
};

pub(crate) struct AppMac {}

impl AppMac {
    pub(crate) async fn handle_events(app: &mut App<'_>) -> Result<()> {
        match event::read()? {
            Event::Key(key_event) if key_event.kind == KeyEventKind::Press => {
                let result = Self::handle_key_event(app, key_event.into()).await;
                result.wrap_err_with(|| format!("handling key event failed:\n{key_event:#?}"))
            }
            _ => Ok(()),
        }
    }

    async fn handle_key_event(app: &mut App<'_>, input: Input) -> Result<()> {
        match app.screen {
            CurrentScreen::Welcome => match input {
                Input { key: Key::Esc, .. } => {
                    Self::show_exit_screen(app);
                }
                Input {
                    key: Key::Char('n'),
                    ..
                } => {
                    Self::init_note(app);
                }
                Input {
                    key: Key::Char('l'),
                    ..
                } => {
                    Self::load_note(app).await?;
                }
                _ => {}
            },
            CurrentScreen::Main => match input {
                Input { key: Key::Esc, .. } => {
                    Self::show_exit_screen(app);
                }
                Input {
                    key: Key::Char('s'),
                    alt: true,
                    ..
                } => {
                    Self::save_note(app).await?;
                }
                input => {
                    app.editor.body.input(input);
                }
            },
            CurrentScreen::Popup => match input {
                Input { key: Key::Esc, .. } => {
                    app.screen = CurrentScreen::Main;
                }
                _ => {}
            },
            CurrentScreen::Exiting => match input {
                Input {
                    key: Key::Char('y'),
                    ..
                } => {
                    Self::exit(app);
                }
                Input {
                    key: Key::Char('n'),
                    ..
                } => {
                    app.screen = CurrentScreen::Main;
                }
                _ => {}
            },
        };
        Ok(())
    }

    async fn save_note(app: &mut App<'_>) -> Result<()> {
        let note = app.editor.body.lines().join("\n");
        let result = DbMac::save_note(&app.db, note).await;

        match &result {
            Ok(_) => {
                let new_msg =
                    UserMessage::new("Note saved!".to_string(), true, 2, MessageType::Info);
                //app.user_msg = new_msg;
                app.screen = CurrentScreen::Popup;
            }
            Err(err) => {
                let new_msg = UserMessage::new(
                    format!("Error saving note!: {:?}", err),
                    true,
                    2,
                    MessageType::Error,
                );
                //app.user_msg = new_msg;
                app.screen = CurrentScreen::Popup;
            }
        }
        return result;
    }

    async fn load_note(app: &mut App<'_>) -> Result<()> {
        let result = DbMac::load_note(&app.db, note).await;

        match &result {
            Ok(_) => {}
            Err(err) => {
                let new_msg = UserMessage::new(
                    format!("Error saving note!: {:?}", err),
                    true,
                    2,
                    MessageType::Error,
                );
                //app.user_msg = new_msg;
                app.screen = CurrentScreen::Popup;
            }
        }
        return result;
    }
    fn init_note(app: &mut App<'_>) {
        let note = app.editor.body.lines().join("\n");
    }

    fn show_exit_screen(app: &mut App) {
        app.screen = CurrentScreen::Exiting;
    }

    fn exit(app: &mut App) {
        app.state = AppState::Exit;
    }
}
