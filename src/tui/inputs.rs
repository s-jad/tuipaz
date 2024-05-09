use ratatui::{
    layout::Alignment,
    prelude::{Buffer, Rect},
    style::{Color, Style, Stylize},
    widgets::{Block, BorderType, Borders, Paragraph, Widget, Wrap},
};
use tui_textarea::TextArea;

#[derive(Debug, Clone, Copy)]
pub(crate) enum InputState {
    Active,
    Submit,
    Inactive,
}

#[derive(Debug, Clone, Copy)]
pub(crate) enum InputAction {
    SubmitNoteTitle,
}

#[derive(Debug, Clone)]
pub(crate) struct UserInput<'i> {
    pub(crate) text: TextArea<'i>,
    state: InputState,
    action: InputAction,
}

impl<'i> UserInput<'i> {
    pub(crate) fn new(state: InputState, action: InputAction) -> Self {
        Self {
            text: TextArea::default(),
            state,
            action,
        }
    }

    pub(crate) fn set_state(&mut self, new_state: InputState) {
        self.state = new_state;
    }

    pub(crate) fn get_action(&mut self) -> InputAction {
        self.action
    }
}

impl Widget for UserInput<'_> {
    fn render(self, area: Rect, buf: &mut Buffer)
    where
        Self: Sized,
    {
        let (bg_clr, fg_clr, border_style) = match self.state {
            InputState::Active => (
                Color::Gray,
                Color::DarkGray,
                Style::default().bg(Color::Blue),
            ),
            InputState::Submit => (
                Color::Red,
                Color::Gray,
                Style::default().bold().bg(Color::Green),
            ),
            InputState::Inactive => (
                Color::DarkGray,
                Color::Gray,
                Style::default().bg(Color::LightBlue).dim(),
            ),
        };

        let title = match self.action {
            InputAction::SubmitNoteTitle => " Note title: ",
        };

        let input_block = Block::new()
            .title(title)
            .title_alignment(Alignment::Left)
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .border_style(border_style);

        let mut input = TextArea::default();
        input.set_block(input_block);
        input.set_style(Style::default().fg(fg_clr).bg(bg_clr));

        input.widget().render(area, buf);
    }
}
