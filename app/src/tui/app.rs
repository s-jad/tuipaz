use std::collections::HashMap;

use color_eyre::eyre::{Context, Result};
use sqlx::{Pool, Sqlite};

use crate::db::db_mac::NoteIdentifier;
use tuipaz_textarea::Link as TextAreaLink;

use super::{
    buttons::{Button, ButtonAction},
    editor::Editor,
    events::Events,
    inputs::{InputAction, UserInput},
    note_list::{NoteList, NoteListAction, NoteListMode},
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

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub(crate) enum ComponentState {
    Active,
    Inactive,
    Unavailable,
    Error,
}

#[derive(PartialEq, Debug, Clone, Copy)]
pub(crate) enum ActiveWidget {
    Editor,
    Sidebar,
    NoteList,
    NoteTitleInput,
}

#[derive(PartialEq, Debug, Clone, Copy)]
pub(crate) enum Screen {
    Welcome,
    Main,
    NewNote,
    NewLinkedNote,
    LoadNote,
    DeleteNoteConfirmation,
    Popup,
    Exiting,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum SidebarState {
    Open,
    Hidden(u16),
}

#[derive(Debug)]
pub(crate) struct App<'a> {
    pub(crate) state: AppState,
    pub(crate) db: Pool<Sqlite>,
    pub(crate) current_screen: Screen,
    pub(crate) prev_screen: Screen,
    pub(crate) editor: Editor<'a>,
    pub(crate) note_list: NoteList,
    pub(crate) btns: [Button; 2],
    pub(crate) btn_idx: usize,
    pub(crate) user_input: UserInput<'a>,
    pub(crate) user_msg: UserMessage,
    pub(crate) sidebar: SidebarState,
    pub(crate) sidebar_size: u16,
    pub(crate) pending_link: Option<TextAreaLink>,
    pub(crate) active_widget: Option<ActiveWidget>,
}

impl<'a> App<'a> {
    pub fn new(db: Pool<Sqlite>, note_identifiers: Vec<NoteIdentifier>) -> Self {
        let load_btn_state = match note_identifiers.len() {
            0 => ComponentState::Unavailable,
            _ => ComponentState::Inactive,
        };

        let note_list = NoteList::new(
            note_identifiers,
            NoteListAction::LoadNote,
            ComponentState::Active,
        );

        Self {
            state: AppState::default(),
            db,
            current_screen: Screen::Welcome,
            prev_screen: Screen::Welcome,
            editor: Editor::new(" Untitled ".to_owned(), vec!["".to_owned()], HashMap::new(), None),
            note_list,
            btns: [
                    Button::new(
                        "New".to_owned(),
                        ComponentState::Active,
                        ButtonAction::RenderNewNoteScreen,
                    ),
                    Button::new(
                        "Load".to_owned(),
                        load_btn_state,
                        ButtonAction::RenderLoadNoteScreen,
                    ),
            ],
            btn_idx: 0,
            user_input: UserInput::new(ComponentState::Active, InputAction::Note),
            user_msg: UserMessage::welcome(),
            sidebar: SidebarState::Open,
            sidebar_size: 20,
            pending_link: None,
            active_widget: None,
        }
    }

    pub(crate) fn set_active_widget(&mut self, active: ActiveWidget) {
        match active {
            ActiveWidget::NoteList => {
                self.user_input.set_state(ComponentState::Inactive);
                self.note_list.set_state(ComponentState::Active);
            }
            ActiveWidget::NoteTitleInput => {
                self.user_input.set_state(ComponentState::Active);
                self.note_list.set_state(ComponentState::Inactive);
            }
            ActiveWidget::Editor => {
                self.editor.set_state(ComponentState::Active);
                self.note_list.set_state(ComponentState::Inactive);
            },
            ActiveWidget::Sidebar => {
                self.editor.set_state(ComponentState::Inactive);
                self.note_list.set_state(ComponentState::Active);
            },

        }
        self.active_widget = Some(active);
    }

    pub(crate) fn current_btn(&mut self) -> &mut Button {
        &mut self.btns[self.btn_idx]
    }

    pub(crate) fn switch_to_main(&mut self) {
        self.current_screen = Screen::Main;
        self.note_list.set_mode(NoteListMode::Sidebar);
        self.active_widget = Some(ActiveWidget::Editor);
    }
    
    pub(crate) fn switch_to_load_note(&mut self) {
        self.current_screen = Screen::LoadNote;
        self.note_list.set_mode(NoteListMode::Fullscreen);
        self.active_widget = Some(ActiveWidget::NoteList);
    }

    pub(crate) fn switch_to_new_note(&mut self, action: InputAction) {
        self.current_screen = Screen::NewNote;
        self.user_input.set_action(action);
        self.active_widget = Some(ActiveWidget::NoteTitleInput);
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
