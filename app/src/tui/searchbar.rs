use std::collections::HashMap;

use ratatui::{widgets::{Borders, Block, Padding, Widget}, symbols};
use tuipaz_textarea::TextArea;

use super::app::ComponentState;

#[derive(Debug, Clone)]
pub(crate) struct Searchbar<'a> {
    pub(crate) input: TextArea<'a>,
    pub(crate) sidebar_open: bool,
    pub(crate) state: ComponentState,
}

impl<'a> Searchbar<'a> {
    pub(crate) fn new(sidebar_open: bool, state: ComponentState) -> Self {

        let bottom_right = match sidebar_open {
            true => "┴",
            false => "╯",
        };

        let search_block = Block::default()
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
            .padding(Padding::new(0, 0, 1, 1));

        let mut input = TextArea::new(vec!["".to_owned()], HashMap::new());
        input.set_placeholder_text("Search...");
        input.set_block(search_block);

        Self {
            input,
            sidebar_open,
            state
        }
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

        let search_block = Block::default()
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
            .padding(Padding::new(0, 0, 1, 1));

        self.input.set_block(search_block);

        self.input.widget().render(area, buf);
    }
}
