use ratatui::{
    layout::Alignment,
    prelude::{Buffer, Rect},
    style::{Color, Style, Stylize},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, Widget},
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
        let mut text = TextArea::default();
        text.set_cursor_line_style(Style::default());

        Self {
            text,
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
    fn render(mut self, area: Rect, buf: &mut Buffer)
    where
        Self: Sized,
    {
        let (title_clr, fg_clr) = match self.state {
            InputState::Active => (Color::Yellow, Color::default()),
            InputState::Submit => (Color::Yellow, Color::Red),
            InputState::Inactive => (Color::Blue, Color::Blue),
        };

        let (title, hint_text) = match self.action {
            InputAction::SubmitNoteTitle => (" Note title: ", " <Enter> submit title "),
        };

        let input_title = Span::styled(title, Style::default().bold().fg(title_clr));
        let input_hint = Span::styled(hint_text, Style::default().bold().fg(fg_clr));

        let input_block = Block::new()
            .title(Line::from(vec![input_title]))
            .title_alignment(Alignment::Left)
            .title_bottom(Line::from(vec![input_hint]))
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded);

        self.text.set_block(input_block);
        self.text.set_style(Style::default().fg(fg_clr));

        self.text.widget().render(area, buf);
    }
}
