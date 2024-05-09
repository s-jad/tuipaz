use color_eyre::eyre::{Context, Result};
use crossterm::event::{self, Event, KeyEventKind};
use tui_textarea::{Input, Key};

use crate::db::db_mac::DbMac;

use super::{
    app::{App, AppState, Screen},
    buttons::ButtonAction,
    inputs::{InputAction, InputState, UserInput},
    user_messages::{MessageType, UserMessage},
};

pub(crate) struct Events {}

impl Events {
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
        match app.current_screen {
            Screen::Welcome => match input {
                Input { key: Key::Esc, .. } => {
                    Self::show_exit_screen(app);
                }
                Input { key: Key::Tab, .. } => {
                    Self::switch_btns(app);
                }
                Input {
                    key: Key::Enter, ..
                } => {
                    Self::btn_action(app);
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
                    //Self::load_note(app).await?;
                }
                _ => {}
            },
            Screen::Main => match input {
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
            Screen::NewNote => match input {
                Input { key: Key::Esc, .. } => {
                    Self::show_exit_screen(app);
                }
                Input {
                    key: Key::Enter, ..
                } => {
                    Self::input_action(app);
                }
                input => {
                    app.user_input.text.input(input);
                }
            },
            Screen::LoadNote => match input {
                Input { key: Key::Esc, .. } => {
                    app.current_screen = Screen::Main;
                }
                _ => {}
            },
            Screen::Popup => match input {
                Input { key: Key::Esc, .. } => {
                    app.current_screen = app.prev_screen;
                }
                _ => {}
            },
            Screen::Exiting => match input {
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
                    app.current_screen = Screen::Main;
                }
                _ => {}
            },
        };
        Ok(())
    }

    async fn save_note(app: &mut App<'_>) -> Result<()> {
        let note = app.editor.body.lines().join("\n");
        let result = DbMac::save_note(&app.db, note, app.editor.title.clone()).await;

        match &result {
            Ok(_) => {
                let new_msg = UserMessage::new("Note saved!".to_string(), MessageType::Info);
                app.user_msg = new_msg;
                app.prev_screen = app.current_screen;
                app.current_screen = Screen::Popup;
            }
            Err(err) => {
                let new_msg =
                    UserMessage::new(format!("Error saving note!: {:?}", err), MessageType::Error);
                app.user_msg = new_msg;
                app.prev_screen = app.current_screen;
                app.current_screen = Screen::Popup;
            }
        }
        return result;
    }

    //async fn load_note(app: &mut App<'_>) -> Result<()> {
    //    let result = DbMac::load_note(&app.db, note).await;

    //    match &result {
    //        Ok(_) => {}
    //        Err(err) => {
    //            let new_msg = UserMessage::new(
    //                format!("Error saving note!: {:?}", err),
    //                true,
    //                2,
    //                MessageType::Error,
    //            );
    //            //app.user_msg = new_msg;
    //            app.screen = Screen::Popup;
    //        }
    //    }
    //    return result;
    //}
    fn init_note(app: &mut App<'_>) {
        let note = app.editor.body.lines().join("\n");
    }

    fn switch_btns(app: &mut App) {
        let inactive_btn = app
            .btns
            .get_mut(&app.btn_idx)
            .expect("Selected btn should exist");
        inactive_btn.deactivate();

        app.btn_idx = (app.btn_idx + 1) % (app.btns.len()) as u8;
        let active_btn = app
            .btns
            .get_mut(&app.btn_idx)
            .expect("Selected btn should exist");
        active_btn.activate();
    }

    fn btn_action(app: &mut App) {
        let active_btn = app
            .btns
            .get_mut(&app.btn_idx)
            .expect("Selected btn should exist");

        active_btn.clicked();

        match active_btn.get_action() {
            ButtonAction::RenderMainScreen => {
                app.current_screen = Screen::Main;
            }
            ButtonAction::RenderNewNoteScreen => {
                app.user_input = UserInput::new(InputState::Active, InputAction::SubmitNoteTitle);
                app.current_screen = Screen::NewNote;
            }
            ButtonAction::RenderLoadNoteScreen => {
                app.current_screen = Screen::LoadNote;
            }
        }
    }

    fn input_action(app: &mut App) {
        app.user_input.set_state(InputState::Submit);

        match app.user_input.get_action() {
            InputAction::SubmitNoteTitle => {
                let title = app.user_input.text.lines();

                let formatted = format!(" {} ", &title[0]);
                app.editor.title = formatted;

                app.current_screen = Screen::Main;
            }
        }
    }

    fn show_exit_screen(app: &mut App) {
        app.current_screen = Screen::Exiting;
    }

    fn exit(app: &mut App) {
        app.state = AppState::Exit;
    }
}
