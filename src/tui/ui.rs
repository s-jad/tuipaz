use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Text},
    widgets::{block::Title, Block, BorderType, Borders, Clear, Padding, Paragraph, Widget, Wrap},
    Frame,
};
use tui_textarea::TextArea;

use super::{
    app::{App, Screen},
    inputs::UserInput,
    user_messages::centered_rect,
};

pub(crate) fn ui(app: &mut App, frame: &mut Frame) {
    match app.current_screen {
        Screen::Welcome => render_welcome_screen(app, frame),
        Screen::Main => render_main_screen(app, frame),
        Screen::NewNote => render_new_note_screen(app, frame),
        Screen::LoadNote => render_load_note_screen(app, frame),
        Screen::Popup => render_popup(app, frame),
        Screen::Exiting => render_exit_screen(frame),
    }
}

fn render_welcome_screen<'a>(app: &mut App, frame: &mut Frame) {
    let area = frame.size();
    let buf = frame.buffer_mut();

    let two_btn_split = [
        Constraint::Percentage(10),
        Constraint::Percentage(30),
        Constraint::Percentage(10),
        Constraint::Percentage(30),
        Constraint::Percentage(10),
    ];

    // Split the main area into three sections:
    // - The top section for title.
    // - The middle section for button labels.
    // - The bottom section for buttons.
    let layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints(&[
            Constraint::Min(1),         // Title section
            Constraint::Percentage(20), // button labels
            Constraint::Percentage(20), // buffer between buttons and labels
            Constraint::Percentage(60), // buttons
        ])
        .split(area);

    let welcome_block = Block::default()
        .title("Welcome to Tuipaz!")
        .borders(Borders::NONE)
        .style(Style::default())
        .title_alignment(Alignment::Center);

    welcome_block.render(layout[0], buf);

    let btn_label_layout = Layout::default()
        .direction(Direction::Horizontal)
        .constraints(&two_btn_split)
        .split(layout[1]);

    let btn_layout = Layout::default()
        .direction(Direction::Horizontal)
        .constraints(&two_btn_split)
        .split(layout[2]);

    let new_note_btn_text =
        Text::styled("Start a new note", Style::default()).alignment(Alignment::Center);
    let load_note_btn_text =
        Text::styled("Load a note", Style::default()).alignment(Alignment::Center);

    Paragraph::new(new_note_btn_text)
        .block(Block::default().borders(Borders::ALL))
        .wrap(Wrap { trim: false })
        .render(btn_label_layout[1], buf);

    Paragraph::new(load_note_btn_text)
        .block(Block::default().borders(Borders::ALL))
        .wrap(Wrap { trim: false })
        .render(btn_label_layout[3], buf);

    let new_note_btn = app
        .btns
        .get_mut(&0)
        .expect("New note btn should be present");
    new_note_btn.clone().render(btn_layout[1], buf);

    let load_note_btn = app
        .btns
        .get_mut(&1)
        .expect("Load note btn should be present");
    load_note_btn.clone().render(btn_layout[3], buf);
}

fn render_main_screen<'a>(app: &mut App, frame: &mut Frame) {
    let area = frame.size();
    let buf = frame.buffer_mut();

    let layout = Layout::default()
        .direction(Direction::Horizontal)
        .constraints(&[Constraint::Percentage(80), Constraint::Percentage(20)])
        .split(area);

    let note_title = app.editor.title.clone();

    let editor_block = Block::default()
        .title(Title::from(note_title).alignment(Alignment::Left))
        .title_style(Style::default().add_modifier(Modifier::BOLD))
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .title_bottom(Line::from(" <Esc> Quit "));

    app.editor.body.set_block(editor_block);
    let editor_widget = app.editor.body.widget();
    editor_widget.render(layout[0], buf);

    let files_block = Block::default()
        .title(Title::from(" File Explorer ").alignment(Alignment::Center))
        .title_style(Style::default().add_modifier(Modifier::BOLD))
        .borders(Borders::ALL)
        .border_type(BorderType::Thick);

    let files_text = vec![Line::from("This is the file explorer sidebar").style(Style::default())];

    Paragraph::new(files_text)
        .block(files_block)
        .wrap(Wrap { trim: true })
        .render(layout[1], buf);
}

fn render_popup<'a>(app: &mut App<'a>, frame: &mut Frame) {
    let area = frame.size();
    let buf = frame.buffer_mut();
    app.user_msg.clone().render(area, buf);
}

fn render_exit_screen(frame: &mut Frame) {
    let area = frame.size();
    frame.render_widget(Clear, area);
    let buf = frame.buffer_mut();

    let popup_block = Block::default()
        .title("Y/N")
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .padding(Padding::new(0, 0, 1, 1))
        .style(Style::default());

    let exit_text = Text::styled(" Exit Tuipaz? (y/n)", Style::default().fg(Color::DarkGray));
    // the `trim: false` will stop the text from being cut off when over the edge of the block
    Paragraph::new(exit_text)
        .block(popup_block)
        .alignment(Alignment::Center)
        .wrap(Wrap { trim: false })
        .render(centered_rect(30, 14, area), buf);
}

fn render_load_note_screen<'a>(app: &mut App<'a>, frame: &mut Frame) {
    let area = frame.size();
    frame.render_widget(Clear, area);
    let buf = frame.buffer_mut();

    let load_note_block = Block::default()
        .title(" Load Note ")
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .padding(Padding::new(1, 1, 1, 1))
        .style(Style::default().bg(Color::DarkGray));

    let exit_text = Text::styled("Exit Tuipaz? (y/n)", Style::default().fg(Color::Red))
        .alignment(Alignment::Center);

    Paragraph::new(exit_text)
        .block(load_note_block)
        .wrap(Wrap { trim: false })
        .render(centered_rect(60, 25, area), buf);
}

fn render_new_note_screen<'a>(app: &mut App<'a>, frame: &mut Frame) {
    let area = frame.size();
    let buf = frame.buffer_mut();

    let new_note_block = Block::default()
        .title(" Note Title: ")
        .title_bottom(Line::from(" <Enter> to submit "))
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .padding(Padding::new(1, 1, 1, 1))
        .style(Style::default());

    app.user_input.text.set_block(new_note_block);

    let user_input_widget = app.user_input.text.widget();

    user_input_widget.render(centered_rect(50, 20, area), buf);
}
