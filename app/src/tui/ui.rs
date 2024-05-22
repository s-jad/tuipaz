use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Text},
    widgets::{block::Title, Block, BorderType, Borders, Clear, Padding, Paragraph, Widget, Wrap},
    Frame,
};

use super::{
    app::{App, Screen},
    user_messages::centered_rect,
};

pub(crate) fn ui(app: &mut App, frame: &mut Frame) {
    match app.current_screen {
        Screen::Welcome => render_welcome_screen(app, frame),
        Screen::Main => render_main_screen(app, frame),
        Screen::NewNote => render_new_note_screen(app, frame),
        Screen::NewLinkedNote => render_new_linked_note_screen(app, frame),
        Screen::LoadNote => render_load_note_screen(app, frame),
        Screen::Popup => render_popup(app, frame),
        Screen::Exiting => render_exit_screen(frame),
    }
}

fn render_welcome_screen(app: &mut App, frame: &mut Frame) {
    let area = frame.size();
    let buf = frame.buffer_mut();

    let two_btn_split = [
        Constraint::Min(4),
        Constraint::Percentage(30),
        Constraint::Min(2),
        Constraint::Percentage(30),
        Constraint::Min(4),
    ];

    // Split the main area into three sections:
    // - The top section for title.
    // - The middle section for button labels.
    // - The bottom section for buttons.
    let layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage(40), // Title section
            Constraint::Percentage(20), // buttons
            Constraint::Percentage(40), // Bottom padding
        ])
        .split(area);

    let welcome_block = Block::default()
        .title("Welcome to Tuipaz!")
        .borders(Borders::NONE)
        .style(Style::default())
        .title_alignment(Alignment::Center);

    welcome_block.render(layout[0], buf);

    let btn_layout = Layout::default()
        .direction(Direction::Horizontal)
        .constraints(two_btn_split)
        .split(layout[1]);

    let new_note_btn = app.btns[0].clone();
    new_note_btn.render(btn_layout[1], buf);

    let load_note_btn = app.btns[1].clone();
    load_note_btn.render(btn_layout[3], buf);
}

fn render_main_screen(app: &mut App, frame: &mut Frame) {
    let area = frame.size();
    let buf = frame.buffer_mut();

    let layout = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(100 - app.sidebar_size),
            Constraint::Percentage(app.sidebar_size),
        ])
        .split(area);

    app.editor.clone().render(layout[0], buf);

    let files_block = Block::default()
        .title(Title::from(" File Explorer ").alignment(Alignment::Center))
        .title_style(Style::default().add_modifier(Modifier::BOLD))
        .padding(Padding {
            left: 1,
            right: 1,
            top: 0,
            bottom: 0,
        })
        .borders(Borders::TOP | Borders::RIGHT | Borders::BOTTOM)
        .border_type(BorderType::Rounded);

    let files_text = vec![Line::from("File explorer sidebar").style(Style::default())];

    Paragraph::new(files_text)
        .block(files_block)
        .wrap(Wrap { trim: true })
        .render(layout[1], buf);
}

fn render_popup(app: &mut App<'_>, frame: &mut Frame) {
    let area = frame.size();
    let buf = frame.buffer_mut();
    app.user_msg.clone().render(area, buf);
}

fn render_exit_screen(frame: &mut Frame) {
    let area = frame.size();
    frame.render_widget(Clear, area);
    let buf = frame.buffer_mut();

    let popup_block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .padding(Padding::new(0, 0, 1, 1))
        .style(Style::default());

    let exit_text = Text::styled(" Exit Tuipaz? (y/n) ", Style::default().fg(Color::DarkGray));

    Paragraph::new(exit_text)
        .block(popup_block)
        .alignment(Alignment::Center)
        .wrap(Wrap { trim: false })
        .render(centered_rect(25, 15, area), buf);
}

fn render_load_note_screen(app: &mut App<'_>, frame: &mut Frame) {
    let area = frame.size();
    let buf = frame.buffer_mut();

    app.note_list
        .clone()
        .render(centered_rect(60, 100, area), buf);
}

fn render_new_note_screen(app: &mut App<'_>, frame: &mut Frame) {
    let area = frame.size();
    let buf = frame.buffer_mut();

    app.user_input
        .clone()
        .render(centered_rect(50, 20, area), buf);
}

fn render_new_linked_note_screen(app: &mut App<'_>, frame: &mut Frame) {
    let area = frame.size();
    let buf = frame.buffer_mut();

    let layout = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(area);

    app.user_input
        .clone()
        .render(centered_rect(90, 20, layout[0]), buf);

    app.note_list
        .clone()
        .render(centered_rect(90, 20, layout[1]), buf);
}
