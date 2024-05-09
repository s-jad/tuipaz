use ratatui::{
    layout::Alignment,
    prelude::{Buffer, Rect},
    style::{Color, Style, Stylize},
    widgets::{Block, BorderType, Borders, Paragraph, Widget, Wrap},
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

    pub(crate) fn activate(&mut self) {
        self.state = ButtonState::Active;
    }

    pub(crate) fn deactivate(&mut self) {
        self.state = ButtonState::Inactive;
    }

    pub(crate) fn clicked(&mut self) {
        self.state = ButtonState::Clicked;
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
        let (bg_clr, fg_clr, border_style) = match self.state {
            ButtonState::Active => (
                Color::Gray,
                Color::DarkGray,
                Style::default().bg(Color::Blue),
            ),
            ButtonState::Clicked => (
                Color::Red,
                Color::Gray,
                Style::default().bold().bg(Color::Green),
            ),
            ButtonState::Inactive => (
                Color::DarkGray,
                Color::Gray,
                Style::default().bg(Color::LightBlue).dim(),
            ),
        };

        let btn_block = Block::new()
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .border_style(border_style);

        let p = Paragraph::new(self.text)
            .alignment(Alignment::Center)
            .style(Style::default().fg(fg_clr).bg(bg_clr))
            .block(btn_block)
            .wrap(Wrap { trim: true });

        p.render(area, buf);
    }
}
