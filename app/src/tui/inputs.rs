use ratatui::{
    layout::Alignment,
    prelude::{Buffer, Rect},
    style::{Color, Style, Stylize},
    text::{Line, Span},
    widgets::{Block, BorderType, Padding, Borders, Widget},
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
    NoteTitle,
    Note,
    LinkedNote,
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
        let (title_style, hint_style, text_style) = match self.state {
            InputState::Active => (Style::default().bold().fg(Color::Yellow), Style::default().bold(), Style::default()),
            InputState::Submit => (Style::default().bold().fg(Color::Blue), Style::default().bold(), Style::default()),
            InputState::Inactive => (Style::default().bold().dim(), Style::default().bold().dim(), Style::default().dim()),
            InputState::Error => (Style::default().bold().fg(Color::Red), Style::default().bold().fg(Color::Red), Style::default()),
        };

        let (title_span, input_hint) = match (self.action, self.state) {
            (
                InputAction::NoteTitle | InputAction::Note | InputAction::LinkedNote,
                InputState::Error,
            ) => (
                Span::styled(
                    format!(" Error: {:?} already exists ", self.text.lines()),
                    title_style,
                ),
                Span::styled(
                    " Please choose a different title ",
                    hint_style,
                ),
            ),
            (InputAction::NoteTitle | InputAction::Note | InputAction::LinkedNote, _) => (
                Span::styled(" New Note ", title_style),
                Span::styled(
                    " <Esc> return to prev screen <Enter> submit title ",
                    hint_style,
                ),
            ),
        };
        
        let border_style = text_style.bold();

        let input_block = Block::new()
            .title(Line::from(vec![title_span]))
            .title_alignment(Alignment::Left)
            .title_bottom(Line::from(vec![input_hint]))
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .border_style(border_style)
            .padding(Padding { left: 1, right: 1, top: 1, bottom: 1 });

        self.text.set_block(input_block);
        self.text.set_style(text_style);

        self.text.widget().render(area, buf);
    }
}
