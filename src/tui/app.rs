use color_eyre::eyre::{Context, Result};
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Text},
    widgets::{block::Title, Block, BorderType, Borders, Paragraph, Widget, Wrap},
    Frame,
};
use sqlx::{Pool, Sqlite};
use tui_textarea::TextArea;

use super::{events::AppMac, user_messages::centered_rect, utils::Tui};

#[derive(PartialEq, Debug, Default, Clone, Copy)]
pub(crate) enum AppState {
    #[default]
    Running,
    Exit,
}

#[derive(PartialEq, Debug, Clone, Copy)]
pub(crate) enum CurrentScreen {
    Main,
    Popup,
    Exiting,
}

#[derive(Debug, Clone)]
pub(crate) struct App<'a> {
    pub(crate) state: AppState,
    pub(crate) screen: CurrentScreen,
    pub(crate) editor: TextArea<'a>,
    pub(crate) db: Pool<Sqlite>,
}

impl<'a> App<'a> {
    pub fn new(db: Pool<Sqlite>) -> Self {
        Self {
            state: AppState::default(),
            screen: CurrentScreen::Main,
            editor: TextArea::default(),
            db,
        }
    }
}

pub(crate) async fn run(app: &mut App<'_>, terminal: &mut Tui) -> Result<()> {
    // MAIN PROGRAM LOOP
    while app.state != AppState::Exit {
        terminal.draw(|frame| ui(app, frame))?;
        let result = AppMac::handle_events(app).await;

        result.wrap_err("handle events failed")?;
    }

    Ok(())
}

fn ui(app: &mut App, frame: &mut Frame) {
    let area = frame.size();
    match app.screen {
        CurrentScreen::Main => render_main_screen(app, frame),
        CurrentScreen::Popup => render_popup(app, frame),
        CurrentScreen::Exiting => render_exit_screen(frame),
    }
}

fn render_main_screen<'a>(app: &mut App, frame: &mut Frame) {
    let area = frame.size();
    let buf = frame.buffer_mut();

    let layout = Layout::default()
        .direction(Direction::Horizontal)
        .constraints(&[Constraint::Percentage(80), Constraint::Percentage(20)])
        .split(area);

    let editor_block = Block::default()
        .title(Title::from("Note Editor").alignment(Alignment::Center))
        .title_style(Style::default().add_modifier(Modifier::BOLD))
        .borders(Borders::ALL)
        .border_type(BorderType::Thick)
        .title_bottom(Line::from(" <Esc> Quit "));

    app.editor.set_block(editor_block);
    let editor_widget = app.editor.widget();

    editor_widget.render(layout[0], buf);

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

    let exit_text = Text::styled("Exit Tuipaz? (y/n)", Style::default().fg(Color::Red));
    // the `trim: false` will stop the text from being cut off when over the edge of the block
    Paragraph::new(exit_text)
        .block(popup_block)
        .wrap(Wrap { trim: false })
        .render(centered_rect(60, 25, area), buf);
}
