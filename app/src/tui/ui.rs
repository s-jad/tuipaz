use log::info;

use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout},
    style::{Color, Style, Modifier},
    text::{Span, Line, Text},
    widgets::{Block, BorderType, Borders, Clear, Padding, Paragraph, Widget, Wrap},
    Frame,
};

use super::{
    app::{App, Screen, SearchbarState},
    user_messages::centered_rect,
};

pub(crate) fn ui(app: &mut App, frame: &mut Frame) {
    match app.current_screen {
        Screen::Welcome => render_welcome_screen(app, frame),
        Screen::Main => render_main_screen(app, frame),
        Screen::NewNote => render_new_note_screen(app, frame),
        Screen::NewLinkedNote => render_new_linked_note_screen(app, frame),
        Screen::LoadNote => render_load_note_screen(app, frame),
        Screen::DeleteNoteConfirmation => render_popup(app, frame),
        Screen::Popup => render_popup(app, frame),
        Screen::Exiting => render_exit_screen(frame),
    }
}

fn render_welcome_screen(app: &mut App, frame: &mut Frame) {
    let area = frame.size();
    let buf = frame.buffer_mut();

    let two_btn_split = [
        Constraint::Min(4),
        Constraint::Percentage(25),
        Constraint::Percentage(5),
        Constraint::Percentage(25),
        Constraint::Min(4),
    ];

    // Split the main area into three sections:
    // - The top section for title.
    // - The middle section for button labels.
    // - The bottom section for buttons.
    let layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage(10), // Top padding
            Constraint::Percentage(40), // Title section
            Constraint::Percentage(20), // buttons
            Constraint::Percentage(30), // Bottom padding
        ])
        .split(area);

    let welcome_block = Block::default()
        .borders(Borders::NONE)
        .padding(Padding { left: 0, right: 0, top: 5, bottom: 0 })
        .style(Style::default())
        .title_alignment(Alignment::Center);

    let welcome = Paragraph::new(vec![
        Line::styled("████████╗██╗   ██╗██╗██████╗  █████╗ ███████╗", Style::default()),
        Line::styled("╚══██╔══╝██║   ██║██║██╔══██╗██╔══██╗╚══███╔╝", Style::default()),
        Line::styled("   ██║   ██║   ██║██║██████╔╝███████║  ███╔╝ ", Style::default()),
        Line::styled("   ██║   ██║   ██║██║██╔═══╝ ██╔══██║ ███╔╝  ", Style::default()),
        Line::styled("   ██║   ╚██████╔╝██║██║     ██║  ██║███████╗", Style::default()),
        Line::styled("   ╚═╝    ╚═════╝ ╚═╝╚═╝     ╚═╝  ╚═╝╚══════╝", Style::default()),
    ]).block(welcome_block)
    .alignment(Alignment::Center);

    welcome.render(layout[1], buf);

    let btn_layout = Layout::default()
        .direction(Direction::Horizontal)
        .constraints(two_btn_split)
        .split(layout[2]);

    let new_note_btn = app.btns[0].clone();
    new_note_btn.render(btn_layout[1], buf);

    let load_note_btn = app.btns[1].clone();
    load_note_btn.render(btn_layout[3], buf);
}

fn render_main_screen(app: &mut App, frame: &mut Frame) {
    let area = frame.size();
    let buf = frame.buffer_mut();

    let h_layout = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(100 - app.sidebar_size),
            Constraint::Percentage(app.sidebar_size),
        ])
        .split(area);
    
    let searchbar_size = match app.searchbar_state {
        SearchbarState::Open => 8,
        SearchbarState::Hidden => 0,
    };

    let v_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage(100 - searchbar_size),
            Constraint::Percentage(searchbar_size),
        ])
        .split(h_layout[0]);
    
    app.editor.clone().render(v_layout[0], buf);
    app.searchbar.clone().render(v_layout[1], buf);
    app.note_list.clone().render(h_layout[1], buf);
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
    
    let exit_question = Line::styled(" Exit Tuipaz? ", Style::default());
    let before_yn = Span::styled("(", Style::default());
    let y_span = Span::styled("y", Style::default().fg(Color::Green).add_modifier(Modifier::BOLD));
    let between_yn = Span::styled("/", Style::default());
    let n_span = Span::styled("n", Style::default().fg(Color::Red).add_modifier(Modifier::BOLD));
    let after_yn = Span::styled(")", Style::default());
    let spacer = Line::styled("\n", Style::default());
    let yn_line = Line::from(vec![before_yn, y_span, between_yn, n_span, after_yn]);

    Paragraph::new(vec![exit_question, spacer, yn_line])
        .block(popup_block)
        .alignment(Alignment::Center)
        .wrap(Wrap { trim: false })
        .render(centered_rect(26, 20, area), buf);
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
