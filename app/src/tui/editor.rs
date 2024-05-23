use std::collections::HashMap;

use ratatui::{
    layout::Alignment,
    style::{Color, Modifier, Style, Stylize},
    symbols::border,
    text::{Line, Span},
    widgets::{block::Title, Block, Borders, Padding, Widget},
};
use tuipaz_textarea::{CursorMove, Input, Key, Link as TextAreaLink, TextArea};

use crate::db::db_mac::DbNoteLink;

const DELETE_COMMANDS: [char; 7] = ['d', 'w', 'b', 'j', 'k', 'l', 'h'];
const YANK_COMMANDS: [char; 6] = ['w', 'b', 'j', 'k', 'l', 'h'];
const GOTO_COMMANDS: [char; 1] = ['g'];

#[derive(Debug, Clone)]
pub(crate) struct Editor<'a> {
    pub(crate) title: String,
    pub(crate) note_id: Option<i64>,
    pub(crate) body: TextArea<'a>,
    pub(crate) links: Vec<Link>,
    pub(crate) deleted_link_ids: Vec<(i64, i64)>,
    pub(crate) mode: EditorMode,
    pub(crate) block_info: String,
    pub(crate) prev_cursor: CursorPosition,
    pub(crate) num_buf: Vec<u32>,
    pub(crate) cmd_buf: String,
    pub(crate) cmd_state: CommandState,
}

#[derive(Debug, Clone)]
pub(crate) struct Link {
    pub(crate) id: i64,
    pub(crate) text_id: i64,
    pub(crate) linked_id: i64,
    pub(crate) row: usize,
    pub(crate) start_col: usize,
    pub(crate) end_col: usize,
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
        }
    }
}

#[derive(Debug, Clone)]
pub(crate) enum CursorPosition {
    Head,
    Middle(usize),
    End,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub(crate) enum CommandState {
    NoCommand,
    Delete,
    Yank,
    GoTo,
}

#[derive(Debug, Clone)]
pub(crate) enum EditorMode {
    Insert,
    Normal,
    Visual,
}

impl<'a> Editor<'a> {
    pub(crate) fn new(
        title: String,
        body: Vec<String>,
        links: Vec<Link>,
        note_id: Option<i64>,
    ) -> Self {
        let ta_links = links
            .iter()
            .map(|link| (link.text_id as usize, link.to_textarea_link()))
            .collect::<HashMap<usize,TextAreaLink>>();

        let mut body = TextArea::new(body, ta_links);
        body.set_cursor_line_style(Style::default());
        body.set_selection_style(Style::default().bg(Color::Red));
        body.set_max_histories(1000);

        let block_info = " <| NORMAL |> ".to_string();
        let info_style = Style::default().bold().fg(Color::Yellow);
        let mode_span = Span::styled(block_info.clone(), info_style);
        let key_hint_span = Span::styled(
            " <Alt-q/s/l/n> Quit/Save/Load/New <Alt-t> Edit title ",
            Style::default(),
        );

        let editor_block = Block::default()
            .title(Title::from(title.clone()))
            .title_style(Style::default().add_modifier(Modifier::BOLD))
            .borders(Borders::ALL)
            .border_set(border::Set {
                top_left: "╭",
                top_right: "┬",
                bottom_left: "╰",
                bottom_right: "┴",
                vertical_left: "│",
                vertical_right: "│",
                horizontal_top: "─",
                horizontal_bottom: "─",
            })
            .padding(Padding::new(1, 1, 1, 1))
            .title_bottom(Line::from(vec![mode_span, key_hint_span]));

        body.set_block(editor_block.clone());

        Self {
            title,
            note_id,
            body,
            links,
            deleted_link_ids: vec![],
            mode: EditorMode::Normal,
            block_info,
            prev_cursor: CursorPosition::Head,
            num_buf: Vec::with_capacity(6),
            cmd_buf: String::with_capacity(3),
            cmd_state: CommandState::NoCommand,
        }
    }

    pub(crate) fn set_mode(&mut self, mode: EditorMode) {
        self.block_info = match mode {
            EditorMode::Insert => {
                self.body.cancel_selection();
                " <| INSERT |> ".to_owned()
            }
            EditorMode::Normal => {
                self.body.cancel_selection();
                " <| NORMAL |> ".to_owned()
            }
            EditorMode::Visual => {
                self.body.start_selection();
                " <| VISUAL |> ".to_owned()
            }
        };
        self.mode = mode;
    }

    pub(crate) fn set_title(&mut self, title: String) {
        self.title = title;
    }

    pub(crate) fn handle_input(&mut self, input: Input) {
        match self.mode {
            EditorMode::Insert => match input {
                Input { key: Key::Esc, .. } => {
                    self.set_mode(EditorMode::Normal);
                }
                input => {
                    self.body.input(input);
                }
            },
            EditorMode::Normal => match (input, &self.cmd_state) {
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
                    let num_buf_len = self.num_buf.len() as u32;
                    match num_buf_len {
                        0 => {
                            self.body.move_cursor(CursorMove::Back);
                        }
                        _ => self.repeat_action(num_buf_len, move |editor| {
                            editor.body.move_cursor(CursorMove::Back);
                        }),
                    }
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
                    let num_buf_len = self.num_buf.len() as u32;
                    match num_buf_len {
                        0 => {
                            self.body.move_cursor(CursorMove::Down);
                        }
                        _ => self.repeat_action(num_buf_len, move |editor| {
                            editor.body.move_cursor(CursorMove::Down);
                        }),
                    }
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
                    let num_buf_len = self.num_buf.len() as u32;
                    match num_buf_len {
                        0 => {
                            self.body.move_cursor(CursorMove::Up);
                        }
                        _ => self.repeat_action(num_buf_len, move |editor| {
                            editor.body.move_cursor(CursorMove::Up);
                        }),
                    }
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
                    let num_buf_len = self.num_buf.len() as u32;
                    match num_buf_len {
                        0 => {
                            self.body.move_cursor(CursorMove::Forward);
                        }
                        _ => self.repeat_action(num_buf_len, move |editor| {
                            editor.body.move_cursor(CursorMove::Forward);
                        }),
                    }
                }
                (
                    Input {
                        key: Key::Char('w'),
                        ..
                    },
                    CommandState::NoCommand,
                ) => {
                    let num_buf_len = self.num_buf.len() as u32;
                    match num_buf_len {
                        0 => {
                            self.body.move_cursor(CursorMove::WordForward);
                        }
                        _ => self.repeat_action(num_buf_len, move |editor| {
                            editor.body.move_cursor(CursorMove::WordForward);
                        }),
                    }
                }
                (
                    Input {
                        key: Key::Char('b'),
                        ctrl: false,
                        ..
                    },
                    CommandState::NoCommand,
                ) => {
                    let num_buf_len = self.num_buf.len() as u32;
                    match num_buf_len {
                        0 => {
                            self.body.move_cursor(CursorMove::WordBack);
                        }
                        _ => self.repeat_action(num_buf_len, move |editor| {
                            editor.body.move_cursor(CursorMove::WordBack);
                        }),
                    }
                }
                (
                    Input {
                        key: Key::Char('^'),
                        ..
                    },
                    CommandState::NoCommand,
                ) => {
                    self.body.move_cursor(CursorMove::Head);
                }
                (
                    Input {
                        key: Key::Char('$'),
                        ..
                    },
                    CommandState::NoCommand,
                ) => {
                    self.body.move_cursor(CursorMove::End);
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
                }
                (
                    Input {
                        key: Key::Char('p'),
                        ..
                    },
                    _,
                ) => {
                    self.body.paste();
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
                }
                (
                    Input {
                        key: Key::Char('x'),
                        ..
                    },
                    _,
                ) => {
                    let num_buf_len = self.num_buf.len() as u32;
                    match num_buf_len {
                        0 => {
                            self.body.delete_next_char();
                        }
                        _ => self.repeat_action(num_buf_len, move |editor| {
                            editor.body.delete_next_char();
                        }),
                    }
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
                    let num_buf_len = self.num_buf.len() as u32;
                    match num_buf_len {
                        0 => {
                            self.body.move_cursor(CursorMove::Back);
                        }
                        _ => self.repeat_action(num_buf_len, move |editor| {
                            editor.body.move_cursor(CursorMove::Back);
                        }),
                    }
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
                    let num_buf_len = self.num_buf.len() as u32;
                    match num_buf_len {
                        0 => {
                            self.body.move_cursor(CursorMove::Down);
                        }
                        _ => self.repeat_action(num_buf_len, move |editor| {
                            editor.body.move_cursor(CursorMove::Down);
                        }),
                    }
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
                    let num_buf_len = self.num_buf.len() as u32;
                    match num_buf_len {
                        0 => {
                            self.body.move_cursor(CursorMove::Up);
                        }
                        _ => self.repeat_action(num_buf_len, move |editor| {
                            editor.body.move_cursor(CursorMove::Up);
                        }),
                    }
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
                    let num_buf_len = self.num_buf.len() as u32;
                    match num_buf_len {
                        0 => {
                            self.body.move_cursor(CursorMove::Forward);
                        }
                        _ => self.repeat_action(num_buf_len, move |editor| {
                            editor.body.move_cursor(CursorMove::Forward);
                        }),
                    }
                }
                (
                    Input {
                        key: Key::Char('w'),
                        ..
                    },
                    CommandState::NoCommand,
                ) => {
                    let num_buf_len = self.num_buf.len() as u32;
                    match num_buf_len {
                        0 => {
                            self.body.move_cursor(CursorMove::WordForward);
                        }
                        _ => self.repeat_action(num_buf_len, move |editor| {
                            editor.body.move_cursor(CursorMove::WordForward);
                        }),
                    }
                }
                (
                    Input {
                        key: Key::Char('b'),
                        ctrl: false,
                        ..
                    },
                    CommandState::NoCommand,
                ) => {
                    let num_buf_len = self.num_buf.len() as u32;
                    match num_buf_len {
                        0 => {
                            self.body.move_cursor(CursorMove::WordBack);
                        }
                        _ => self.repeat_action(num_buf_len, move |editor| {
                            editor.body.move_cursor(CursorMove::WordBack);
                        }),
                    }
                }
                (
                    Input {
                        key: Key::Char('^'),
                        ..
                    },
                    CommandState::NoCommand,
                ) => {
                    self.body.move_cursor(CursorMove::Head);
                }
                (
                    Input {
                        key: Key::Char('$'),
                        ..
                    },
                    CommandState::NoCommand,
                ) => {
                    self.body.move_cursor(CursorMove::End);
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
                        key: Key::Char('y'),
                        ..
                    },
                    CommandState::NoCommand,
                ) => {
                    self.body.yank_text();
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
                _ => {}
            },
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
        if let Key::Char(c) = input.key {
            self.cmd_buf.push(c);

            if DELETE_COMMANDS.contains(&c) && self.cmd_state == CommandState::Delete {
                self.execute_delete(c);
            } else if GOTO_COMMANDS.contains(&c) && self.cmd_state == CommandState::GoTo {
                self.execute_goto(c);
            } else if YANK_COMMANDS.contains(&c) && self.cmd_state == CommandState::Yank {
                self.execute_yank(c);
            } else {
                self.cmd_state = CommandState::NoCommand;
            }
        }
    }

    fn execute_delete(&mut self, modifier: char) {
        match modifier {
            'd' | 'j' => {
                let actions = move |editor: &mut Editor<'a>| {
                    editor.body.move_cursor(CursorMove::Head);
                    editor.body.delete_line_by_end();
                    editor.body.delete_newline();
                    editor.body.move_cursor(CursorMove::Down);
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
                    editor.body.move_cursor(CursorMove::Head);
                    editor.body.delete_line_by_end();
                    editor.body.delete_newline();
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
                let actions = move |editor: &mut Editor<'a>| {
                    editor.body.move_cursor(CursorMove::WordBack);
                    editor.body.delete_word();
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

        let mode_span = Span::styled(self.block_info, info_style);
        let key_hint_span = Span::styled(
            " <Alt-q> Quit <Alt-/s/l/n/d> Save/Load/New/Delete note <Alt-t> Edit title ",
            Style::default(),
        );

        let editor_block = Block::default()
            .title(Title::from(self.title).alignment(Alignment::Left))
            .title_style(Style::default().add_modifier(Modifier::BOLD))
            .title_bottom(Line::from(vec![mode_span, key_hint_span]))
            .borders(Borders::ALL)
            .border_set(border::Set {
                top_left: "╭",
                top_right: "┬",
                bottom_left: "╰",
                bottom_right: "┴",
                vertical_left: "│",
                vertical_right: "│",
                horizontal_top: "─",
                horizontal_bottom: "─",
            })
            .padding(Padding::new(1, 1, 1, 1));

        self.body.set_block(editor_block);

        self.body.widget().render(area, buf);
    }
}
