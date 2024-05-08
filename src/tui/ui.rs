use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Text},
    widgets::{block::Title, Block, BorderType, Borders, Paragraph, Widget, Wrap},
    Frame,
};

use super::{
    app::{App, CurrentScreen},
    buttons::{Button, ButtonState},
    user_messages::centered_rect,
};

pub(crate) fn ui(app: &mut App, frame: &mut Frame) {
    match app.screen {
        CurrentScreen::Welcome => render_welcome_screen(app, frame),
        CurrentScreen::Main => render_main_screen(app, frame),
        CurrentScreen::Popup => render_popup(app, frame),
        CurrentScreen::Exiting => render_exit_screen(frame),
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
            Constraint::Min(20),        // Title section
            Constraint::Percentage(20), // button labels
            Constraint::Percentage(0),  // buttons
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

    Button::new("New".to_owned(), ButtonState::Active).render(btn_layout[1], buf);
    Button::new("Load".to_owned(), ButtonState::Active).render(btn_layout[3], buf);
}

fn render_main_screen<'a>(app: &mut App, frame: &mut Frame) {
    let area = frame.size();
    let buf = frame.buffer_mut();

    let layout = Layout::default()
        .direction(Direction::Horizontal)
        .constraints(&[Constraint::Percentage(80), Constraint::Percentage(20)])
        .split(area);

    let editor_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints(&[Constraint::Min(16), Constraint::Percentage(0)])
        .split(layout[0]);

    let title_block = Block::default()
        .title(Title::from("Note Title").alignment(Alignment::Center))
        .title_style(Style::default().add_modifier(Modifier::BOLD))
        .borders(Borders::ALL)
        .border_type(BorderType::Thick);

    let editor_block = Block::default()
        .title(Title::from("Note Editor").alignment(Alignment::Center))
        .title_style(Style::default().add_modifier(Modifier::BOLD))
        .borders(Borders::ALL)
        .border_type(BorderType::Thick)
        .title_bottom(Line::from(" <Esc> Quit "));

    // Set title textarea
    app.editor.title.set_alignment(Alignment::Center);
    app.editor.title.set_block(title_block);
    let title_widget = app.editor.title.widget();
    title_widget.render(editor_layout[0], buf);

    // Set note body textarea
    app.editor.body.set_block(editor_block);
    let editor_widget = app.editor.body.widget();
    editor_widget.render(editor_layout[1], buf);

    let files_block = Block::default()
        .title(Title::from("File Explorer").alignment(Alignment::Center))
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
    //   app.user_msg.clone().render(area, buf);
}

fn render_exit_screen<'a>(frame: &mut Frame) {
    let area = frame.size();
    let buf = frame.buffer_mut();

    let popup_block = Block::default()
        .title("Y/N")
        .borders(Borders::NONE)
        .style(Style::default().bg(Color::DarkGray));

    let exit_text = Text::styled("Exit Tuipaz? (y/n)", Style::default().fg(Color::Red))
        .alignment(Alignment::Center);
    // the `trim: false` will stop the text from being cut off when over the edge of the block
    Paragraph::new(exit_text)
        .block(popup_block)
        .wrap(Wrap { trim: false })
        .render(centered_rect(60, 25, area), buf);
}
