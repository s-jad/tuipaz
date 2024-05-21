use color_eyre::eyre::{Context, Result};
use crossterm::event::{self, Event, KeyEventKind};
use tuipaz_textarea::{Input, Key};

use crate::db::db_mac::{DbMac, DbNoteLink, NoteIdentifier};

use super::{
    app::{ActiveWidget, App, AppState, Screen, SidebarState},
    buttons::{ButtonAction, ButtonState},
    editor::{Editor, Link},
    inputs::{InputAction, InputState, UserInput},
    note_list::{self, NoteListState},
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
                Input {
                    key: Key::Char('q'),
                    alt: true,
                    ..
                } => {
                    Self::show_exit_screen(app);
                }
                Input { key: Key::Tab, .. } => {
                    Self::switch_btns(app);
                }
                Input {
                    key: Key::Enter, ..
                } => {
                    let btn_state = app
                        .btns
                        .get(&app.btn_idx)
                        .expect("btn should exist")
                        .get_state();

                    match btn_state {
                        ButtonState::Active => Self::btn_action(app),
                        _ => {}
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
                    Self::show_exit_screen(app);
                }
                Input {
                    key: Key::Char('s'),
                    alt: true,
                    ..
                } => {
                    let has_links = match app.editor.body.links.len() {
                        0 => false,
                        _ => true,
                    };
                    let title = app.editor.title.clone();
                    let body = app.editor.body.lines().join("\n");
                    let note_id = app.editor.note_id;

                    Self::save_note(app, title, body, has_links, note_id).await?;
                }
                Input {
                    key: Key::Char('l'),
                    alt: true,
                    ..
                } => {
                    app.current_screen = Screen::LoadNote;
                    app.active_widget = Some(ActiveWidget::NoteList);
                }
                Input {
                    key: Key::Char('n'),
                    alt: true,
                    ..
                } => {
                    app.current_screen = Screen::NewNote;
                    app.user_input.set_action(InputAction::NewNote);
                    app.active_widget = Some(ActiveWidget::NoteTitleInput);
                }
                Input {
                    key: Key::Char('t'),
                    alt: true,
                    ..
                } => {
                    app.current_screen = Screen::NewNote;
                    app.user_input.set_action(InputAction::NewNoteTitle);
                    app.active_widget = Some(ActiveWidget::NoteTitleInput);
                }
                Input {
                    key: Key::Char('f'),
                    alt: true,
                    ..
                } => {
                    Self::toggle_sidebar(app);
                }
                Input {
                    key: Key::Char('['),
                    ..
                }
                | Input {
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
                                .last()
                                .expect("Link should be present"),
                        );

                        // Set the user_input widget to create a new linked note
                        app.user_input.set_action(InputAction::NewLinkedNote);
                        app.current_screen = Screen::NewLinkedNote;
                        app.active_widget = Some(ActiveWidget::NoteTitleInput);
                        app.user_input.set_state(InputState::Active);
                        app.note_list.set_state(NoteListState::Inactive);
                    }
                }
                Input {
                    key: Key::Enter, ..
                } => match app.editor.body.in_link(app.editor.body.cursor()) {
                    Some((_, link_id)) => {
                        let linked_note_id = app
                            .editor
                            .links
                            .iter()
                            .find(|link| link.text_id == link_id as i64)
                            .expect("Link should be set up")
                            .linked_id;

                        Self::load_note(app, linked_note_id as i64).await?;
                    }
                    None => {
                        app.editor.body.input(input);
                    }
                },
                input => {
                    app.editor.handle_input(input);
                }
            },
            Screen::NewNote => match input {
                Input {
                    key: Key::Char('q'),
                    alt: true,
                    ..
                } => {
                    Self::show_exit_screen(app);
                }
                Input { key: Key::Esc, .. } => {
                    app.current_screen = Screen::Main;
                    app.active_widget = Some(ActiveWidget::Editor);
                }
                Input {
                    key: Key::Enter, ..
                } => match app.user_input.get_action() {
                    InputAction::NewNoteTitle => Self::input_new_note_title(app),
                    InputAction::NewNote => Self::input_new_note(app, false).await?,
                    _ => {}
                },
                Input {
                    key: Key::Backspace,
                    ..
                } => {
                    app.user_input.text.delete_char();
                    if app.user_input.get_state() == InputState::Error {
                        app.user_input.set_state(InputState::Active);
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
                    Self::show_exit_screen(app);
                }
                Input { key: Key::Esc, .. } => {
                    app.current_screen = Screen::Main;
                    app.active_widget = Some(ActiveWidget::Editor);
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
                        if app.user_input.get_state() == InputState::Error {
                            app.user_input.set_state(InputState::Active);
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
                Input { key: Key::Esc, .. } => {
                    app.current_screen = Screen::Main;
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
                    app.active_widget = Some(ActiveWidget::Editor);
                }
                _ => {}
            },
        };
        Ok(())
    }

    async fn save_note(
        app: &mut App<'_>,
        title: String,
        body: String,
        has_links: bool,
        note_id: Option<i64>,
    ) -> Result<()> {
        let (save_note_result, updated) = match note_id {
            Some(id) => (
                DbMac::update_note(&app.db, title.clone(), body, has_links, id).await,
                true,
            ),
            None => (
                DbMac::save_note(&app.db, title.clone(), body, has_links).await,
                false,
            ),
        };

        match save_note_result {
            Ok(parent_id) => {
                if updated {
                    let new_nid = NoteIdentifier {
                        id: parent_id,
                        title,
                    };

                    // Replaces prev note title with new one in the load note screen
                    app.note_list.replace(new_nid);
                } else {
                    let new_nid = NoteIdentifier {
                        id: parent_id,
                        title,
                    };

                    // Makes the note available in the load note screen
                    app.note_list.update(new_nid);
                    // Triggers update_note on next save
                    app.editor.note_id = Some(parent_id);
                }

                match has_links {
                    true => {
                        let db_links = app
                            .editor
                            .links
                            .clone()
                            .into_iter()
                            .map(|link| link.to_db_link())
                            .collect::<Vec<DbNoteLink>>();
                        let result = DbMac::save_links(&app.db, db_links, parent_id).await;

                        match result {
                            Ok(_) => {
                                let new_msg =
                                    UserMessage::new("Note saved!".to_string(), MessageType::Info);
                                app.user_msg = new_msg;
                                app.prev_screen = app.current_screen;
                                app.current_screen = Screen::Popup;
                                return Ok(());
                            }
                            Err(err) => {
                                let new_msg = UserMessage::new(
                                    format!("Error saving note links!: {:?}", err),
                                    MessageType::Error,
                                );
                                app.user_msg = new_msg;
                                app.prev_screen = app.current_screen;
                                app.current_screen = Screen::Popup;
                                return Err(err);
                            }
                        }
                    }
                    false => {
                        let new_msg =
                            UserMessage::new("Note saved!".to_string(), MessageType::Info);
                        app.user_msg = new_msg;
                        app.prev_screen = app.current_screen;
                        app.current_screen = Screen::Popup;
                        return Ok(());
                    }
                }
            }
            Err(err) => {
                let new_msg =
                    UserMessage::new(format!("Error saving note!: {:?}", err), MessageType::Error);
                app.user_msg = new_msg;
                app.prev_screen = app.current_screen;
                app.current_screen = Screen::Popup;
                return Err(err);
            }
        }
    }

    async fn load_note(app: &mut App<'_>, id: i64) -> Result<()> {
        let result = DbMac::load_note(&app.db, id).await;

        match result {
            Ok(note) => {
                let body = match note.body {
                    Some(text) => text
                        .split("\n")
                        .map(|line| line.to_owned())
                        .collect::<Vec<String>>(),
                    None => vec!["".to_owned()],
                };

                let db_links = match note.has_links {
                    true => DbMac::load_note_links(&app.db, id).await?,
                    false => vec![],
                };

                let links = match db_links.len() {
                    0 => vec![],
                    _ => db_links
                        .into_iter()
                        .map(|dbl| Link::from_db_link(dbl))
                        .collect::<Vec<Link>>(),
                };

                app.editor = Editor::new(note.title, body, links, Some(note.id));
                app.current_screen = Screen::Main;
                Ok(())
            }
            Err(err) => {
                let new_msg = UserMessage::new(
                    format!("Error loading note!: {:?}", err),
                    MessageType::Error,
                );
                app.user_msg = new_msg;
                app.prev_screen = app.current_screen;
                app.current_screen = Screen::Popup;
                Err(err.into())
            }
        }
    }

    fn switch_btns(app: &mut App) {
        let inactive_btn = app
            .btns
            .get_mut(&app.btn_idx)
            .expect("Selected btn should exist");

        match inactive_btn.get_state() {
            ButtonState::Unavailable => {}
            _ => inactive_btn.set_state(ButtonState::Inactive),
        }

        app.btn_idx = (app.btn_idx + 1) % (app.btns.len()) as u8;

        let active_btn = app
            .btns
            .get_mut(&app.btn_idx)
            .expect("Selected btn should exist");

        match active_btn.get_state() {
            ButtonState::Unavailable => {}
            _ => active_btn.set_state(ButtonState::Active),
        }
    }

    fn btn_action(app: &mut App) {
        let active_btn = app
            .btns
            .get_mut(&app.btn_idx)
            .expect("Selected btn should exist");

        active_btn.set_state(ButtonState::Clicked);

        match active_btn.get_action() {
            ButtonAction::RenderMainScreen => {
                app.current_screen = Screen::Main;
            }
            ButtonAction::RenderNewNoteScreen => {
                app.user_input = UserInput::new(InputState::Active, InputAction::NewNote);
                app.current_screen = Screen::NewNote;
            }
            ButtonAction::RenderLoadNoteScreen => {
                app.current_screen = Screen::LoadNote;
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
                app.user_input.set_state(InputState::Error);
                Ok(())
            }
            // If no pre-exisiting notes have that title, create and save new note with that title
            false => {
                let linked_body = "".to_string();
                let result =
                    DbMac::save_note(&app.db, linked_title.clone(), linked_body.clone(), false)
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
                            let has_links = true;
                            let parent_title = app.editor.title.clone();
                            let parent_body = app.editor.body.lines().join("\n");
                            let note_id = app.editor.note_id;
                            let parent_result =
                                Self::save_note(app, parent_title, parent_body, has_links, note_id)
                                    .await;

                            match parent_result {
                                Ok(_) => {
                                    // If parent note saved correctly, switch editor to linked) note
                                    app.editor = Editor::new(
                                        linked_title,
                                        vec![linked_body],
                                        vec![],
                                        Some(id),
                                    );
                                    app.note_list.update(new_nid);
                                    app.current_screen = Screen::Main;
                                    app.active_widget = Some(ActiveWidget::Editor);
                                    return Ok(());
                                }
                                Err(err) => return Err(err),
                            }
                        } else {
                            // if not linked to another note, simply switch to editor with new note
                            app.editor =
                                Editor::new(linked_title, vec![linked_body], vec![], Some(id));
                            app.note_list.update(new_nid);
                            app.current_screen = Screen::Main;
                            app.active_widget = Some(ActiveWidget::Editor);
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
        };

        app.editor.links.push(new_link);
        app.current_screen = Screen::Main;
        app.active_widget = Some(ActiveWidget::Editor);
        app.editor.body.new_link = false;
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
                app.user_input.set_state(InputState::Error);
            }
            false => {
                app.editor.set_title(title);
                app.current_screen = Screen::Main;
                app.active_widget = Some(ActiveWidget::Editor);
            }
        }
    }

    fn toggle_sidebar(app: &mut App) {
        match app.sidebar {
            SidebarState::Open(_) => {
                app.sidebar = SidebarState::Hidden(0);
                app.active_widget = Some(ActiveWidget::Sidebar);
            }
            SidebarState::Hidden(_) => {
                app.sidebar = SidebarState::Open(app.sidebar_size);
                app.active_widget = Some(ActiveWidget::Editor);
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
