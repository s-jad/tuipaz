use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Text},
    widgets::{block::Title, Block, BorderType, Borders, Paragraph, Widget, Wrap},
    Frame,
};

use super::{
    app::{App, CurrentScreen},
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

    let popup_block = Block::default()
        .title("Welcome to Tuipaz!")
        .borders(Borders::NONE)
        .style(Style::default().bg(Color::DarkGray));

    let welcome_text = Text::styled("Welcome to Tuipaz!", Style::default().fg(Color::Red))
        .alignment(Alignment::Center);

    Paragraph::new(welcome_text)
        .block(popup_block)
        .wrap(Wrap { trim: false })
        .render(centered_rect(60, 25, area), buf);
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

    let title_input_block = Block::default()
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
    app.editor.title.set_block(editor_block);
    let editor_widget = app.editor.title.widget();
    editor_widget.render(editor_layout[0], buf);

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
