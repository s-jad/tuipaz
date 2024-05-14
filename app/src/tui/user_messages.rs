use std::time::{Duration, Instant};

use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout},
    prelude::{Buffer, Rect},
    style::{Style, Stylize},
    text::Line,
    widgets::{block::Title, Block, BorderType, Borders, Padding, Paragraph, Widget, Wrap},
};

#[derive(Debug, Clone)]
pub(crate) enum MessageType {
    Info,
    Warning,
    Error,
}

#[derive(Debug, Clone)]
pub(crate) struct UserMessage {
    pub(crate) msg: String,
    pub(crate) typ: MessageType,
}

impl UserMessage {
    pub(crate) fn welcome() -> Self {
        Self {
            msg: "Welcome to Tuipaz!".to_string(),
            typ: MessageType::Info,
        }
    }

    pub(crate) fn new(msg: String, typ: MessageType) -> Self {
        Self { msg, typ }
    }
}

impl Widget for UserMessage {
    fn render(self, area: Rect, buf: &mut Buffer)
    where
        Self: Sized,
    {
        let (title, style) = match self.typ {
            MessageType::Info => (" Info ".to_string(), Style::new().blue().bold()),
            MessageType::Warning => (" Warning ".to_string(), Style::new().yellow().bold()),
            MessageType::Error => (" Error ".to_string(), Style::new().red().bold()),
        };

        let bottom_title = " <Esc> Return to previous screen ".to_string();
        let bt_width = (bottom_title.len() + 2) as u16;

        let popup_block = Block::default()
            .title(Title::from(title).alignment(Alignment::Center))
            .borders(Borders::ALL)
            .title_bottom(Line::from(bottom_title))
            .padding(Padding::vertical(1))
            .border_type(BorderType::Rounded);

        let msg_width = self
            .msg
            .lines()
            .fold(0usize, |acc, l| std::cmp::max(l.len(), acc)) as u16;

        let width = std::cmp::max(msg_width, bt_width);

        let popup_text = self
            .msg
            .lines()
            .map(|l| Line::from(l).style(style).alignment(Alignment::Center))
            .collect::<Vec<Line>>();

        let msg_height = (popup_text.len() as f32 * 1.2).ceil() as u16 + 4;

        Paragraph::new(popup_text)
            .block(popup_block)
            .wrap(Wrap { trim: true })
            .render(centered_msg(width, msg_height, area), buf);
    }
}

fn centered_msg(msg_x: u16, msg_y: u16, r: Rect) -> Rect {
    // Cut the given rectangle into three vertical pieces
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Min((r.height - msg_y) / 2),
            Constraint::Min(msg_y),
            Constraint::Min((r.height - msg_y) / 2),
        ])
        .split(r);

    // Then cut the middle vertical piece into three width-wise pieces
    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Min((r.width - msg_x) / 2),
            Constraint::Min(msg_x),
            Constraint::Min((r.width - msg_x) / 2),
        ])
        .split(popup_layout[1])[1] // Return the middle chunk
}

/// helper function to create a centered rect using up certain percentage of the available rect `r`
pub(crate) fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    // Cut the given rectangle into three vertical pieces
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);

    // Then cut the middle vertical piece into three width-wise pieces
    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1] // Return the middle chunk
}
