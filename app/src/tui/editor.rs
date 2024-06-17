use std::collections::HashMap;

use log::{info, error};
use ratatui::{
    style::{Color, Modifier, Style, Stylize},
    symbols::border,
    text::{Line, Span},
    widgets::{block::Title, Block, Borders, Padding, Widget}, layout::Alignment,
};
use tuipaz_textarea::{CursorMove, Input, Key, Link as TextAreaLink, TextArea};

use crate::db::db_mac::DbNoteLink;

use super::app::ComponentState;

const DELETE_COMMANDS: [char; 7] = ['d', 'w', 'b', 'j', 'k', 'l', 'h'];
const YANK_COMMANDS: [char; 6] = ['w', 'b', 'j', 'k', 'l', 'h'];
const GOTO_COMMAND: char = 'g';

#[derive(Debug, Clone)]
pub(crate) struct Editor<'a> {
    pub(crate) title: String,
    pub(crate) note_id: Option<i64>,
    pub(crate) body: TextArea<'a>,
    pub(crate) links: HashMap<i64, Link>,
    pub(crate) deleted_link_ids: Vec<i64>,
    pub(crate) mode: EditorMode,
    pub(crate) block_info: String,
    pub(crate) prev_cursor_col: usize,
    pub(crate) num_buf: Vec<u32>,
    pub(crate) cmd_buf: String,
    pub(crate) cmd_state: CommandState,
    pub(crate) sidebar_open: bool,
    pub(crate) searchbar_open: bool,
    pub(crate) state: ComponentState,
    pub(crate) max_col: u16,
    pub(crate) hop_indexes: Vec<(usize, (usize, usize))>,
}

#[derive(Debug, Clone)]
pub(crate) struct Link {
    pub(crate) id: i64,
    pub(crate) text_id: i64,
    pub(crate) linked_id: i64,
    pub(crate) row: usize,
    pub(crate) start_col: usize,
    pub(crate) end_col: usize,
    pub(crate) saved: bool,
    pub(crate) updated: bool,
    pub(crate) deleted: bool,
}

impl Link {
    pub(crate) fn from_db_link(db_link: DbNoteLink) -> Self {
        Self {
            id: db_link.parent_note_id,
            text_id: db_link.textarea_id,
            linked_id: db_link.linked_note_id,
            row: db_link.textarea_row as usize,
            start_col: db_link.start_col as usize,
            end_col: db_link.end_col as usize,
            saved: true,
            updated: false,
            deleted: false,
        }
    }

    pub(crate) fn to_db_link(&self) -> DbNoteLink {
        DbNoteLink {
            parent_note_id: self.id,
            textarea_id: self.text_id,
            textarea_row: self.row as i64,
            start_col: self.start_col as i64,
            end_col: self.end_col as i64,
            linked_note_id: self.linked_id,
        }
    }

    pub(crate) fn to_textarea_link(&self) -> TextAreaLink {
        TextAreaLink {
            id: self.text_id as usize,
            row: self.row,
            start_col: self.start_col,
            end_col: self.end_col,
            edited: false,
            deleted: self.deleted,
        }
    }

    pub(crate) fn moved(&self, ta_link: &TextAreaLink) -> bool {
        self.row != ta_link.row
            || self.start_col != ta_link.start_col
            || self.end_col != ta_link.end_col
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub(crate) enum CommandState {
    NoCommand,
    Delete,
    Yank,
    GoTo,
    FindForward,
    FindBackward,
    PrimeHop,
    ExecuteHop,
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) enum EditorMode {
    Insert,
    Normal,
    Visual,
}

impl<'a> Editor<'a> {
    pub(crate) fn new(
        title: String,
        body: Vec<String>,
        links: HashMap<i64, Link>,
        note_id: Option<i64>,
        sidebar_open: bool,
        max_col: u16,
    ) -> Self {
        let ta_links = links
            .values()
            .map(|link| (link.text_id as usize, link.to_textarea_link()))
            .collect::<HashMap<usize,TextAreaLink>>();

        let mut body = TextArea::new(body, ta_links, max_col);
        body.set_cursor_line_style(Style::default());
        body.set_selection_style(Style::default().bg(Color::Red));
        body.set_max_histories(100);

        let block_info = " <| NORMAL |>".to_string();

        Self {
            title,
            note_id,
            body,
            links,
            deleted_link_ids: vec![],
            mode: EditorMode::Normal,
            block_info,
            prev_cursor_col: 0,
            num_buf: Vec::with_capacity(6),
            cmd_buf: String::with_capacity(6),
            cmd_state: CommandState::NoCommand,
            sidebar_open,
            searchbar_open: false,
            state: ComponentState::Active,
            max_col,
            hop_indexes: Vec::new(),
        }
    }

    pub(crate) fn set_state(&mut self, new_state: ComponentState) {
        self.state = new_state;
    }

    pub(crate) fn set_mode(&mut self, mode: EditorMode) {
        self.block_info = match mode {
            EditorMode::Insert => {
                self.body.cancel_selection();
                self.body.clear_search();
                " <| INSERT |>".to_owned()
            }
            EditorMode::Normal => {
                self.body.cancel_selection();
                " <| NORMAL |>".to_owned()
            }
            EditorMode::Visual => {
                self.body.start_selection();
                self.body.clear_search();
                " <| VISUAL |>".to_owned()
            }
        };
        self.mode = mode;
    }

    pub(crate) fn set_title(&mut self, title: String) {
        self.title = title;
    }

    pub(crate) fn handle_input(&mut self, input: Input) {
        let num_buf_len = self.num_buf.len() as u32;
        match self.mode {
            EditorMode::Insert => match input {
                Input { key: Key::Esc, .. } => {
                    self.set_mode(EditorMode::Normal);
                }
                Input { key: Key::Up, shift: false, ..} 
                | Input { key: Key::Down, shift: false, .. }
                | Input { key: Key::Up, shift: true, ..}
                | Input { key: Key::Down, shift: true, ..} => {
                    self.body.input(input);
                    self.jump_cursor_to_prev_col();
                }
                Input { key: Key::Left, shift: false, ..} 
                | Input { key: Key::Right, shift: false, .. }
                | Input { key: Key::Left, shift: true, ..}
                | Input { key: Key::Right, shift: true, ..} => {
                    self.body.input(input);
                    self.set_prev_cursor_col();
                }
                input => {
                    self.body.input(input);
                }
            },
            EditorMode::Normal => match (input, &self.cmd_state) {
                // Handle multi-key commands
                (
                    input, 
                    CommandState::GoTo | CommandState::Delete | CommandState::Yank
                    | CommandState::FindForward | CommandState::FindBackward
                    | CommandState::PrimeHop | CommandState::ExecuteHop 
                 ) => {
                    self.process_command_key_inputs(input)
                }
                // Move left
                (
                    Input {
                        key: Key::Char('h'),
                        ..
                    },
                    CommandState::NoCommand,
                )
                | (Input { key: Key::Left, .. }, CommandState::NoCommand) => {
                    match num_buf_len {
                        0 => {
                            self.body.move_cursor(CursorMove::Back);
                        }
                        _ => self.repeat_action(num_buf_len, move |editor| {
                            editor.body.move_cursor(CursorMove::Back);
                        }),
                    }
                    self.set_prev_cursor_col();
                }
                // Move Down
                (
                    Input {
                        key: Key::Char('j'),
                        ..
                    },
                    CommandState::NoCommand,
                )
                | (Input { key: Key::Down, .. }, CommandState::NoCommand) => {
                    match num_buf_len {
                        0 => {
                            self.body.move_cursor(CursorMove::Down);
                        }
                        _ => self.repeat_action(num_buf_len, move |editor| {
                            editor.body.move_cursor(CursorMove::Down);
                        }),
                    }
                    self.jump_cursor_to_prev_col();
                }
                // Move Up
                (
                    Input {
                        key: Key::Char('k'),
                        ..
                    },
                    CommandState::NoCommand,
                )
                | (Input { key: Key::Up, .. }, CommandState::NoCommand) => {
                    match num_buf_len {
                        0 => {
                            self.body.move_cursor(CursorMove::Up);
                        }
                        _ => self.repeat_action(num_buf_len, move |editor| {
                            editor.body.move_cursor(CursorMove::Up);
                        }),
                    }
                    self.jump_cursor_to_prev_col();
                }
                // Move Right
                (
                    Input {
                        key: Key::Char('l'),
                        ..
                    },
                    CommandState::NoCommand,
                )
                | (
                    Input {
                        key: Key::Right, ..
                    },
                    CommandState::NoCommand,
                ) => {
                    match num_buf_len {
                        0 => {
                            self.body.move_cursor(CursorMove::Forward);
                        }
                        _ => self.repeat_action(num_buf_len, move |editor| {
                            editor.body.move_cursor(CursorMove::Forward);
                        }),
                    }
                    self.set_prev_cursor_col();
                }
                (
                    Input {
                        key: Key::Char('w'),
                        ..
                    },
                    CommandState::NoCommand,
                ) => {
                    match num_buf_len {
                        0 => {
                            self.body.move_cursor(CursorMove::WordForward);
                        }
                        _ => self.repeat_action(num_buf_len, move |editor| {
                            editor.body.move_cursor(CursorMove::WordForward);
                        }),
                    }
                    self.set_prev_cursor_col();
                }
                (
                    Input {
                        key: Key::Char('b'),
                        ..
                    },
                    CommandState::NoCommand,
                ) => {
                    match num_buf_len {
                        0 => {
                            self.body.move_cursor(CursorMove::WordBack);
                        }
                        _ => self.repeat_action(num_buf_len, move |editor| {
                            editor.body.move_cursor(CursorMove::WordBack);
                        }),
                    }
                    self.set_prev_cursor_col();
                }
                (
                    Input {
                        key: Key::Char('^'),
                        ..
                    },
                    CommandState::NoCommand,
                ) => {
                    self.body.move_cursor(CursorMove::Head);
                    self.set_prev_cursor_col();
                }
                (
                    Input {
                        key: Key::Char('$'),
                        ..
                    },
                    CommandState::NoCommand,
                ) => {
                    self.body.move_cursor(CursorMove::End);
                    self.set_prev_cursor_col();
                }
                (
                    Input {
                        key: Key::Char('G'),
                        ..
                    },
                    CommandState::NoCommand,
                ) => {
                    self.body.move_cursor(CursorMove::Bottom);
                    self.body.move_cursor(CursorMove::Head);
                }
                (
                    Input {
                        key: Key::Char('D'),
                        ..
                    },
                    CommandState::NoCommand,
                ) => {
                    self.body.delete_line_by_end();
                }
                (
                    Input {
                        key: Key::Char('C'),
                        ..
                    },
                    CommandState::NoCommand,
                ) => {
                    self.body.delete_line_by_end();
                    self.set_mode(EditorMode::Insert);
                    self.set_prev_cursor_col();
                }
                (
                    Input {
                        key: Key::Char('p'),
                        ..
                    },
                    _,
                ) => {
                    self.body.paste();
                    self.set_prev_cursor_col();
                }
                (
                    Input {
                        key: Key::Char('u'),
                        ctrl: false,
                        ..
                    },
                    _,
                ) => {
                    self.body.undo();
                    self.set_prev_cursor_col();
                }
                (
                    Input {
                        key: Key::Char('r'),
                        ctrl: false,
                        ..
                    },
                    _,
                ) => {
                    self.body.redo();
                    self.set_prev_cursor_col();
                }
                (
                    Input {
                        key: Key::Char('x'),
                        ..
                    },
                    _,
                ) => {
                    match num_buf_len {
                        0 => {
                            self.body.delete_next_char();
                        }
                        _ => self.repeat_action(num_buf_len, move |editor| {
                            editor.body.delete_next_char();
                        }),
                    }
                    self.set_prev_cursor_col();
                }
                (
                    Input {
                        key: Key::Char('O'),
                        shift: true,
                        ..
                    },
                    _,
                ) => {
                    self.body.move_cursor(CursorMove::Up);
                    self.body.move_cursor(CursorMove::End);
                    self.body.insert_newline();
                    self.set_mode(EditorMode::Insert);
                }
                (
                    Input {
                        key: Key::Char('o'),
                        shift: false,
                        ..
                    },
                    _,
                ) => {
                    self.body.move_cursor(CursorMove::End);
                    self.body.insert_newline();
                    self.set_mode(EditorMode::Insert);
                }
                (
                    Input {
                        key: Key::Char('n'),
                        shift: false,
                        alt: false,
                        ..
                    },
                    _,
                ) => {
                    self.body.search_forward(false);
                    self.set_prev_cursor_col();
                }
                (
                    Input {
                        key: Key::Char('N'),
                        shift: true,
                        alt: false,
                        ..
                    },
                    _,
                ) => {
                    self.body.search_back(false);
                    self.set_prev_cursor_col();
                }
                // Switch modes
                (
                    Input {
                        key: Key::Char('a'),
                        ctrl: false,
                        ..
                    },
                    _,
                ) => {
                    self.set_mode(EditorMode::Insert);
                }
                (
                    Input {
                        key: Key::Char('A'),
                        ctrl: false,
                        ..
                    },
                    _,
                ) => {
                    self.body.move_cursor(CursorMove::End);
                    self.set_mode(EditorMode::Insert);
                }
                (
                    Input {
                        key: Key::Char('i'),
                        ctrl: false,
                        ..
                    },
                    _,
                ) => {
                    self.set_mode(EditorMode::Insert);
                }
                (
                    Input {
                        key: Key::Char('I'),
                        ctrl: false,
                        ..
                    },
                    _,
                ) => {
                    self.body.move_cursor(CursorMove::Head);
                    self.set_mode(EditorMode::Insert);
                }
                (
                    Input {
                        key: Key::Char('v'),
                        ctrl: false,
                        ..
                    },
                    _,
                ) => {
                    self.set_mode(EditorMode::Visual);
                }
                (
                    Input {
                        key: Key::Char('V'),
                        ctrl: false,
                        ..
                    },
                    _,
                ) => {
                    self.body.move_cursor(CursorMove::Head);
                    self.set_mode(EditorMode::Visual);
                    self.body.move_cursor(CursorMove::End);
                }
                (input, CommandState::NoCommand) => self.prime_command_state(input),
            },
            EditorMode::Visual => match (input, &self.cmd_state) {
                (Input { key: Key::Esc, .. }, _) => {
                    self.set_mode(EditorMode::Normal);
                }
                // Handle multi-key commands
                (input, CommandState::GoTo | CommandState::Delete | CommandState::Yank) => {
                    self.process_command_key_inputs(input)
                }
                // Move left
                (
                    Input {
                        key: Key::Char('h'),
                        ..
                    },
                    CommandState::NoCommand,
                )
                | (Input { key: Key::Left, .. }, CommandState::NoCommand) => {
                    match num_buf_len {
                        0 => {
                            self.body.move_cursor(CursorMove::Back);
                        }
                        _ => self.repeat_action(num_buf_len, move |editor| {
                            editor.body.move_cursor(CursorMove::Back);
                        }),
                    }
                    self.set_prev_cursor_col();
                }
                // Move Down
                (
                    Input {
                        key: Key::Char('j'),
                        ..
                    },
                    CommandState::NoCommand,
                )
                | (Input { key: Key::Down, .. }, CommandState::NoCommand) => {
                    match num_buf_len {
                        0 => {
                            self.body.move_cursor(CursorMove::Down);
                        }
                        _ => self.repeat_action(num_buf_len, move |editor| {
                            editor.body.move_cursor(CursorMove::Down);
                        }),
                    }
                    self.jump_cursor_to_prev_col();
                }
                // Move Up
                (
                    Input {
                        key: Key::Char('k'),
                        ..
                    },
                    CommandState::NoCommand,
                )
                | (Input { key: Key::Up, .. }, CommandState::NoCommand) => {
                    match num_buf_len {
                        0 => {
                            self.body.move_cursor(CursorMove::Up);
                        }
                        _ => self.repeat_action(num_buf_len, move |editor| {
                            editor.body.move_cursor(CursorMove::Up);
                        }),
                    }
                    self.jump_cursor_to_prev_col();
                }
                // Move Right
                (
                    Input {
                        key: Key::Char('l'),
                        ..
                    },
                    CommandState::NoCommand,
                )
                | (
                    Input {
                        key: Key::Right, ..
                    },
                    CommandState::NoCommand,
                ) => {
                    match num_buf_len {
                        0 => {
                            self.body.move_cursor(CursorMove::Forward);
                        }
                        _ => self.repeat_action(num_buf_len, move |editor| {
                            editor.body.move_cursor(CursorMove::Forward);
                        }),
                    }
                    self.set_prev_cursor_col();
                }
                (
                    Input {
                        key: Key::Char('w'),
                        ..
                    },
                    CommandState::NoCommand,
                ) => {
                    match num_buf_len {
                        0 => {
                            self.body.move_cursor(CursorMove::WordForward);
                        }
                        _ => self.repeat_action(num_buf_len, move |editor| {
                            editor.body.move_cursor(CursorMove::WordForward);
                        }),
                    }
                    self.set_prev_cursor_col();
                }
                (
                    Input {
                        key: Key::Char('b'),
                        ctrl: false,
                        ..
                    },
                    CommandState::NoCommand,
                ) => {
                    match num_buf_len {
                        0 => {
                            self.body.move_cursor(CursorMove::WordBack);
                        }
                        _ => self.repeat_action(num_buf_len, move |editor| {
                            editor.body.move_cursor(CursorMove::WordBack);
                        }),
                    }
                    self.set_prev_cursor_col();
                }
                (
                    Input {
                        key: Key::Char('^'),
                        ..
                    },
                    CommandState::NoCommand,
                ) => {
                    self.body.move_cursor(CursorMove::Head);
                    self.set_prev_cursor_col();
                }
                (
                    Input {
                        key: Key::Char('$'),
                        ..
                    },
                    CommandState::NoCommand,
                ) => {
                    self.body.move_cursor(CursorMove::End);
                    self.set_prev_cursor_col();
                }
                (
                    Input {
                        key: Key::Char('G'),
                        ..
                    },
                    CommandState::NoCommand,
                ) => {
                    self.body.move_cursor(CursorMove::Bottom);
                    self.body.move_cursor(CursorMove::Head);
                    self.set_prev_cursor_col();
                }
                (
                    Input {
                        key: Key::Char('y'),
                        ..
                    },
                    CommandState::NoCommand,
                ) => {
                    self.body.copy();
                    self.set_mode(EditorMode::Normal);
                }
                (
                    Input {
                        key: Key::Char('x'),
                        ..
                    },
                    CommandState::NoCommand,
                ) => {
                    self.body.cut();
                    self.set_mode(EditorMode::Normal);
                }
                (
                    Input {
                        key: Key::Char('1'),
                        ..
                    },
                    CommandState::NoCommand,
                ) => {
                    self.body.set_alignment(Alignment::Left);
                }
                (
                    Input {
                        key: Key::Char('2'),
                        ..
                    },
                    CommandState::NoCommand,
                ) => {
                    self.body.set_alignment(Alignment::Center);
                }
                (
                    Input {
                        key: Key::Char('3'),
                        ..
                    },
                    CommandState::NoCommand,
                ) => {
                    self.body.set_alignment(Alignment::Right);
                }
                _ => {}
            },
        }
    }

    fn set_prev_cursor_col(&mut self) {
        self.prev_cursor_col = self.body.cursor().1;
    }

    fn jump_cursor_to_prev_col(&mut self) {
        let c = self.body.cursor();
        let line_len = self.body.lines()[c.0].len();

        if self.prev_cursor_col < line_len {
            self.body.move_cursor(CursorMove::Jump(c.0 as u16, self.prev_cursor_col as u16));
        } else {
            self.body.move_cursor(CursorMove::End);
        }
    }

    fn prime_command_state(&mut self, input: Input) {
        match input.key {
            Key::Char(c) => {
                if let Some(num) = c.to_digit(10) {
                    self.num_buf.push(num);
                } else {
                    match (c, &self.cmd_state) {
                        ('d', CommandState::NoCommand) => {
                            self.cmd_buf.push(c);
                            self.cmd_state = CommandState::Delete;
                        }
                        ('g', CommandState::NoCommand) => {
                            self.cmd_buf.push(c);
                            self.cmd_state = CommandState::GoTo;
                        }
                        ('y', CommandState::NoCommand) => {
                            self.cmd_buf.push(c);
                            self.cmd_state = CommandState::Yank;
                        }
                        ('f', CommandState::NoCommand) => {
                            self.cmd_buf.push(c);
                            self.cmd_state = CommandState::FindForward;
                        }
                        ('F', CommandState::NoCommand) => {
                            self.cmd_buf.push(c);
                            self.cmd_state = CommandState::FindBackward;
                        }
                        ('s', CommandState::NoCommand) => {
                            self.cmd_state = CommandState::PrimeHop;
                        }
                        _ => {
                            self.cmd_state = CommandState::NoCommand;
                            self.cmd_buf.clear();
                        }
                    };
                }
            }
            _ => {
                self.cmd_buf.clear();
                self.num_buf.clear();
                self.cmd_state = CommandState::NoCommand;
            }
        }
    }

    fn process_command_key_inputs(&mut self, input: Input) {
        info!("process_command_key_inputs::self.input: {:?}", input);
        info!("process_command_key_inputs::self.cmd_state: {:?}", self.cmd_state);
        info!("process_command_key_inputs::self.body.hop: {:?}", self.body.hop);
        if let Key::Char(c) = input.key {
            self.cmd_buf.push(c);

            if DELETE_COMMANDS.contains(&c) && self.cmd_state == CommandState::Delete {
                self.execute_delete(c);
            } else if YANK_COMMANDS.contains(&c) && self.cmd_state == CommandState::Yank {
                self.execute_yank(c);
            } else if c == GOTO_COMMAND && self.cmd_state == CommandState::GoTo {
                self.execute_goto(c);
            } else if self.cmd_state == CommandState::FindForward {
                self.execute_find(c, true);
            } else if self.cmd_state == CommandState::FindBackward {
                self.execute_find(c, false);
            } else if self.cmd_state == CommandState::PrimeHop {
                if self.cmd_buf.len() == 2 {
                    let search_str = self.cmd_buf.clone();
                    self.prime_hop(&search_str);
                }
            } else if self.cmd_state == CommandState::ExecuteHop {
                if let Some(num) = c.to_digit(10) {
                    self.num_buf.push(num);
                } else if c == 'h' {
                    self.execute_hop();
                }
            } else {
                self.cmd_state = CommandState::NoCommand;
            }
        }

    }

    fn execute_delete(&mut self, modifier: char) {
        match modifier {
            'd' | 'j' => {
                let actions = move |editor: &mut Editor<'a>| {
                    editor.body.delete_line(false);
                };

                let num_buf_len = self.num_buf.len() as u32;
                match num_buf_len {
                    0 => {
                        actions(self);
                    }
                    _ => self.repeat_action(num_buf_len, move |editor| {
                        actions(editor);
                    }),
                }

                self.cmd_buf.clear();
            }
            'k' => {
                let actions = move |editor: &mut Editor<'a>| {
                    editor.body.delete_line(true)
                };

                let num_buf_len = self.num_buf.len() as u32;
                match num_buf_len {
                    0 => {
                        actions(self);
                    }
                    _ => self.repeat_action(num_buf_len, move |editor| {
                        actions(editor);
                    }),
                }
                self.cmd_buf.clear();
            }
            'h' => {
                let actions = move |editor: &mut Editor<'a>| editor.body.delete_char();
                let num_buf_len = self.num_buf.len() as u32;
                match num_buf_len {
                    0 => {
                        actions(self);
                    }
                    _ => self.repeat_action(num_buf_len, move |editor| {
                        actions(editor);
                    }),
                }

                self.cmd_buf.clear();
            }
            'l' => {
                let actions = move |editor: &mut Editor<'a>| editor.body.delete_next_char();
                let num_buf_len = self.num_buf.len() as u32;
                match num_buf_len {
                    0 => {
                        actions(self);
                    }
                    _ => self.repeat_action(num_buf_len, move |editor| {
                        actions(editor);
                    }),
                }
                self.cmd_buf.clear();
            }
            'w' => {
                let actions = move |editor: &mut Editor<'a>| editor.body.delete_next_word();
                let num_buf_len = self.num_buf.len() as u32;
                match num_buf_len {
                    0 => {
                        actions(self);
                    }
                    _ => self.repeat_action(num_buf_len, move |editor| {
                        actions(editor);
                    }),
                }
                self.cmd_buf.clear();
            }
            'b' => {
                let actions = move |editor: &mut Editor<'a>| editor.body.delete_word();
                let num_buf_len = self.num_buf.len() as u32;
                match num_buf_len {
                    0 => {
                        actions(self);
                    }
                    _ => self.repeat_action(num_buf_len, move |editor| {
                        actions(editor);
                    }),
                }
                self.cmd_buf.clear();
            }
            _ => {
                self.cmd_buf.clear();
                self.num_buf.clear();
            }
        }
        self.cmd_state = CommandState::NoCommand;
    }

    fn execute_yank(&mut self, modifier: char) {
        match modifier {
            'l' => {
                let num_buf_len = self.num_buf.len() as u32;
                if num_buf_len != 0 {
                    let num = self.get_num_from_buf(num_buf_len);
                    for _ in 0..num {
                        self.body.move_cursor(CursorMove::Head);
                        self.body.delete_line_by_end();
                        self.body.delete_newline();
                        self.body.move_cursor(CursorMove::Down);
                    }
                    self.num_buf.clear();
                } else {
                    self.body.move_cursor(CursorMove::Head);
                    self.body.delete_line_by_end();
                    self.body.delete_newline();
                }
                self.cmd_buf.clear();
            }
            'w' => {
                let num_buf_len = self.num_buf.len() as u32;
                if num_buf_len != 0 {
                    let num = self.get_num_from_buf(num_buf_len);
                    for _ in 0..num {
                        self.body.delete_word();
                    }
                    self.num_buf.clear();
                } else {
                    self.body.delete_word();
                }
                self.cmd_buf.clear();
            }
            'b' => {
                let num_buf_len = self.num_buf.len() as u32;
                if num_buf_len != 0 {
                    let num = self.get_num_from_buf(num_buf_len);
                    for _ in 0..num {
                        self.body.move_cursor(CursorMove::WordBack);
                        self.body.delete_word();
                    }
                    self.num_buf.clear();
                } else {
                    self.body.move_cursor(CursorMove::WordBack);
                    self.body.delete_word();
                }
                self.cmd_buf.clear();
            }
            _ => {
                self.cmd_buf.clear();
                self.num_buf.clear();
            }
        }
        self.cmd_state = CommandState::NoCommand;
    }

    fn execute_goto(&mut self, modifier: char) {
        match modifier {
            'g' => {
                let num_buf_len = self.num_buf.len() as u32;
                if num_buf_len != 0 {
                    let num = self.get_num_from_buf(num_buf_len);
                    self.body.move_cursor(CursorMove::Jump(num - 1, 0));
                    self.num_buf.clear();
                } else {
                    self.body.move_cursor(CursorMove::Top);
                    self.body.move_cursor(CursorMove::Head);
                }
                self.cmd_buf.clear();
            }
            _ => {
                self.cmd_buf.clear();
                self.num_buf.clear();
            }
        }
        self.cmd_state = CommandState::NoCommand;
    }

    fn execute_find(&mut self, target: char, forward: bool) {
        let cursor = self.body.cursor();
        let line = &self.body.lines()[cursor.0];

        match forward {
            true => {
                if let Some(col) = line[cursor.1 + 1..].chars().position(|c| c == target) {
                    self.body.move_cursor(CursorMove::Jump(cursor.0 as u16, (cursor.1 + 1 + col) as u16));
                }
            },
            false => {
                if let Some(col) = line[..cursor.1 - 1].chars().rev().position(|c| c == target) {
                    self.body.move_cursor(CursorMove::Jump(cursor.0 as u16, (cursor.1 - 2 - col) as u16));
                }
            },
        } 

        self.cmd_buf.clear();
        self.num_buf.clear();
        self.cmd_state = CommandState::NoCommand;
    }

    fn prime_hop(&mut self, target: &str) {
        match self.body.set_hop_pattern(target) {
            Ok(_) => info!("found: {}", target),
            Err(e) => error!("Hop error: {}", e),
        }

        info!("prime_hop::self.body.hop: {:?}", self.body.hop);
        self.cmd_state = CommandState::ExecuteHop;
    }

    fn execute_hop(&mut self) {
        info!("Executing hop BEFORE CLEAR: {:?}", self.body.hop);
        let num_buf_len = self.num_buf.len();
        let idx = self.get_num_from_buf(num_buf_len as u32);
        info!("execute_hop::idx: {}", idx);
        self.body.hop_to_idx(idx as usize);
        self.body.clear_hop();
        self.cmd_buf.clear();
        self.num_buf.clear();
        self.cmd_state = CommandState::NoCommand;
        info!("Executing hop AFTER CLEAR: {:?}", self.body.hop);
    }

    fn repeat_action<F>(&mut self, num_buf_len: u32, mut action: F)
    where
        F: FnMut(&mut Self) + 'static,
    {
        let repetitions = self.get_num_from_buf(num_buf_len);

        for _ in 0..repetitions {
            action(self);
        }

        self.num_buf.clear();
    }

    fn get_num_from_buf(&self, num_buf_len: u32) -> u16 {
        self.num_buf
            .iter()
            .enumerate()
            .fold(0u32, |mut acc, (idx, n)| {
                acc += std::cmp::max(
                    *n,
                    (10u32).pow(num_buf_len - 1 - idx as u32) * *n,
                );
                acc
            }) as u16
    }
}

impl<'a> Widget for Editor<'a> {
    fn render(mut self, area: ratatui::prelude::Rect, buf: &mut ratatui::prelude::Buffer)
    where
        Self: Sized,
    {
        let info_style = match self.mode {
            EditorMode::Insert => Style::default().bold().fg(Color::Blue),
            EditorMode::Normal => Style::default().bold().fg(Color::Yellow),
            EditorMode::Visual => Style::default().bold().fg(Color::Red),
        };

        let (title_style, key_hint_style, text_style) = match self.state {
            ComponentState::Active => (
                Style::default().bold().fg(Color::Yellow),
                Style::default().bold(),
                Style::default()
            ),
            ComponentState::Inactive => (
                Style::default().bold().dim(),
                Style::default().bold().dim(),
                Style::default().dim()
            ),
            _ => (Style::default(), Style::default(), Style::default())
        };

        let block_info_len = self.block_info.len();

        let (mode_span, key_hint_span) = match self.searchbar_open {
            true => {
                (Span::styled("", Style::default()), Span::styled("", key_hint_style))
            },
            false => {
                (
                    Span::styled(self.block_info, info_style),
                    Span::styled(
                        " | <Alt-q> quit | <Alt-s/l/d/n> save/load/delete/new | <Alt-t> edit title ",
                        key_hint_style,
                    ),
                )
            },
        };

        let (
            file_explorer_hint_text, 
            cursor_style, 
            top_right, 
            bottom_left, 
            bottom_right
        ) = match (self.sidebar_open, self.searchbar_open) {
            (true, true) => ("".to_owned(), Style::default(), "┬", "├", "┤"),
            (false, true) => ("".to_owned(), Style::default(), "╮", "├", "┤"),
            (true, false) => ("".to_owned(), Style::default().add_modifier(Modifier::REVERSED), "┬", "╰","┴"),
            (false, false) => (
                " <Alt-f> show files ".to_owned(),
                Style::default().add_modifier(Modifier::REVERSED),
                "╮", "╰", "╯"
            ),
        };

        let feh_len = file_explorer_hint_text.len();
        let title_text = format!(" {} ", self.title);
        let title = Span::styled(title_text, title_style);
        let file_explorer_hint = Span::styled(file_explorer_hint_text, Style::default().add_modifier(Modifier::BOLD));

        let kht_len = key_hint_span.content.len();
        let tb_text_len = (kht_len + block_info_len + feh_len) as u16;
        let tb_padding_width = match self.sidebar_open {
            true => 0,
            false => (area.width - tb_text_len - 5) as usize,
        };
        let tb_padding = Span::styled("─".repeat(tb_padding_width), Style::default());
        let prefix_padding = Span::styled("─", Style::default());

        let editor_block = Block::default()
            .title(Title::from(title).alignment(Alignment::Left))
            .title_bottom(Line::from(vec![
                prefix_padding,
                mode_span,
                key_hint_span,
                tb_padding,
                file_explorer_hint
            ]))
            .borders(Borders::ALL)
            .border_set(border::Set {
                top_left: "╭",
                top_right,
                bottom_left,
                bottom_right,
                vertical_left: "│",
                vertical_right: "│",
                horizontal_top: "─",
                horizontal_bottom: "─",
            })
            .padding(Padding::new(1, 1, 1, 1));

        self.body.set_block(editor_block);
        self.body.set_style(text_style);
        self.body.set_cursor_style(cursor_style);

        self.body.widget().render(area, buf);
    }
}

#[cfg(test)]

mod tests {
    use super::*;

    #[test]
    fn test_execute_delete_dd_no_links() {
        let mut editor = Editor::new(
            "Test Note".to_string(),
            vec!["Line 1".to_string(), "Line 2".to_string()],
            HashMap::new(),
            None,
            false,
            140,
        );
        editor.set_mode(EditorMode::Normal);
        editor.body.move_cursor(CursorMove::Jump(0, 0));
        
        editor.execute_delete('d');
        assert_eq!(editor.body.lines(), vec!["Line 2".to_string()]);
    }

    #[test]
    fn test_execute_delete_dk_no_links() {
        let mut editor = Editor::new(
            "Test Note".to_string(),
            vec!["Line 1".to_string(), "Line 2".to_string()],
            HashMap::new(),
            None,
            false,
            140,
        );
        editor.set_mode(EditorMode::Normal);
        editor.body.move_cursor(CursorMove::Jump(1, 0));
        
        editor.execute_delete('k');
        assert_eq!(editor.body.lines(), vec!["Line 1".to_string()]);
    }
    #[test]
    fn test_execute_delete_dj_no_links() {
        let mut editor = Editor::new(
            "Test Note".to_string(),
            vec!["Line 1".to_string(), "Line 2".to_string()],
            HashMap::new(),
            None,
            false,
            140,
        );
        editor.set_mode(EditorMode::Normal);
        editor.body.move_cursor(CursorMove::Jump(0, 0));
        
        editor.execute_delete('j');
        assert_eq!(editor.body.lines(), vec!["Line 2".to_string()]);
    }

    #[test]
    fn test_execute_delete_num_dd_no_links() {
        let mut editor = Editor::new(
            "Test Note".to_string(),
            vec!["Line 1".to_string(), "Line 2".to_string()],
            HashMap::new(),
            None,
            false,
            140,
        );
        editor.set_mode(EditorMode::Normal);
        editor.body.move_cursor(CursorMove::Jump(0, 0));
        editor.num_buf = vec![2];
        editor.execute_delete('d');
        assert_eq!(editor.body.lines(), vec!["".to_string()]);
    }

    #[test]
    fn test_execute_delete_num_dj_no_links() {
     let mut editor = Editor::new(
            "Test Note".to_string(), 
            vec!["Line 1".to_string(), "Line 2".to_string(), "Line 3".to_string()],
            HashMap::new(),
            None,
            false,
            140,
        );

        editor.set_mode(EditorMode::Normal);
        editor.body.move_cursor(CursorMove::Jump(0, 0));
        editor.num_buf = vec![2]; 
        editor.execute_delete('j');
        assert_eq!(editor.body.lines(), vec!["Line 3".to_string()]);
    }

    #[test]
    fn test_execute_delete_num_dk_no_links() {
        let mut editor = Editor::new(
            "Test Note".to_string(), 
            vec!["Line 1".to_string(), "Line 2".to_string(), "Line 3".to_string()],
            HashMap::new(),
            None,
            false,
            140,
        );
        editor.set_mode(EditorMode::Normal);
        editor.num_buf = vec![2];
        editor.body.move_cursor(CursorMove::Jump(1, 0));
        editor.execute_delete('k');
        assert_eq!(editor.body.lines(), vec!["Line 3".to_string()]);
    }

    #[test]
    fn test_execute_delete_dw_no_links() {
        let mut editor = Editor::new(
            "Test Note".to_string(),
            vec!["Line 1".to_string()],
            HashMap::new(),
            None,
            false,
            140
        );
        editor.set_mode(EditorMode::Normal);
        editor.body.move_cursor(CursorMove::Jump(0, 0));
        editor.execute_delete('w');
        assert_eq!(editor.body.lines(), vec![" 1".to_string()]);
    }
    #[test]
    fn test_execute_delete_db_no_links() {
        let mut editor = Editor::new(
            "Test Note".to_string(),
            vec!["Line one".to_string()],
            HashMap::new(),
            None,
            false,
            140,
        );
        editor.set_mode(EditorMode::Normal);
        editor.body.move_cursor(CursorMove::End);
        editor.execute_delete('b');
        assert_eq!(editor.body.lines(), vec!["Line ".to_string()]);
    }

    #[test]
    fn test_execute_delete_num_dw_no_links() {
        let mut editor = Editor::new(
            "Test Note".to_string(),
            vec!["Word one Word two".to_string()],
            HashMap::new(),
            None,
            false,
            140,
        );
        editor.set_mode(EditorMode::Normal);
        editor.body.move_cursor(CursorMove::Jump(0, 0));
        editor.num_buf = vec![3]; 
        editor.execute_delete('w');
        assert_eq!(editor.body.lines(), vec![" two".to_string()]);
    }

    #[test]
    fn test_execute_delete_num_db_no_links() {
        let mut editor = Editor::new(
            "Test Note".to_string(),
            vec!["First word second word".to_string()],
            HashMap::new(),
            None,
            false,
            140,
        );
        editor.set_mode(EditorMode::Normal);
        editor.num_buf = vec![3];
        editor.body.move_cursor(CursorMove::End);
        editor.execute_delete('b');
        assert_eq!(editor.body.lines(), vec!["First ".to_string()]);
    }

    #[test]
    fn test_multiple_nums_in_buf_delete_char() {
        let mut editor = Editor::new(
            "Test Note".to_string(),
            vec!["1 2 3 4 5 6 7".to_string()],
            HashMap::new(),
            None,
            false,
            140,
        );
        editor.set_mode(EditorMode::Normal);
        editor.num_buf = vec![1, 2];
        editor.body.move_cursor(CursorMove::Jump(0,0));
        editor.handle_input(Input { key: Key::Char('x'), ..Default::default() });
        assert_eq!(editor.body.lines(), vec!["7".to_string()]);
    }


    #[test]
    fn test_goto_top_of_note() {
        let mut editor = Editor::new(
            "Test Note".to_string(),
            vec!["1".to_string(), "2".to_string(), 
                "3".to_string(), "4".to_string(), 
                "5".to_string()
            ], 
            HashMap::new(), 
            None,
            false,
            140,
        );
        editor.set_mode(EditorMode::Normal);
        editor.body.move_cursor(CursorMove::Jump(4,0));
        editor.execute_goto('g');
        assert_eq!(editor.body.cursor(), (0, 0));
    }

    #[test]
    fn test_goto_line_num_note() {
        let mut editor = Editor::new(
            "Test Note".to_string(),
            vec!["1".to_string(), "2".to_string(), 
                "3".to_string(), "4".to_string(), 
                "5".to_string()
            ], 
            HashMap::new(), 
            None,
            false,
            140,
        );
        editor.set_mode(EditorMode::Normal);
        editor.num_buf = vec![3];
        editor.body.move_cursor(CursorMove::Jump(4,0));
        editor.execute_goto('g');
        assert_eq!(editor.body.cursor(), (2, 0));
    }

    #[test]
    fn test_goto_end_of_note() {
        let mut editor = Editor::new(
            "Test Note".to_string(),
            vec!["1".to_string(), "2".to_string(), 
                "3".to_string(), "4".to_string(), 
                "5".to_string()
            ], 
            HashMap::new(), 
            None,
            false,
            140,
        );
        editor.set_mode(EditorMode::Normal);
        editor.body.move_cursor(CursorMove::Jump(0,0));
        editor.handle_input(Input { key: Key::Char('G'), shift: true, ..Default::default() });
        assert_eq!(editor.body.cursor(), (4, 0));
    }
}
