use std::collections::HashMap;

use color_eyre::eyre::{Context, Result};
use sqlx::{Pool, Sqlite};

use crate::db::db_mac::NoteIdentifier;

use super::{
    buttons::{Button, ButtonAction, ButtonState},
    editor::Editor,
    events::Events,
    inputs::{InputAction, InputState, UserInput},
    note_list::NoteList,
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum SidebarState {
    Open(u16),
    Hidden(u16),
}

#[derive(Debug)]
pub(crate) struct App<'a> {
    pub(crate) state: AppState,
    pub(crate) db: Pool<Sqlite>,
    pub(crate) current_screen: Screen,
    pub(crate) prev_screen: Screen,
    pub(crate) editor: Editor<'a>,
    pub(crate) note_list: NoteList<'a>,
    pub(crate) btns: HashMap<u8, Button>,
    pub(crate) btn_idx: u8,
    pub(crate) user_input: UserInput<'a>,
    pub(crate) user_msg: UserMessage,
    pub(crate) sidebar: SidebarState,
    pub(crate) sidebar_size: u16,
}

impl<'a> App<'a> {
    pub fn new(db: Pool<Sqlite>, note_identifiers: Vec<NoteIdentifier>) -> Self {
        let load_btn_state = match note_identifiers.len() {
            0 => ButtonState::Unavailable,
            _ => ButtonState::Inactive,
        };

        let note_list = NoteList::new(note_identifiers);

        Self {
            state: AppState::default(),
            db,
            current_screen: Screen::Welcome,
            prev_screen: Screen::Welcome,
            editor: Editor::new(" Untitled ".to_owned(), vec!["".to_owned()], vec![], None),
            note_list,
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
                        load_btn_state,
                        ButtonAction::RenderLoadNoteScreen,
                    ),
                ),
            ]),
            btn_idx: 0,
            user_input: UserInput::new(InputState::Active, InputAction::NewNote),
            user_msg: UserMessage::welcome(),
            sidebar: SidebarState::Open(20),
            sidebar_size: 20,
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
