use ratatui::{
    prelude::{Buffer, Rect},
    style::{Color, Style, Stylize},
    widgets::{Block, BorderType, Borders, Padding, Paragraph, Widget, Wrap},
};

use super::{app::ComponentState, user_messages::centered_rect};

#[derive(Debug, Clone, Copy)]
pub(crate) enum ButtonAction {
    RenderMainScreen,
    RenderNewNoteScreen,
    RenderLoadNoteScreen,
}

#[derive(Debug, Clone)]
pub(crate) struct Button {
    pub(crate) text: String,
    state: ComponentState,
    action: ButtonAction,
}

impl Button {
    pub(crate) fn new(text: String, state: ComponentState, action: ButtonAction) -> Self {
        Self {
            text,
            state,
            action,
        }
    }

    pub(crate) fn set_state(&mut self, new_state: ComponentState) {
        self.state = new_state;
    }

    pub(crate) fn get_state(&self) -> ComponentState {
        self.state
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
            ComponentState::Active => (Color::default(), Style::default().bold()),
            ComponentState::Error => (Color::Red, Style::default().bold()),
            ComponentState::Inactive => (Color::Blue, Style::default().dim()),
            ComponentState::Unavailable => (Color::Gray, Style::default().dim()),
        };

        let btn_block = Block::new()
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .border_style(border_style)
            .padding(Padding::new(2, 2, 2, 1));

        let p = Paragraph::new(self.text)
            .centered()
            .style(Style::default().fg(fg_clr))
            .block(btn_block)
            .wrap(Wrap { trim: true });

        p.render(centered_rect(80, 100, area), buf);
    }
}
