use std::time::{Duration, Instant};

use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout},
    prelude::{Buffer, Rect},
    style::{Style, Stylize},
    text::Line,
    widgets::{block::Title, Block, Borders, Paragraph, Widget, Wrap},
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
    pub(crate) show: bool,
    pub(crate) start: Instant,
    pub(crate) duration: Duration,
    pub(crate) typ: MessageType,
}

impl UserMessage {
    pub(crate) fn welcome() -> Self {
        Self {
            msg: "Welcome to Tuipaz!".to_string(),
            show: true,
            start: Instant::now(),
            duration: Duration::from_secs(3),
            typ: MessageType::Info,
        }
    }

    pub(crate) fn new(msg: String, show: bool, secs: u64, typ: MessageType) -> Self {
        Self {
            msg,
            show,
            start: Instant::now(),
            duration: Duration::from_secs(secs),
            typ,
        }
    }
}

impl Widget for UserMessage {
    fn render(self, area: Rect, buf: &mut Buffer)
    where
        Self: Sized,
    {
        let (title, style) = match self.typ {
            MessageType::Info => ("Info".to_string(), Style::new().blue().bold()),
            MessageType::Warning => ("Warning".to_string(), Style::new().yellow().bold()),
            MessageType::Error => ("Error".to_string(), Style::new().red().bold()),
        };

        let popup_block = Block::default()
            .title(Title::from(title).alignment(Alignment::Center))
            .borders(Borders::ALL);

        let popup_text = vec![Line::from(self.msg).style(style)];

        Paragraph::new(popup_text)
            .block(popup_block)
            .wrap(Wrap { trim: true })
            .render(centered_rect(40, 40, area), buf);
    }
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
