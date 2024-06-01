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
    pub(crate) fn new(sidebar_open: bool, state: ComponentState) -> Self {
        let mut input = TextArea::new(vec!["".to_owned()], HashMap::new());
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
        let bottom_right = match self.sidebar_open {
            true => "┴",
            false => "╯",
        };


        let (mode_span, key_hint_span, cursor_style) = match self.state {
            ComponentState::Active => (
                Span::styled(" <| SEARCH |> ", Style::default().add_modifier(Modifier::BOLD).fg(Color::Green)),
                Span::styled(
                    " | <Alt-q> quit | <Alt-/s/l/n/d> save/load/new/delete | <Alt-t> edit title | ",
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

        let search_block = Block::default()
            .title_bottom(Line::from(vec![mode_span, key_hint_span]))
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
        self.input.set_placeholder_text("Search...");

        self.input.widget().render(area, buf);
    }
}
