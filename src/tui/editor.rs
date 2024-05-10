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
    pub(crate) block: Block<'a>,
    pub(crate) block_info: String,
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
            .title_bottom(Line::from(block_info.clone()))
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .padding(Padding::new(1, 1, 1, 1))
            .title_bottom(Line::from(" <Esc> Quit <Alt-s> Save Note "));

        body.set_block(editor_block.clone());

        Self {
            title,
            body,
            mode: EditorMode::Normal,
            block: editor_block,
            block_info,
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
                input => {
                    self.body.input(input);
                }
            },
            EditorMode::Normal => match input {
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
                    self.set_mode(EditorMode::Insert);
                }
                Input {
                    key: Key::Char('v'),
                    ctrl: false,
                    ..
                } => {
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
            EditorMode::Visual => {}
        }
    }
}

impl<'a> Widget for Editor<'a> {
    fn render(mut self, area: ratatui::prelude::Rect, buf: &mut ratatui::prelude::Buffer)
    where
        Self: Sized,
    {
        let tb = format!(" {}  <Esc> Quit <Alt-S> Save ", self.block_info);

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
