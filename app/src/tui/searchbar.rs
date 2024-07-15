use log::info;
use ratatui::{
    style::{Color, Modifier, Style},
    symbols,
    text::{Line, Span},
    widgets::{Block, Borders, Padding, Widget},
};
use tuipaz_textarea::{Input, Key, TextInput};

use super::app::ComponentState;

#[derive(Debug, Clone)]
pub(crate) struct Searchbar<'a> {
    pub(crate) input: TextInput<'a>,
    pub(crate) sidebar_open: bool,
    pub(crate) state: ComponentState,
    pub(crate) theme: SearchbarTheme,
}

#[derive(Debug, Clone)]
pub(crate) struct SearchbarTheme {
    pub(crate) text: Color,
    pub(crate) search_mode: Color,
    pub(crate) borders: Color,
}

impl<'a> Searchbar<'a> {
    pub(crate) fn new(
        sidebar_open: bool,
        state: ComponentState,
        max_col: u16,
        theme: SearchbarTheme,
    ) -> Self {
        let input = TextInput::new("".to_owned(), max_col, theme.text, "Search...".to_owned());

        Self {
            input,
            sidebar_open,
            state,
            theme,
        }
    }

    pub(crate) fn set_state(&mut self, new_state: ComponentState) {
        self.state = new_state;
    }

    pub(crate) fn clear_search(&mut self) {
        self.input.clear();
        self.input.cursor = (0, 0);
    }

    pub(crate) fn get_search_text(&self) -> &str {
        self.input.get_text()
    }

    pub(crate) fn handle_input(&mut self, input: Input) {
        match input {
            Input { key: Key::Esc, .. } => {
                self.clear_search();
                self.set_state(ComponentState::Inactive);
            }
            _ => {
                self.input.input(input);
            }
        }
    }
}

impl<'a> Widget for Searchbar<'a> {
    fn render(mut self, area: ratatui::prelude::Rect, buf: &mut ratatui::prelude::Buffer)
    where
        Self: Sized,
    {
        let (file_explorer_span_text, bottom_right) = match self.sidebar_open {
            true => ("".to_owned(), "┴"),
            false => (" <Alt-f> show files ".to_owned(), "╯"),
        };

        let (mode_span, key_hint_span, cursor_style) = match self.state {
            ComponentState::Active => (
                Span::styled(
                    " <| SEARCH |>",
                    Style::default()
                        .add_modifier(Modifier::BOLD)
                        .fg(self.theme.search_mode),
                ),
                Span::styled(
                    " | <Alt-q> quit | <Alt-s/l/d/n> save/load/delete/new | <Alt-t> edit title | ",
                    Style::default().add_modifier(Modifier::BOLD),
                ),
                Style::default().add_modifier(Modifier::REVERSED),
            ),
            _ => (
                Span::styled("", Style::default()),
                Span::styled("", Style::default()),
                Style::default(),
            ),
        };

        let ms_len = mode_span.content.len();
        let kh_len = key_hint_span.content.len();
        let fh_len = file_explorer_span_text.len();
        let tb_bottom_len = (ms_len + kh_len + fh_len) as u16;
        let padding_len = match self.sidebar_open {
            true => 0,
            false => (area.width - tb_bottom_len - 5) as usize,
        };

        let prefix_padding = Span::styled(
            "─".to_owned(),
            Style::default().add_modifier(Modifier::BOLD),
        );
        let padding = Span::styled(
            "─".repeat(padding_len),
            Style::default().add_modifier(Modifier::BOLD),
        );
        let file_explorer_span = Span::styled(
            file_explorer_span_text,
            Style::default().add_modifier(Modifier::BOLD),
        );

        let search_block = Block::default()
            .title_bottom(Line::from(vec![
                prefix_padding,
                mode_span,
                key_hint_span,
                padding,
                file_explorer_span,
            ]))
            .borders(Borders::LEFT | Borders::RIGHT | Borders::BOTTOM)
            .border_set(symbols::border::Set {
                top_left: " ",
                top_right: " ",
                bottom_left: "╰",
                bottom_right,
                vertical_left: "│",
                vertical_right: "│",
                horizontal_bottom: "─",
                horizontal_top: " ",
            })
            .border_style(Style::default().fg(self.theme.borders))
            .padding(Padding {
                left: 1,
                right: 1,
                top: 0,
                bottom: 0,
            });

        self.input.set_block(search_block);
        self.input.set_text_style(self.theme.text);
        self.input.set_cursor_style(cursor_style);
        self.input.set_placeholder_text("...");

        self.input.widget().render(area, buf);
    }
}
