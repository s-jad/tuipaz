use std::collections::HashMap;

use color_eyre::eyre::{Context, Result};
use log::info;
use sqlx::{Pool, Sqlite};

use crate::db::db_mac::NoteIdentifier;
use tuipaz_textarea::{Link as TextAreaLink, Input};

use super::{
    buttons::{Button, ButtonAction},
    editor::{Editor, EditorTheme},
    events::{Events, Action},
    inputs::{InputAction, UserInput},
    note_list::{NoteList, NoteListAction, NoteListMode, NoteListTheme, SelectionStyle},
    ui::ui,
    user_messages::UserMessage,
    utils::Tui, searchbar::{Searchbar, SearchbarTheme}, config::Config,
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
    Searchbar,
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum SearchbarState {
    Open,
    Hidden,
}

#[derive(Debug)]
pub(crate) struct App<'a> {
    pub(crate) state: AppState,
    pub(crate) keymap: HashMap<Action, Input>,
    pub(crate) db: Pool<Sqlite>,
    pub(crate) current_screen: Screen,
    pub(crate) prev_screen: Screen,
    pub(crate) editor: Editor<'a>,
    pub(crate) note_list: NoteList,
    pub(crate) btns: [Button; 2],
    pub(crate) btn_idx: usize,
    pub(crate) user_input: UserInput<'a>,
    pub(crate) user_msg: UserMessage,
    pub(crate) sidebar_state: SidebarState,
    pub(crate) sidebar_size: u16,
    pub(crate) searchbar: Searchbar<'a>,
    pub(crate) searchbar_state: SearchbarState,
    pub(crate) pending_link: Option<TextAreaLink>,
    pub(crate) active_widget: Option<ActiveWidget>,
    pub(crate) max_col: u16,
}

impl<'a> App<'a> {
    pub fn new(config: Config, db: Pool<Sqlite>, note_identifiers: Vec<NoteIdentifier>, term_size: u16) -> Self {
        let load_btn_state = match note_identifiers.len() {
            0 => ComponentState::Unavailable,
            _ => ComponentState::Inactive,
        };
        
        let max_col = term_size - 4;

        let editor_theme = EditorTheme {
            title: config.theme.note_title,
            text: config.theme.text,
            borders: config.theme.borders,
            normal_mode: config.theme.modes.normal_mode,
            insert_mode: config.theme.modes.insert_mode,
            visual_mode: config.theme.modes.visual_mode,
            select: config.theme.highlights.select,
            search: config.theme.highlights.search,
            links: config.theme.highlights.links,
            main_heading: config.theme.headings.main_color,
            main_heading_modifiers: config.theme.headings.main_modifiers,
            sub_heading: config.theme.headings.sub_color,
            sub_heading_modifiers: config.theme.headings.sub_modifiers,
        };

        let search_theme = SearchbarTheme {
            text: config.theme.text, 
            search_mode: config.theme.modes.search_mode,
            borders: config.theme.borders,
        };

        let note_list_theme = NoteListTheme {
            text: config.theme.text,
            title: config.theme.note_title,
            selection_style: SelectionStyle {
                highlight: config.theme.notelist.selection_highlight,
                pointer: config.theme.notelist.selection_symbol.to_owned(),
                modifier: config.theme.notelist.selection_modifier,
            },
            borders: config.theme.borders,
        };

        let note_list = NoteList::new(
            note_identifiers,
            NoteListAction::LoadNote,
            ComponentState::Active,
            note_list_theme,
        );

        Self {
            state: AppState::default(),
            keymap: config.keymap,
            db,
            current_screen: Screen::Welcome,
            prev_screen: Screen::Welcome,
            editor: Editor::new(
                " Untitled ".to_owned(),
                vec!["".to_owned()],
                HashMap::new(),
                None,
                false,
                max_col,
                editor_theme,
            ),
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
            sidebar_state: SidebarState::Hidden(18),
            sidebar_size: 0,
            searchbar: Searchbar::new(false, ComponentState::Inactive, max_col, search_theme),
            searchbar_state: SearchbarState::Hidden,
            pending_link: None,
            active_widget: None,
            max_col,
        }
    }

    pub(crate) fn set_active_widget(&mut self, active: ActiveWidget) {
        info!("set_active_widget::active: {:?}", active);
        match active {
            ActiveWidget::NoteList => {
                self.note_list.set_state(ComponentState::Active);
                self.user_input.set_state(ComponentState::Inactive);
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
                self.note_list.set_state(ComponentState::Active);
                self.editor.set_state(ComponentState::Inactive);
            },
            ActiveWidget::Searchbar => {
                self.searchbar.set_state(ComponentState::Active);
                self.editor.set_state(ComponentState::Active);
                self.note_list.set_state(ComponentState::Inactive);
            }
        }
        self.active_widget = Some(active);
    }

    pub(crate) fn current_btn(&mut self) -> &mut Button {
        &mut self.btns[self.btn_idx]
    }

    pub(crate) fn switch_to_main(&mut self) {
        self.current_screen = Screen::Main;
        self.note_list.set_mode(NoteListMode::Sidebar);
        match self.active_widget {
            Some(active) => match active {
                ActiveWidget::NoteList
                | ActiveWidget::NoteTitleInput
                | ActiveWidget::Searchbar => self.set_active_widget(ActiveWidget::Editor),
                _ => {}
            },
            None => self.set_active_widget(ActiveWidget::Editor),
        }
    }
    
    pub(crate) fn switch_to_load_note(&mut self) {
        self.current_screen = Screen::LoadNote;
        self.note_list.set_mode(NoteListMode::Fullscreen);
        self.set_active_widget(ActiveWidget::NoteList);
    }

    pub(crate) fn switch_to_new_note(&mut self, action: InputAction) {
        self.current_screen = Screen::NewNote;
        self.user_input.set_action(action);
        self.set_active_widget(ActiveWidget::NoteTitleInput);
    }

    pub(crate) fn switch_to_prev_screen(&mut self) {
        self.current_screen = self.prev_screen;
    }

    pub(crate) fn get_max_col(&self) -> u16 {
        self.max_col - self.sidebar_size
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
