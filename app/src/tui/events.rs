use std::collections::HashMap;
use std::cmp;
use color_eyre::eyre::{Context, Result, eyre};
use crossterm::event::{self, Event, KeyEventKind};
use log::{info, error};
use tuipaz_textarea::{Input, Key};

use crate::db::db_mac::{DbMac, DbNoteLink, NoteIdentifier};

use super::{
    app::{ActiveWidget, App, AppState, Screen, SidebarState, ComponentState, SearchbarState},
    buttons::ButtonAction,
    editor::{Link, EditorMode},
    inputs::{InputAction, UserInput},
    user_messages::{MessageType, UserMessage}, note_list::{NoteListMode, NoteListAction},
};

const DELETE_KEYS: [Key; 10] = [
    Key::Char('d'),
    Key::Char('w'),
    Key::Char('b'),
    Key::Char('j'),
    Key::Char('k'),
    Key::Char('l'),
    Key::Char('h'),
    Key::Char('x'),
    Key::Delete,
    Key::Backspace,
];

#[derive(Debug, Clone, Hash)]
#[allow(clippy::derived_hash_with_manual_eq)]
pub(crate) enum Action {
    ShowExitScreen,
    PrevScreen,
    SwitchBtns,
    Quit,
    Save,
    Load,
    Delete,
    NewNote,
    NewTitle,
    OpenNoteList,
    ToggleSearchbar(Input),
    ToggleSidebar,
    IncreaseSidebar,
    DecreaseSidebar,
    InsertLink(Input),
    SwitchActiveWidget,
    Confirm,
    Cancel,
    Activate(Input),
    Up(Input),
    Down(Input),
    Next,
    Prev,
    DeleteChar,
    Edit(Input),
    Null,
}

 impl PartialEq for Action {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            // Compare variants without associated data
            (Action::ShowExitScreen, Action::ShowExitScreen) => true,
            (Action::PrevScreen, Action::PrevScreen) => true,
            (Action::SwitchBtns, Action::SwitchBtns) => true,
            (Action::Quit, Action::Quit) => true,
            (Action::Save, Action::Save) => true,
            (Action::Load, Action::Load) => true,
            (Action::Delete, Action::Delete) => true,
            (Action::NewNote, Action::NewNote) => true,
            (Action::NewTitle, Action::NewTitle) => true,
            (Action::OpenNoteList, Action::OpenNoteList) => true,
            (Action::ToggleSearchbar(_), Action::ToggleSearchbar(_)) => true,
            (Action::ToggleSidebar, Action::ToggleSidebar) => true,
            (Action::IncreaseSidebar, Action::IncreaseSidebar) => true,
            (Action::DecreaseSidebar, Action::DecreaseSidebar) => true,
            (Action::InsertLink(_), Action::InsertLink(_)) => true,
            (Action::SwitchActiveWidget, Action::SwitchActiveWidget) => true,
            (Action::Confirm, Action::Confirm) => true,
            (Action::Cancel, Action::Cancel) => true,
            (Action::Activate(_), Action::Activate(_)) => true,
            (Action::Up(_), Action::Up(_)) => true,
            (Action::Down(_), Action::Down(_)) => true,
            (Action::Next, Action::Next) => true,
            (Action::Prev, Action::Prev) => true,
            (Action::DeleteChar, Action::DeleteChar) => true,
            (Action::Edit(_), Action::Edit(_)) => true,
            (Action::Null, Action::Null) => true,

            // Handle all other cases as unequal
            _ => false,
        }
    }
}

impl Eq for Action {}

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

    async fn execute_action(app: &mut App<'_>, action: Action) -> Result<()> {
        match (app.current_screen, action) {
            (Screen::Welcome, Action::ShowExitScreen) => {
                app.prev_screen = app.current_screen;
                Self::show_exit_screen(app);
            },
            (Screen::Welcome, Action::SwitchBtns) => {
                Self::switch_btns(app);
            },
            (Screen::Welcome, Action::Activate(_)) => {
                let btn_state = app.current_btn().get_state();

                if btn_state == ComponentState::Active {
                    Self::btn_action(app);
                }
            }
            (Screen::Main, Action::ShowExitScreen) => {
                app.prev_screen = app.current_screen;
                Self::show_exit_screen(app);
            }
            (Screen::Main, Action::Save) => {
                let has_links = !matches!(app.editor.body.links.len(), 0);
                let title = app.editor.title.clone();
                let body = app.editor.body.lines().join("\n");
                let note_id = app.editor.note_id;

                Self::save_note(app, &title, &body, has_links, note_id).await?;
            }
            (Screen::Main, Action::Load) => {
                app.prev_screen = app.current_screen;
                app.switch_to_load_note();
            }
            (Screen::Main, Action::NewNote) => {
                app.prev_screen = app.current_screen;
                app.switch_to_new_note(InputAction::Note);
            }
            (Screen::Main, Action::NewTitle) => {
                app.prev_screen = app.current_screen;
                app.switch_to_new_note(InputAction::NoteTitle);
            }
            (Screen::Main, Action::Delete) => {
                app.prev_screen = app.current_screen;
                app.current_screen = Screen::DeleteNoteConfirmation;
                app.user_msg = UserMessage::new(
                    format!("Are you sure you want to delete {}? (y/n)", app.editor.title),
                    MessageType::Warning,
                    None,
                );
            }
            (Screen::Main, Action::ToggleSearchbar(input)) => {
                if app.editor.mode == EditorMode::Normal {
                    Self::toggle_searchbar(app);
                } else {
                    app.editor.handle_input(input);
                }
            }
            (Screen::Main, Action::ToggleSidebar) => {
                Self::toggle_sidebar(app);
            }
            (Screen::Main, Action::IncreaseSidebar) => {
                match app.sidebar_state {
                    SidebarState::Open => {
                        app.sidebar_size = cmp::min(app.sidebar_size + 2, 70);
                    },
                    SidebarState::Hidden(_) => {},
                }
            }
            (Screen::Main, Action::DecreaseSidebar) => {
                match app.sidebar_state {
                    SidebarState::Open => {
                        app.sidebar_size = cmp::max(app.sidebar_size - 2, 12);
                    },
                    SidebarState::Hidden(_) => {},
                }
            }
            (Screen::Main, Action::InsertLink(input)) => {
                app.editor.handle_input(input);

                // If there is a new link in the textarea
                if app.editor.body.new_link {
                    app.pending_link = Some(
                        *app.editor
                        .body
                        .links
                        .get(&(app.editor.body.next_link_id - 1))
                        .expect("Link should be present")
                    );

                    // Set the user_input widget to create a new linked note
                    app.prev_screen = app.current_screen;
                    app.current_screen = Screen::NewLinkedNote;
                    app.user_input.set_action(InputAction::LinkedNote);
                    app.user_input.set_state(ComponentState::Active);
                    app.active_widget = Some(ActiveWidget::NoteTitleInput);
                    app.note_list.set_state(ComponentState::Inactive);
                    app.note_list.set_mode(NoteListMode::Fullscreen);
                    app.note_list.set_action(NoteListAction::LinkNote);
                    app.editor.body.new_link = false;
                }
            }
           (Screen::Main, Action::SwitchActiveWidget) => match app.active_widget {
                Some(ActiveWidget::Editor) => app.set_active_widget(ActiveWidget::Sidebar),
                Some(ActiveWidget::Sidebar) => app.set_active_widget(ActiveWidget::Editor),
                Some(_) | None => {}
            }
            (Screen::Main, Action::Activate(input)) => match app.active_widget {
                Some(ActiveWidget::Editor) => {
                    match app.editor.body.in_link(app.editor.body.cursor()) {
                        Some(link_id) => {
                            let linked_note_id = app
                                .editor
                                .links
                                .values()
                                .find(|link| link.text_id == link_id as i64)
                                .expect("Link should be set up")
                                .linked_id;

                            Self::load_note(app, linked_note_id).await?;
                        }
                        None => {
                            app.editor.body.input(input);
                        }
                    }
                },
                Some(ActiveWidget::Sidebar) => {
                    let note_idx = app.note_list.selected;
                    let id = app.note_list.note_identifiers[note_idx].id;
                    Self::load_note(app, id).await?;
                },
                Some(ActiveWidget::Searchbar) => {
                    app.searchbar.clear_search();
                    Self::toggle_searchbar(app);
                },
                Some(_) | None => {},
            }
            (Screen::Main, Action::Up(input)) => match app.active_widget {
                Some(ActiveWidget::Editor) => app.editor.handle_input(input),
                Some(ActiveWidget::Sidebar) => app.note_list.prev(),
                Some(_) | None => {},
            }
            (Screen::Main, Action::Down(input)) => match app.active_widget {
                Some(ActiveWidget::Editor) => app.editor.handle_input(input),
                Some(ActiveWidget::Sidebar) => app.note_list.next(),
                Some(_) | None => {},
            }
            (Screen::Main, Action::Edit(input)) => match app.active_widget {
                Some(ActiveWidget::Editor) => {
                    app.editor.handle_input(input);

                    if let Some(key) = DELETE_KEYS.iter().find(|&&k| k == input.key) {
                        Self::check_link_deletion(app, key);
                    }

                    if input.key == Key::Char('u') || input.key == Key::Char('r') {
                        Self::check_link_edits(app);
                    }

                    if !app.editor.links.is_empty() {
                        Self::check_link_moved(app);
                    }
                },
                Some(ActiveWidget::Searchbar) => {
                    app.searchbar.input.input(input);
                    let search_pattern = &app.searchbar.get_search_text();
                    match app.editor.body.set_search_pattern(search_pattern) {
                        Ok(_) => info!("Searching for {:?}", search_pattern),
                        Err(e) => error!("Error searching for {:?}: {:?}", search_pattern, e),
                    }
                }
                Some(_) | None => {},
            }
            (Screen::NewNote, Action::ShowExitScreen) => {
                app.prev_screen = app.current_screen;
                Self::show_exit_screen(app);
            },
            (Screen::NewNote, Action::PrevScreen) => {
                app.current_screen = app.prev_screen;
                app.active_widget = Some(ActiveWidget::Editor);
            },
            (Screen::NewNote, Action::Activate(_)) => match app.user_input.get_action() {
                InputAction::NoteTitle => Self::input_new_note_title(app),
                InputAction::Note => Self::input_new_note(app, false).await?,
                _ => {}
            },
            (Screen::NewNote, Action::DeleteChar) => {
                app.user_input.text.delete_char();
                if app.user_input.get_state() == ComponentState::Error {
                    app.user_input.set_state(ComponentState::Active);
                }
            },
           (Screen::NewNote, Action::Edit(input)) => {
                app.user_input.text.input(input);
            },
            (Screen::NewLinkedNote, Action::ShowExitScreen) => {
                app.prev_screen = app.current_screen;
                Self::show_exit_screen(app);
            }
            (Screen::NewLinkedNote, Action::PrevScreen) => {
                app.current_screen = app.prev_screen;
                app.note_list.set_mode(NoteListMode::Sidebar);
                app.note_list.set_action(NoteListAction::LoadNote);
                app.active_widget = Some(ActiveWidget::Editor);
                app.editor.body.delete_link(app.editor.body.next_link_id - 1);
                
            }
            (Screen::NewLinkedNote, Action::Activate(_)) => {
                if app.active_widget == Some(ActiveWidget::NoteTitleInput) {
                    Self::input_new_note(app, true).await?;
                } else if app.active_widget == Some(ActiveWidget::NoteList) {
                    let selected = app.note_list.selected;
                    let nid = &app.note_list.note_identifiers[selected];
                    Self::link_note(app, nid.id);
                }
            }
            (Screen::NewLinkedNote, Action::Next) => {
                if app.active_widget == Some(ActiveWidget::NoteList) {
                    app.note_list.next();
                }
            }
            (Screen::NewLinkedNote, Action::Prev) => {
                if app.active_widget == Some(ActiveWidget::NoteList) {
                    app.note_list.prev();
                }
            }
            (Screen::NewLinkedNote, Action::SwitchActiveWidget) => {
                if app.active_widget == Some(ActiveWidget::NoteList) {
                    app.set_active_widget(ActiveWidget::NoteTitleInput);
                } else if app.active_widget == Some(ActiveWidget::NoteTitleInput) {
                    app.set_active_widget(ActiveWidget::NoteList);
                }
            }
            (Screen::NewLinkedNote, Action::DeleteChar) => {
                if app.active_widget == Some(ActiveWidget::NoteTitleInput) {
                    app.user_input.text.delete_char();
                    if app.user_input.get_state() == ComponentState::Error {
                        app.user_input.set_state(ComponentState::Active);
                    }
                }
            }
            (Screen::NewLinkedNote, Action::Edit(input)) => {
                if app.active_widget == Some(ActiveWidget::NoteTitleInput) {
                    app.user_input.text.input(input);
                }
            }
            (Screen::LoadNote, Action::ShowExitScreen) => {
                app.prev_screen = app.current_screen;
                Self::show_exit_screen(app);
            }
            (Screen::LoadNote, Action::PrevScreen) => {
                app.switch_to_main();
            }
            (Screen::LoadNote, Action::Next) => {
                app.note_list.next();
            }
            (Screen::LoadNote, Action::Prev) => {
                app.note_list.prev();
            }
            (Screen::LoadNote, Action::Activate(_)) => {
                let note_idx = app.note_list.selected;
                let id = app.note_list.note_identifiers[note_idx].id;
                Self::load_note(app, id).await?;
            }
            (Screen::Popup, _) => {
                if let Some(screen) = app.user_msg.next_screen {
                    app.current_screen = screen;
                } else {
                    app.current_screen = app.prev_screen;
                }
            },
            (Screen::DeleteNoteConfirmation, Action::PrevScreen) => {
                app.current_screen = app.prev_screen;
            }
            (Screen::DeleteNoteConfirmation, Action::Confirm) => {
                if let Some(note_id) = app.editor.note_id {
                    DbMac::delete_note(&app.db, note_id).await?;
                    app.note_list.remove(note_id);
                    app.current_screen = Screen::Welcome;
                } else {
                    app.current_screen = Screen::Popup;
                    app.user_msg = UserMessage::new(
                        format!("Error: couldn't delete {}", app.editor.title),
                        MessageType::Error,
                        Some(Screen::Main),
                    );
                }
            }
            (Screen::DeleteNoteConfirmation, Action::Cancel) => {
                app.current_screen = app.prev_screen;
                app.active_widget = Some(ActiveWidget::Editor);
            }
            (Screen::Exiting, Action::Confirm) => {
                Self::exit(app);
            },
            (Screen::Exiting, Action::Cancel) => {
                    app.current_screen = app.prev_screen;
                    app.active_widget = Some(ActiveWidget::Editor);
            }
            _ => {}
        }
        Ok(())
    }
    async fn handle_key_event(app: &mut App<'_>, input: Input) -> Result<()> {
        match app.current_screen {
            Screen::Welcome => match input {
                Input {
                    key: Key::Char('q'),
                    alt: true,
                    ..
                } => {
                    app.prev_screen = app.current_screen;
                    Self::show_exit_screen(app);
                }
                Input { key: Key::Tab, .. } => {
                    Self::switch_btns(app);
                }
                Input {
                    key: Key::Enter, ..
                } => {
                    let btn_state = app.current_btn().get_state();

                    if btn_state == ComponentState::Active {
                        Self::btn_action(app);
                    }
                }
                _ => {}
            },
            Screen::Main => match input {
                Input {
                    key: Key::Char('q'),
                    alt: true,
                    ..
                } => {
                    app.prev_screen = app.current_screen;
                    Self::show_exit_screen(app);
                }
                Input {
                    key: Key::Char('s'),
                    alt: true,
                    ..
                } => {
                    let has_links = !matches!(app.editor.body.links.len(), 0);
                    let title = app.editor.title.clone();
                    let body = app.editor.body.lines().join("\n");
                    let note_id = app.editor.note_id;

                    Self::save_note(app, &title, &body, has_links, note_id).await?;
                }
                Input {
                    key: Key::Char('l'),
                    alt: true,
                    ..
                } => {
                    app.prev_screen = app.current_screen;
                    app.switch_to_load_note();
                }
                Input {
                    key: Key::Char('n'),
                    alt: true,
                    ..
                } => {
                    app.prev_screen = app.current_screen;
                    app.switch_to_new_note(InputAction::Note);
                }
                Input {
                    key: Key::Char('t'),
                    alt: true,
                    ..
                } => {
                    app.prev_screen = app.current_screen;
                    app.switch_to_new_note(InputAction::NoteTitle);
                }
                Input {
                    key: Key::Char('d'),
                    alt: true,
                    ..
                } => {
                    app.prev_screen = app.current_screen;
                    app.current_screen = Screen::DeleteNoteConfirmation;
                    app.user_msg = UserMessage::new(
                        format!("Are you sure you want to delete {}? (y/n)", app.editor.title),
                        MessageType::Warning,
                        None,
                    );
                }
                Input {
                    key: Key::Char('/'),
                    ..
                } => {
                    if app.editor.mode == EditorMode::Normal {
                        Self::toggle_searchbar(app);
                    } else {
                        app.editor.handle_input(input);
                    }
                }
                Input {
                    key: Key::Char('f'),
                    alt: true,
                    ..
                } => {
                    Self::toggle_sidebar(app);
                }
                Input {
                    key: Key::Char(','),
                    alt: true,
                    ..
                } => {
                    match app.sidebar_state {
                        SidebarState::Open => {
                            app.sidebar_size = cmp::min(app.sidebar_size + 2, 70);
                        },
                        SidebarState::Hidden(_) => {},
                    }
                }
                Input {
                    key: Key::Char('.'),
                    alt: true,
                    ..
                } => {
                    match app.sidebar_state {
                        SidebarState::Open => {
                            app.sidebar_size = cmp::max(app.sidebar_size - 2, 12);
                        },
                        SidebarState::Hidden(_) => {},
                    }
                }
                Input {
                    key: Key::Char(']'),
                    ..
                } => {
                    app.editor.handle_input(input);

                    // If there is a new link in the textarea
                    if app.editor.body.new_link {
                        app.pending_link = Some(
                            *app.editor
                                .body
                                .links
                                .get(&(app.editor.body.next_link_id - 1))
                                .expect("Link should be present")
                        );

                        // Set the user_input widget to create a new linked note
                        app.prev_screen = app.current_screen;
                        app.current_screen = Screen::NewLinkedNote;
                        app.user_input.set_action(InputAction::LinkedNote);
                        app.user_input.set_state(ComponentState::Active);
                        app.active_widget = Some(ActiveWidget::NoteTitleInput);
                        app.note_list.set_state(ComponentState::Inactive);
                        app.note_list.set_mode(NoteListMode::Fullscreen);
                        app.note_list.set_action(NoteListAction::LinkNote);
                        app.editor.body.new_link = false;
                    }
                }
                Input {
                    key: Key::Tab, alt: true, ..
                } => match app.active_widget {
                    Some(ActiveWidget::Editor) => app.set_active_widget(ActiveWidget::Sidebar),
                    Some(ActiveWidget::Sidebar) => app.set_active_widget(ActiveWidget::Editor),
                    Some(_) | None => {}
                }
                Input {
                    key: Key::Enter, ..
                }  => match app.active_widget {
                        Some(ActiveWidget::Editor) => {
                            match app.editor.body.in_link(app.editor.body.cursor()) {
                                Some(link_id) => {
                                    let linked_note_id = app
                                        .editor
                                        .links
                                        .values()
                                        .find(|link| link.text_id == link_id as i64)
                                        .expect("Link should be set up")
                                        .linked_id;

                                    Self::load_note(app, linked_note_id).await?;
                                }
                                None => {
                                    app.editor.body.input(input);
                                }
                            }
                        },
                        Some(ActiveWidget::Sidebar) => {
                            let note_idx = app.note_list.selected;
                            let id = app.note_list.note_identifiers[note_idx].id;
                            Self::load_note(app, id).await?;
                        },
                        Some(ActiveWidget::Searchbar) => {
                            app.searchbar.clear_search();
                            Self::toggle_searchbar(app);
                        },
                        Some(_) | None => {},
                }
                Input {
                    key: Key::Up, ..
                } => match app.active_widget {
                        Some(ActiveWidget::Editor) => app.editor.handle_input(input),
                        Some(ActiveWidget::Sidebar) => app.note_list.prev(),
                        Some(_) | None => {},
                }
                Input {
                    key: Key::Down, ..
                } => match app.active_widget {
                        Some(ActiveWidget::Editor) => app.editor.handle_input(input),
                        Some(ActiveWidget::Sidebar) => app.note_list.next(),
                        Some(_) | None => {},
                }
                input => match app.active_widget {
                        Some(ActiveWidget::Editor) => {
                            app.editor.handle_input(input);
                            
                            if let Some(key) = DELETE_KEYS.iter().find(|&&k| k == input.key) {
                                Self::check_link_deletion(app, key);
                            }

                            if input.key == Key::Char('u') || input.key == Key::Char('r') {
                                Self::check_link_edits(app);
                            }

                            if !app.editor.links.is_empty() {
                                Self::check_link_moved(app);
                            }
                        },
                        Some(ActiveWidget::Searchbar) => {
                            app.searchbar.input.input(input);
                            let search_pattern = &app.searchbar.get_search_text();
                            match app.editor.body.set_search_pattern(search_pattern) {
                                Ok(_) => info!("Searching for {:?}", search_pattern),
                                Err(e) => error!("Error searching for {:?}: {:?}", search_pattern, e),
                            }
                        }
                        Some(_) | None => {},
                }
            },
            Screen::NewNote => match input {
                Input {
                    key: Key::Char('q'),
                    alt: true,
                    ..
                } => {
                    app.prev_screen = app.current_screen;
                    Self::show_exit_screen(app);
                }
                Input { key: Key::Esc, .. } => {
                    app.current_screen = app.prev_screen;
                    app.active_widget = Some(ActiveWidget::Editor);
                }
                Input {
                    key: Key::Enter, ..
                } => match app.user_input.get_action() {
                    InputAction::NoteTitle => Self::input_new_note_title(app),
                    InputAction::Note => Self::input_new_note(app, false).await?,
                    _ => {}
                },
                Input {
                    key: Key::Backspace,
                    ..
                } => {
                    app.user_input.text.delete_char();
                    if app.user_input.get_state() == ComponentState::Error {
                        app.user_input.set_state(ComponentState::Active);
                    }
                }
                input => {
                    app.user_input.text.input(input);
                }
            },
            Screen::NewLinkedNote => match input {
                Input {
                    key: Key::Char('q'),
                    alt: true,
                    ..
                } => {
                    app.prev_screen = app.current_screen;
                    Self::show_exit_screen(app);
                }
                Input { key: Key::Esc, .. } => {
                    app.current_screen = app.prev_screen;
                    app.note_list.set_mode(NoteListMode::Sidebar);
                    app.note_list.set_action(NoteListAction::LoadNote);
                    app.active_widget = Some(ActiveWidget::Editor);
                    app.editor.body.delete_link(app.editor.body.next_link_id - 1);
                    
                }
                Input {
                    key: Key::Enter, ..
                } => {
                    if app.active_widget == Some(ActiveWidget::NoteTitleInput) {
                        Self::input_new_note(app, true).await?;
                    } else if app.active_widget == Some(ActiveWidget::NoteList) {
                        let selected = app.note_list.selected;
                        let nid = &app.note_list.note_identifiers[selected];
                        Self::link_note(app, nid.id);
                    }
                }
                Input { key: Key::Down, .. } => {
                    if app.active_widget == Some(ActiveWidget::NoteList) {
                        app.note_list.next();
                    }
                }
                Input { key: Key::Up, .. } => {
                    if app.active_widget == Some(ActiveWidget::NoteList) {
                        app.note_list.prev();
                    }
                }
                Input { key: Key::Tab, .. } => {
                    if app.active_widget == Some(ActiveWidget::NoteList) {
                        app.set_active_widget(ActiveWidget::NoteTitleInput);
                    } else if app.active_widget == Some(ActiveWidget::NoteTitleInput) {
                        app.set_active_widget(ActiveWidget::NoteList);
                    }
                }
                Input {
                    key: Key::Backspace,
                    ..
                } => {
                    if app.active_widget == Some(ActiveWidget::NoteTitleInput) {
                        app.user_input.text.delete_char();
                        if app.user_input.get_state() == ComponentState::Error {
                            app.user_input.set_state(ComponentState::Active);
                        }
                    }
                }
                input => {
                    if app.active_widget == Some(ActiveWidget::NoteTitleInput) {
                        app.user_input.text.input(input);
                    }
                }
            },
            Screen::LoadNote => match input {
                Input {
                    key: Key::Char('q'),
                    alt: true,
                    ..
                } => {
                    app.prev_screen = app.current_screen;
                    Self::show_exit_screen(app);
                }
                Input { key: Key::Esc, .. } => {
                    app.switch_to_main();
                }
                Input { key: Key::Down, .. }
                | Input {
                    key: Key::Char('j'),
                    ..
                } => {
                    app.note_list.next();
                }
                Input { key: Key::Up, .. }
                | Input {
                    key: Key::Char('k'),
                    ..
                } => {
                    app.note_list.prev();
                }
                Input {
                    key: Key::Enter, ..
                } => {
                    let note_idx = app.note_list.selected;
                    let id = app.note_list.note_identifiers[note_idx].id;
                    Self::load_note(app, id).await?;
                }
                _ => {}
            },
            Screen::Popup => if let Input { key: Key::Esc, .. } = input {
                if let Some(screen) = app.user_msg.next_screen {
                    app.current_screen = screen;
                } else {
                    app.current_screen = app.prev_screen;
                }
            },
            Screen::DeleteNoteConfirmation => match input {
                Input {
                    key: Key::Esc,
                    ..
                } => {
                    app.current_screen = app.prev_screen;
                }
                Input {
                    key: Key::Char('y'),
                    ..
                } => {
                    if let Some(note_id) = app.editor.note_id {
                        DbMac::delete_note(&app.db, note_id).await?;
                        app.note_list.remove(note_id);
                        app.current_screen = Screen::Welcome;
                    } else {
                        app.current_screen = Screen::Popup;
                        app.user_msg = UserMessage::new(
                            format!("Error: couldn't delete {}", app.editor.title),
                            MessageType::Error,
                            Some(Screen::Main),
                        );
                    }
                }
                Input {
                    key: Key::Char('n'),
                    ..
                } => {
                    app.current_screen = app.prev_screen;
                    app.active_widget = Some(ActiveWidget::Editor);
                }
                _ => {}
            }
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
                    app.current_screen = app.prev_screen;
                    app.active_widget = Some(ActiveWidget::Editor);
                }
                _ => {}
            },
        };
        Ok(())
    }

    async fn save_note(
        app: &mut App<'_>,
        title: &str,
        body: &str,
        has_links: bool,
        note_id: Option<i64>,
    ) -> Result<()> {
        let (save_note_result, updated) = match note_id {
            Some(id) => (
                DbMac::update_note(&app.db, title, body, has_links, id).await,
                true,
            ),
            None => (
                DbMac::save_note(&app.db, title, body, has_links).await,
                false,
            ),
        };

        info!("fn save_note\nupdated: {:?}\nsave_note_result: {:?}", updated, save_note_result);

        match save_note_result {
            Ok(parent_id) => {
                if updated {
                    let new_nid = NoteIdentifier {
                        id: parent_id,
                        title: title.to_owned(),
                    };

                    // Replaces prev note title with new one in the load note screen
                    app.note_list.replace(new_nid);
                } else {
                    let new_nid = NoteIdentifier {
                        id: parent_id,
                        title: title.to_owned(),
                    };

                    // Makes the note available in the load note screen
                    app.note_list.update(new_nid);
                    // Triggers update_note on next save
                    app.editor.note_id = Some(parent_id);
                }
                
                match has_links {
                    true => {
                        let sync_db_links_result = Self::sync_db_links(app).await;
                        info!("sync_db_links_result: {:?}", sync_db_links_result);
                        match sync_db_links_result {
                            Ok(_) => {
                                app.user_msg = UserMessage::new(
                                    "Note saved!".to_string(),
                                    MessageType::Info,
                                    None,
                                );
                                app.prev_screen = app.current_screen;
                                app.current_screen = Screen::Popup;
                                // Don't resave the same links over and over
                                for link in app.editor.links.values_mut() {
                                    if !link.saved {
                                        link.saved = true;
                                    }
                                }
                                Ok(())
                            }
                            Err(err) => {
                                app.user_msg = UserMessage::new(
                                    format!("Error saving note links!: {:?}", err),
                                    MessageType::Error,
                                    None,
                                );
                                app.prev_screen = app.current_screen;
                                app.current_screen = Screen::Popup;
                                Err(err)
                            }
                        }
                    }   
                    false => {
                        app.user_msg = UserMessage::new(
                            "Note saved!".to_string(),
                            MessageType::Info,
                            None,
                        );
                        app.prev_screen = app.current_screen;
                        app.current_screen = Screen::Popup;
                        Ok(())
                    }
                }
            }
            Err(err) => {
                app.user_msg = UserMessage::new(
                    format!("Error saving note!: {:?}", err),
                    MessageType::Error,
                    None,
                );
                app.prev_screen = app.current_screen;
                app.current_screen = Screen::Popup;
                Err(err)
            }
        }
    }
    
    fn check_links_to_update(link: &&Link) -> bool {
        !link.deleted && link.saved && link.updated
    }

    fn check_links_to_save(link: &&Link) -> bool {
        !link.deleted && !link.saved
    }

    fn check_links_to_delete(link: &&Link) -> bool {
        link.deleted && link.saved
    }

    async fn update_links(tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>, app: &mut App<'_>) -> Result<()> {
        let links_to_update = app
            .editor
            .links
            .values()
            .filter(Self::check_links_to_update)
            .map(|link| link.to_db_link())
            .collect::<Vec<DbNoteLink>>();
        
        info!("fn update_links\nlinks_to_update: {:?}", links_to_update);

        if !links_to_update.is_empty() {
            DbMac::update_links(tx, &app.db, links_to_update, app.editor.note_id.expect("Note should have an id")).await?
        }

        Ok(())
    }

    async fn save_links(tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>, app: &mut App<'_>) -> Result<()> {
        let links_to_save = app
            .editor
            .links
            .values()
            .filter(Self::check_links_to_save)
            .map(|link| link.to_db_link())
            .collect::<Vec<DbNoteLink>>();
        
        info!("fn save_links\nlinks_to_save: {:?}", links_to_save);

        if !links_to_save.is_empty() {
            DbMac::save_links(tx, &app.db, links_to_save, app.editor.note_id.expect("Note should have an id")).await?
        }

        Ok(())
    }

    async fn delete_links(tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>, app: &mut App<'_>) -> Result<()> {
        let parent_note_id = app.editor.note_id.expect("Note should have an id");

        let links_to_delete = app
            .editor
            .links
            .values()
            .filter(Self::check_links_to_delete)
            .map(|link| (parent_note_id, link.text_id))
            .collect::<Vec<(i64, i64)>>();
        
        info!("fn delete_links\nlinks_to_delete: {:?}", links_to_delete);

        if !links_to_delete.is_empty() {
            DbMac::delete_links(tx, &app.db, links_to_delete).await?
        }

        Ok(())
    }

    async fn sync_db_links(
        app: &mut App<'_>, 
    ) -> Result<()> {
        let mut tx = app.db.begin().await?;
        info!("fn sync_db_links\neditor links: {:?}", app.editor.links);
        let update_links_result = Self::update_links(&mut tx, app).await;
        let save_links_result = Self::save_links(&mut tx, app).await;
        let delete_links_result = Self::delete_links(&mut tx, app).await;
        info!("update_links_result: {:?}\nsave_links_result: {:?}\ndelete_links_result: {:?}",
            update_links_result, save_links_result, delete_links_result
        );

        match (save_links_result, delete_links_result, update_links_result) {
            (Ok(_), Ok(_), Ok(_)) => {
                tx.commit().await?;
                Ok(())
            },
            (Err(se), Ok(_), Ok(_)) =>{
                tx.rollback().await?;
                Err(eyre!("Transaction error::update_links: {:?}", se))
            },
            (Ok(_), Err(de), Ok(_)) => {
                tx.rollback().await?;
               Err(eyre!("Transaction error::delete_links: {:?}", de))
            },
            (Ok(_), Ok(_), Err(ue)) => {
                tx.rollback().await?;
                Err(eyre!("Transaction error::update_links: {:?}", ue))
            },
            (Err(se), Err(de), Ok(_)) =>{
                tx.rollback().await?;
                Err(eyre!(
                    "Transaction errors\n
                    save_links: {:?}\n
                    delete_links: {:?}",
                    se, de)
                )
            },
            (Ok(_), Err(de), Err(ue)) =>{
                tx.rollback().await?;
                Err(eyre!(
                    "Transaction errors\n
                    delete_links: {:?}\n
                    update_links: {:?}",
                    de, ue)
                )
             },
            (Err(se), Ok(_), Err(ue)) =>{
                tx.rollback().await?;
                Err(eyre!(
                    "Transaction errors\n
                    save_links: {:?}\n
                    update_links: {:?}",
                    se, ue)
                )
             },
            (Err(se), Err(de), Err(ue)) =>{
                tx.rollback().await?;
                Err(eyre!(
                    "Transaction errors\n
                    save_links: {:?}\n
                    delete_links: {:?}\n
                    update_links: {:?}",
                    se, de, ue)
                )
             },
        }
    }

    async fn load_note(app: &mut App<'_>, id: i64) -> Result<()> {
        let result = DbMac::load_note(&app.db, id).await;

        match result {
            Ok(note) => {
                let body = match note.body {
                    Some(text) => text
                        .split('\n')
                        .map(|line| line.to_owned())
                        .collect::<Vec<String>>(),
                    None => vec!["".to_owned()],
                };

                let db_links = match note.has_links {
                    true => DbMac::load_note_links(&app.db, id).await?,
                    false => vec![],
                };

                info!("load_note::db_links: {:?}", db_links);

                let links = match db_links.len() {
                    0 => HashMap::new(),
                    _ => db_links
                        .into_iter()
                        .map(|link| (link.textarea_id, Link::from_db_link(link)))
                        .collect::<HashMap<i64, Link>>(),
                };

                info!("load_note::links for editor: {:?}", links);
                
                // Save current note before loading new one
                let has_links = !matches!(app.editor.links.len(), 0);
                let sync_title = &app.editor.title.clone();
                let sync_body = &app.editor.body.lines().join("\n");
                let sync_note_db_result = if sync_title != " Untitled " {
                    info!("saving note with title: {:?}", sync_title);
                    Self::save_note(
                        app,
                        sync_title,
                        sync_body,
                        has_links,
                        app.editor.note_id,
                    ).await
                } else {
                    Ok(())
                };

                match sync_note_db_result {
                    Ok(_) => {
                        app.editor.refresh(
                            note.title, 
                            body, 
                            links, 
                            Some(note.id), 
                            app.max_col,
                        );

                        app.switch_to_main();
                        Ok(())
                    },
                    Err(err) => {
                        app.user_msg = UserMessage::new(
                            format!("Error saving current note!: {:?}", err),
                            MessageType::Error,
                            None,
                        );
                        app.prev_screen = app.current_screen;
                        app.current_screen = Screen::Popup;
                        Err(err)
                    },
                }
            }
            Err(err) => {
                app.user_msg = UserMessage::new(
                    format!("Error saving note links!: {:?}", err),
                    MessageType::Error,
                    None,
                );
                app.prev_screen = app.current_screen;
                app.current_screen = Screen::Popup;
                Err(err)
            }
        }
    }

    fn switch_btns(app: &mut App) {
        if app.current_btn().get_state() != ComponentState::Unavailable {
            app.current_btn().set_state(ComponentState::Inactive);
        }

        app.btn_idx = (app.btn_idx + 1) % app.btns.len();

        if app.current_btn().get_state() != ComponentState::Unavailable {
            app.current_btn().set_state(ComponentState::Active);
        }
    }

    fn btn_action(app: &mut App) {
        match app.btns[app.btn_idx].get_action() {
            ButtonAction::RenderMainScreen => {
                app.switch_to_main();
            }
            ButtonAction::RenderNewNoteScreen => {
                app.user_input = UserInput::new(ComponentState::Active, InputAction::Note);
                app.switch_to_new_note(InputAction::Note);
            }
            ButtonAction::RenderLoadNoteScreen => {
                app.switch_to_load_note();
            }
        }
    }

    async fn input_new_note<'a>(app: &mut App<'a>, linked: bool) -> Result<()> {
        let linked_title = app.user_input.text.lines()[0].clone();

        match app
            .note_list
            .note_identifiers
            .iter()
            .any(|nid| nid.title == linked_title)
        {
            // If any pre-exisiting notes have that title, warn user with input error state
            true => {
                app.user_input.set_state(ComponentState::Error);
                Ok(())
            }
            // If no pre-exisiting notes have that title, create and save new note with that title
            false => {
                let linked_body = "";
                let result =
                    DbMac::save_note(&app.db, &linked_title, linked_body, false)
                        .await;

                match result {
                    Ok(id) => {
                        let new_nid = NoteIdentifier {
                            id,
                            title: linked_title.clone(),
                        };

                        if linked {
                            // link the new note to the parent
                            Self::link_note(app, new_nid.id);

                            // Save parent note to preserve link in textarea
                            let parent_title = app.editor.title.clone();
                            let parent_body = app.editor.body.lines().join("\n");
                            let has_links = true;
                            let note_id = app.editor.note_id;
                            let parent_result =
                                Self::save_note(app, &parent_title, &parent_body, has_links, note_id)
                                    .await;

                            match parent_result {
                                Ok(_) => {
                                    // If parent note saved correctly, switch editor to linked) note
                                    app.editor.refresh(
                                        linked_title,
                                        vec![linked_body.to_owned()],
                                        HashMap::new(),
                                        Some(id),
                                        app.get_max_col(),
                                    );
                                    app.note_list.update(new_nid);
                                    app.switch_to_main();
                                    Ok(())
                                }
                                Err(err) => Err(err),
                            }
                        } else {
                            // if not linked to another note, simply switch to editor with new note
                            app.editor.refresh(
                                linked_title,
                                vec![linked_body.to_owned()],
                                HashMap::new(),
                                Some(id),
                                app.get_max_col(),
                            );

                            app.note_list.update(new_nid);
                            app.switch_to_main();
                            Ok(())
                        }
                    }
                    Err(err) => Err(err),
                }
            }
        }
    }

    fn link_note(app: &mut App, linked_id: i64) {
        let textarea_link = app
            .pending_link
            .expect("Should be a pending link if we reached this far");

        let parent_nid = app
            .note_list
            .note_identifiers
            .iter()
            .find(|nid| nid.title == app.editor.title)
            .expect("Parent note should already be saved and added to app.note_list");

        let new_link = Link {
            id: parent_nid.id,
            text_id: textarea_link.id as i64,
            linked_id,
            row: textarea_link.row,
            start_col: textarea_link.start_col,
            end_col: textarea_link.end_col,
            saved: false,
            updated: false,
            deleted: false,
        };
        
        let link_id = new_link.text_id;
        app.editor.links.insert(link_id, new_link);
        app.switch_to_main();
        app.editor.body.new_link = false;
    }

    fn check_link_edits(app: &mut App<'_>) {
        for link in app.editor.links.values_mut() {
            let ta_link = app.editor.body.links.get(&(link.text_id as usize))
                .expect("editor links and textarea links should be synced");
            link.deleted = ta_link.deleted;
        }
    }

    fn check_link_deletion(app: &mut App<'_>, key: &Key) {
        let delete_amount = app.editor.body.deleted_link_ids.len();
        
        if DELETE_KEYS.contains(key) && delete_amount > 0 {
            for _ in 0..delete_amount {
                let textarea_id = app.editor
                    .body
                    .deleted_link_ids
                    .pop()
                    .expect("Link to delete should exist");
                
                // guards against cases where link hasn't been saved to editor yet
                if !app.editor.links.is_empty() {
                    let ta_id_int = textarea_id as i64;
                    let link = app.editor.links.get_mut(&ta_id_int)
                        .expect("editor and textarea links should be synced");
                    link.deleted = true;
                    app.editor.deleted_link_ids.push(ta_id_int);
                }
            }
        }
    }

    fn check_link_moved(app: &mut App) {
        for link in app.editor.links.values_mut() {
            let ta_link = app.editor.body.links
                .get(&(link.text_id as usize))
                .expect("Same links should be present in editor and textarea");
            
            if link.moved(ta_link) {
                link.row = ta_link.row;
                link.start_col = ta_link.start_col;
                link.end_col = ta_link.end_col;
                link.updated = true;
            }
        }
    }

    fn input_new_note_title(app: &mut App) {
        let title = app.user_input.text.lines()[0].clone();

        match app
            .note_list
            .note_identifiers
            .iter()
            .any(|nid| nid.title == title)
        {
            true => {
                app.user_input.set_state(ComponentState::Error);
            }
            false => {
                app.editor.set_title(title);
                app.switch_to_main();
            }
        }
    }
    
    fn toggle_searchbar(app: &mut App) {
        match app.searchbar_state {
            SearchbarState::Open => {
                app.searchbar_state = SearchbarState::Hidden;
                app.editor.searchbar_open = false;
                app.set_active_widget(ActiveWidget::Editor);
                app.searchbar.state = ComponentState::Inactive;
            }
            SearchbarState::Hidden => {
                app.searchbar_state = SearchbarState::Open;
                app.editor.searchbar_open = true;
                app.set_active_widget(ActiveWidget::Searchbar);
                app.searchbar.state = ComponentState::Active;
            }
        }
    }

    fn toggle_sidebar(app: &mut App) {
        match app.sidebar_state {
            SidebarState::Open => {
                app.sidebar_state = SidebarState::Hidden(app.sidebar_size);
                app.sidebar_size = 0;
                app.editor.sidebar_open = false;
                app.searchbar.sidebar_open = false;
                match app.editor.searchbar_open {
                    true => app.set_active_widget(ActiveWidget::Searchbar),
                    false => app.set_active_widget(ActiveWidget::Editor),
                }
            }
            SidebarState::Hidden(n) => {
                app.sidebar_size = n;
                app.sidebar_state = SidebarState::Open;
                app.editor.sidebar_open = true;
                app.searchbar.sidebar_open = true;
                app.set_active_widget(ActiveWidget::Sidebar);
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
