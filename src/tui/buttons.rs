use ratatui::{
    layout::Alignment,
    prelude::{Buffer, Rect},
    style::{Color, Style, Stylize},
    widgets::{Block, Borders, Paragraph, Widget, Wrap},
};

#[derive(Debug, Clone)]
pub(crate) enum ButtonState {
    Active,
    Clicked,
    Inactive,
}

#[derive(Debug, Clone)]
pub(crate) struct Button {
    pub(crate) text: String,
    state: ButtonState,
}

impl Button {
    pub(crate) fn new(text: String, state: ButtonState) -> Self {
        Self { text, state }
    }

    pub(crate) fn activate(&mut self) {
        self.state = ButtonState::Active;
    }

    pub(crate) fn deactivate(&mut self) {
        self.state = ButtonState::Inactive;
    }
}

impl Widget for Button {
    fn render(self, area: Rect, buf: &mut Buffer)
    where
        Self: Sized,
    {
        let border_style = match self.state {
            ButtonState::Active => Style::default().bg(Color::Blue),
            ButtonState::Clicked => Style::default().bold().bg(Color::Green),
            ButtonState::Inactive => Style::default().bg(Color::LightBlue).dim(),
        };

        let btn_block = Block::new()
            .borders(Borders::ALL)
            .border_style(border_style);

        Paragraph::new(self.text)
            .alignment(Alignment::Center)
            .block(btn_block)
            .wrap(Wrap { trim: true })
            .render(area, buf);
    }
}
