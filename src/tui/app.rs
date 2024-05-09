use std::collections::HashMap;

use color_eyre::eyre::{Context, Result};
use sqlx::{Pool, Sqlite};
use tui_textarea::TextArea;

use super::{
    buttons::{Button, ButtonAction, ButtonState},
    events::Events,
    inputs::{InputAction, InputState, UserInput},
    ui::ui,
    user_messages::UserMessage,
    utils::Tui,
};

#[derive(PartialEq, Debug, Default, Clone, Copy)]
pub(crate) enum AppState {
    #[default]
    Running,
    Exit,
}

#[derive(PartialEq, Debug, Clone, Copy)]
pub(crate) enum Screen {
    Welcome,
    Main,
    NewNote,
    LoadNote,
    Popup,
    Exiting,
}

#[derive(Debug, Clone)]
pub(crate) struct Editor<'a> {
    pub(crate) title: String,
    pub(crate) body: TextArea<'a>,
}

#[derive(Debug)]
pub(crate) struct App<'a> {
    pub(crate) state: AppState,
    pub(crate) current_screen: Screen,
    pub(crate) prev_screen: Screen,
    pub(crate) editor: Editor<'a>,
    pub(crate) db: Pool<Sqlite>,
    pub(crate) btns: HashMap<u8, Button>,
    pub(crate) btn_idx: u8,
    pub(crate) user_input: UserInput<'a>,
    pub(crate) user_msg: UserMessage,
}

impl<'a> App<'a> {
    pub fn new(db: Pool<Sqlite>) -> Self {
        Self {
            state: AppState::default(),
            current_screen: Screen::Welcome,
            prev_screen: Screen::Welcome,
            editor: Editor {
                title: "New note".to_owned(),
                body: TextArea::default(),
            },
            db,
            btns: HashMap::from([
                (
                    0,
                    Button::new(
                        "New".to_owned(),
                        ButtonState::Active,
                        ButtonAction::RenderNewNoteScreen,
                    ),
                ),
                (
                    1,
                    Button::new(
                        "Load".to_owned(),
                        ButtonState::Inactive,
                        ButtonAction::RenderLoadNoteScreen,
                    ),
                ),
            ]),
            btn_idx: 0,
            user_input: UserInput::new(InputState::Active, InputAction::SubmitNoteTitle),
            user_msg: UserMessage::welcome(),
        }
    }
}

pub(crate) async fn run(app: &mut App<'_>, terminal: &mut Tui) -> Result<()> {
    // MAIN PROGRAM LOOP
    while app.state != AppState::Exit {
        terminal.draw(|frame| ui(app, frame))?;
        let result = Events::handle_events(app).await;

        result.wrap_err("handle events failed")?;
    }

    Ok(())
}
