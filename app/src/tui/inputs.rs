use ratatui::{
    layout::Alignment,
    prelude::{Buffer, Rect},
    style::{Color, Style, Stylize},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, Widget},
};
use tuipaz_textarea::TextArea;

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub(crate) enum InputState {
    Active,
    Submit,
    Inactive,
    Error,
}

#[derive(Debug, Clone, Copy)]
pub(crate) enum InputAction {
    NewNoteTitle,
    NewNote,
    NewLinkedNote,
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
        text.set_placeholder_text("Enter a title...");
        text.set_placeholder_style(Style::default().dim());

        Self {
            text,
            state,
            action,
        }
    }

    pub(crate) fn set_state(&mut self, new_state: InputState) {
        self.state = new_state;
    }

    pub(crate) fn get_state(&mut self) -> InputState {
        self.state
    }

    pub(crate) fn set_action(&mut self, new_action: InputAction) {
        self.action = new_action
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
        let (title_clr, fg_clr, border_style) = match self.state {
            InputState::Active => (Color::Yellow, Color::default(), Style::default().bold()),
            InputState::Submit => (Color::Yellow, Color::Red, Style::default().bold()),
            InputState::Inactive => (Color::Gray, Color::Gray, Style::default().dim()),
            InputState::Error => (Color::Red, Color::Red, Style::default().bold()),
        };

        let (title_span, input_hint) = match (self.action, self.state) {
            (
                InputAction::NewNoteTitle | InputAction::NewNote | InputAction::NewLinkedNote,
                InputState::Error,
            ) => (
                Span::styled(
                    format!(" Error: {:?} already exists ", self.text.lines()),
                    Style::default().bold().fg(Color::Red),
                ),
                Span::styled(
                    " Please choose a different title ",
                    Style::default().bold().fg(Color::Red),
                ),
            ),
            (InputAction::NewNoteTitle | InputAction::NewNote | InputAction::NewLinkedNote, _) => (
                Span::styled(" New Note ", Style::default().bold().fg(title_clr)),
                Span::styled(
                    " <Esc> return to prev screen <Enter> submit title ",
                    Style::default().bold().fg(fg_clr),
                ),
            ),
        };

        let input_block = Block::new()
            .title(Line::from(vec![title_span]))
            .title_alignment(Alignment::Left)
            .title_bottom(Line::from(vec![input_hint]))
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .border_style(border_style);

        self.text.set_block(input_block);
        self.text.set_style(Style::default().fg(fg_clr));

        self.text.widget().render(area, buf);
    }
}
