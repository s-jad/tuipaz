use ratatui::{
    layout::Alignment,
    style::{Color, Modifier, Style},
    text::Line,
    widgets::{block::Title, Block, BorderType, Borders, Padding, Widget},
};
use tui_textarea::{CursorMove, Input, Key, TextArea};

#[derive(Debug, Clone)]
pub(crate) struct Editor<'a> {
    pub(crate) title: String,
    pub(crate) body: TextArea<'a>,
    pub(crate) mode: EditorMode,
    pub(crate) block_info: String,
    pub(crate) prev_cursor: CursorPosition,
}

#[derive(Debug, Clone)]
enum CursorPosition {
    Head,
    Middle(usize),
    End,
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
            EditorMode::Normal => match input {
                // Move left
                Input {
                    key: Key::Char('h'),
                    ..
                }
                | Input { key: Key::Left, .. } => self.body.move_cursor(CursorMove::Back),
                // Move Down
                Input {
                    key: Key::Char('j'),
                    ..
                } => self.body.move_cursor(CursorMove::Down),
                Input { key: Key::Down, .. } => self.body.move_cursor(CursorMove::Down),
                // Move Up
                Input {
                    key: Key::Char('k'),
                    ..
                }
                | Input { key: Key::Up, .. } => self.body.move_cursor(CursorMove::Up),
                // Move Right
                Input {
                    key: Key::Char('l'),
                    ..
                }
                | Input {
                    key: Key::Right, ..
                } => self.body.move_cursor(CursorMove::Forward),
                Input {
                    key: Key::Char('w'),
                    ..
                } => self.body.move_cursor(CursorMove::WordForward),
                Input {
                    key: Key::Char('b'),
                    ctrl: false,
                    ..
                } => self.body.move_cursor(CursorMove::WordBack),
                Input {
                    key: Key::Char('^'),
                    ..
                } => self.body.move_cursor(CursorMove::Head),
                Input {
                    key: Key::Char('$'),
                    ..
                } => self.body.move_cursor(CursorMove::End),
                Input {
                    key: Key::Char('D'),
                    ..
                } => {
                    self.body.delete_line_by_end();
                }
                Input {
                    key: Key::Char('C'),
                    ..
                } => {
                    self.body.delete_line_by_end();
                    self.set_mode(EditorMode::Insert);
                }
                Input {
                    key: Key::Char('p'),
                    ..
                } => {
                    self.body.paste();
                }
                Input {
                    key: Key::Char('u'),
                    ctrl: false,
                    ..
                } => {
                    self.body.undo();
                }
                Input {
                    key: Key::Char('r'),
                    ctrl: false,
                    ..
                } => {
                    self.body.redo();
                }
                Input {
                    key: Key::Char('x'),
                    ..
                } => {
                    self.body.delete_next_char();
                }
                Input {
                    key: Key::Char('a'),
                    ctrl: false,
                    ..
                } => {
                    self.set_mode(EditorMode::Insert);
                }
                Input {
                    key: Key::Char('A'),
                    ctrl: false,
                    ..
                } => {
                    self.body.move_cursor(CursorMove::End);
                    self.set_mode(EditorMode::Insert);
                }
                Input {
                    key: Key::Char('i'),
                    ctrl: false,
                    ..
                } => {
                    self.set_mode(EditorMode::Insert);
                }
                Input {
                    key: Key::Char('I'),
                    ctrl: false,
                    ..
                } => {
                    self.body.move_cursor(CursorMove::Head);
                    self.set_mode(EditorMode::Insert);
                }
                Input {
                    key: Key::Char('v'),
                    ctrl: false,
                    ..
                } => {
                    self.body.start_selection();
                    self.set_mode(EditorMode::Visual);
                }
                Input {
                    key: Key::Char('V'),
                    ctrl: false,
                    ..
                } => {
                    self.body.move_cursor(CursorMove::Head);
                    self.body.start_selection();
                    self.body.move_cursor(CursorMove::End);
                    self.set_mode(EditorMode::Visual);
                }
                _ => {}
            },
            EditorMode::Visual => match input {
                Input { key: Key::Esc, .. } => {
                    self.set_mode(EditorMode::Normal);
                }
                _ => {}
            },
        }
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
