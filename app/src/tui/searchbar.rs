use std::collections::HashMap;

use log::info;
use ratatui::{widgets::{Borders, Block, Padding, Widget}, symbols, style::{Modifier, Style, Color}, text::{Span, Line}};
use tuipaz_textarea::{TextArea, CursorMove};

use super::app::ComponentState;

#[derive(Debug, Clone)]
pub(crate) struct Searchbar<'a> {
    pub(crate) input: TextArea<'a>,
    pub(crate) sidebar_open: bool,
    pub(crate) state: ComponentState,
}

impl<'a> Searchbar<'a> {
    pub(crate) fn new(sidebar_open: bool, state: ComponentState, max_col: u16) -> Self {
        let mut input = TextArea::new(vec!["".to_owned()], HashMap::new(), max_col);
        input.set_placeholder_text("Search...");

        Self {
            input,
            sidebar_open,
            state
        }
    }

    pub(crate) fn set_state(&mut self, new_state: ComponentState) {
        self.state = new_state;
    }

    pub(crate) fn search(&mut self) {
        self.input.clear_lines();
        self.input.move_cursor(CursorMove::Head);
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
                Span::styled(" <| SEARCH |>", Style::default().add_modifier(Modifier::BOLD).fg(Color::Green)),
                Span::styled(
                    " | <Alt-q> quit | <Alt-s/l/d/n> save/load/delete/new | <Alt-t> edit title | ",
                    Style::default().add_modifier(Modifier::BOLD),
                ),
                Style::default().add_modifier(Modifier::REVERSED),
            ),
            _ => (
                Span::styled("", Style::default()),
                Span::styled(
                    "",
                    Style::default(),
                ),
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

        let prefix_padding = Span::styled("─".to_owned(), Style::default().add_modifier(Modifier::BOLD));
        let padding = Span::styled("─".repeat(padding_len), Style::default().add_modifier(Modifier::BOLD));
        let file_explorer_span = Span::styled(file_explorer_span_text, Style::default().add_modifier(Modifier::BOLD));

        let search_block = Block::default()
            .title_bottom(Line::from(vec![
                prefix_padding, 
                mode_span, 
                key_hint_span,
                padding,
                file_explorer_span
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
            .padding(Padding { left: 1, right: 1, top: 0, bottom: 0 });

        self.input.set_block(search_block);
        self.input.set_cursor_style(cursor_style);
        self.input.set_placeholder_text("...");

        self.input.widget().render(area, buf);
    }
}
