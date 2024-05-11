use ratatui::{
    layout::Alignment,
    style::{Color, Modifier, Style},
    text::Line,
    widgets::{block::Title, Block, BorderType, Borders, Padding, Widget},
};
use tui_textarea::{CursorMove, Input, Key, TextArea};

const DELETE_COMMANDS: [char; 7] = ['d', 'w', 'b', 'j', 'k', 'l', 'h'];
const YANK_COMMANDS: [char; 6] = ['w', 'b', 'j', 'k', 'l', 'h'];
const GOTO_COMMANDS: [char; 1] = ['g'];

#[derive(Debug, Clone)]
pub(crate) struct Editor<'a> {
    pub(crate) title: String,
    pub(crate) body: TextArea<'a>,
    pub(crate) mode: EditorMode,
    pub(crate) block_info: String,
    pub(crate) prev_cursor: CursorPosition,
    pub(crate) num_buf: Vec<u32>,
    pub(crate) cmd_buf: String,
    pub(crate) cmd_state: CommandState,
}

#[derive(Debug, Clone)]
enum CursorPosition {
    Head,
    Middle(usize),
    End,
}

#[derive(Debug, Clone, Eq, PartialEq)]
enum CommandState {
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
    pub(crate) fn new(title: String) -> Self {
        let mut body = TextArea::default();
        body.set_cursor_line_style(Style::default());
        body.set_selection_style(Style::default().bg(Color::Red));
        body.set_max_histories(1000);

        let block_info = " <| NORMAL |> ".to_string();

        let editor_block = Block::default()
            .title(Title::from(title.clone()).alignment(Alignment::Left))
            .title_style(Style::default().add_modifier(Modifier::BOLD))
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .padding(Padding::new(1, 1, 1, 1))
            .title_bottom(Line::from(
                " <| NORMAL |> <Alt-q/s/l/n> Quit/Save/Load/New ",
            ));

        body.set_block(editor_block.clone());

        Self {
            title,
            body,
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
            EditorMode::Insert => " <| INSERT |> ".to_owned(),
            EditorMode::Normal => " <| NORMAL |> ".to_owned(),
            EditorMode::Visual => " <| VISUAL |> ".to_owned(),
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
                    self.body.move_cursor(CursorMove::Back)
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
                    self.body.move_cursor(CursorMove::Down);
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
                    self.body.move_cursor(CursorMove::Up);
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
                    self.body.move_cursor(CursorMove::Forward);
                }
                (
                    Input {
                        key: Key::Char('w'),
                        ..
                    },
                    CommandState::NoCommand,
                ) => {
                    self.body.move_cursor(CursorMove::WordForward);
                }
                (
                    Input {
                        key: Key::Char('b'),
                        ctrl: false,
                        ..
                    },
                    CommandState::NoCommand,
                ) => {
                    self.body.move_cursor(CursorMove::WordBack);
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
                    self.body.delete_next_char();
                }
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
                    self.body.start_selection();
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
                    self.body.start_selection();
                    self.body.move_cursor(CursorMove::End);
                    self.set_mode(EditorMode::Visual);
                }
                (input, CommandState::NoCommand) => self.prime_command_state(input),
            },
            EditorMode::Visual => match input {
                Input { key: Key::Esc, .. } => {
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
        match input.key {
            Key::Char(c) => {
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
            _ => {}
        }
    }

    fn execute_delete(&mut self, modifier: char) {
        match modifier {
            'd' | 'j' => {
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
                    self.body.move_cursor(CursorMove::Down);
                }
                self.cmd_buf.clear();
            }
            'k' => {
                let num_buf_len = self.num_buf.len() as u32;
                if num_buf_len != 0 {
                    let num = self.get_num_from_buf(num_buf_len);
                    for _ in 0..num {
                        self.body.move_cursor(CursorMove::Head);
                        self.body.delete_line_by_end();
                        self.body.delete_newline();
                        //self.body.move_cursor(CursorMove::Up);
                    }
                    self.num_buf.clear();
                } else {
                    self.body.move_cursor(CursorMove::Head);
                    self.body.delete_line_by_end();
                    self.body.delete_newline();
                    //self.body.move_cursor(CursorMove::Up);
                }
                self.cmd_buf.clear();
            }
            'h' => {
                let num_buf_len = self.num_buf.len() as u32;
                if num_buf_len != 0 {
                    let num = self.get_num_from_buf(num_buf_len);
                    for _ in 0..num {
                        self.body.delete_char();
                    }
                    self.num_buf.clear();
                } else {
                    self.body.delete_char();
                }
                self.cmd_buf.clear();
            }
            'l' => {
                let num_buf_len = self.num_buf.len() as u32;
                if num_buf_len != 0 {
                    let num = self.get_num_from_buf(num_buf_len);
                    for _ in 0..num {
                        self.body.delete_next_char();
                    }
                    self.num_buf.clear();
                } else {
                    self.body.delete_next_char();
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
                    self.body.move_cursor(CursorMove::Jump(num as u16, 0));
                    self.num_buf.clear();
                } else {
                    self.body.move_cursor(CursorMove::Top);
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

    fn get_num_from_buf(&self, num_buf_len: u32) -> u16 {
        self.num_buf
            .iter()
            .enumerate()
            .fold(0u32, |mut acc, (idx, n)| {
                acc += std::cmp::max(
                    *n as u32,
                    (10u32).pow(num_buf_len - 1 - idx as u32) * *n as u32,
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
        let tb = format!(" {}  <Alt-q/s/l/n> Quit/Save/Load/New ", self.block_info);

        let editor_block = Block::default()
            .title(Title::from(self.title).alignment(Alignment::Left))
            .title_style(Style::default().add_modifier(Modifier::BOLD))
            .title_bottom(Line::from(tb))
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .padding(Padding::new(1, 1, 1, 1));

        self.body.set_block(editor_block);

        self.body.widget().render(area, buf);
    }
}
