use ratatui::{
    prelude::{Buffer, Rect},
    style::{Color, Style, Stylize},
    widgets::{Block, BorderType, Borders, Padding, Paragraph, Widget, Wrap},
};

#[derive(Debug, Clone, Copy)]
pub(crate) enum ButtonState {
    Active,
    Clicked,
    Inactive,
}

#[derive(Debug, Clone, Copy)]
pub(crate) enum ButtonAction {
    RenderMainScreen,
    RenderNewNoteScreen,
    RenderLoadNoteScreen,
}

#[derive(Debug, Clone)]
pub(crate) struct Button {
    pub(crate) text: String,
    state: ButtonState,
    action: ButtonAction,
}

impl Button {
    pub(crate) fn new(text: String, state: ButtonState, action: ButtonAction) -> Self {
        Self {
            text,
            state,
            action,
        }
    }

    pub(crate) fn set_state(&mut self, new_state: ButtonState) {
        self.state = new_state;
    }

    pub(crate) fn get_action(&self) -> ButtonAction {
        self.action
    }
}

impl Widget for Button {
    fn render(self, area: Rect, buf: &mut Buffer)
    where
        Self: Sized,
    {
        let (fg_clr, border_style) = match self.state {
            ButtonState::Active => (Color::default(), Style::default().bold()),
            ButtonState::Clicked => (Color::Yellow, Style::default().bold()),
            ButtonState::Inactive => (Color::Blue, Style::default().dim()),
        };

        let btn_block = Block::new()
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .border_style(border_style)
            .padding(Padding::new(2, 2, 2, 2));

        let p = Paragraph::new(self.text)
            .centered()
            .style(Style::default().fg(fg_clr))
            .block(btn_block)
            .wrap(Wrap { trim: true });

        p.render(area, buf);
    }
}
