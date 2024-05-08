use color_eyre::eyre::{Context, Result};
use sqlx::{Pool, Sqlite};
use tui_textarea::TextArea;

use super::{events::AppMac, ui::ui, utils::Tui};

#[derive(PartialEq, Debug, Default, Clone, Copy)]
pub(crate) enum AppState {
    #[default]
    Running,
    Exit,
}

#[derive(PartialEq, Debug, Clone, Copy)]
pub(crate) enum CurrentScreen {
    Welcome,
    Main,
    Popup,
    Exiting,
}

#[derive(Debug, Clone)]
pub(crate) struct Editor<'a> {
    pub(crate) title: TextArea<'a>,
    pub(crate) body: TextArea<'a>,
}

#[derive(Debug, Clone)]
pub(crate) struct App<'a> {
    pub(crate) state: AppState,
    pub(crate) screen: CurrentScreen,
    pub(crate) editor: Editor<'a>,
    pub(crate) db: Pool<Sqlite>,
}

impl<'a> App<'a> {
    pub fn new(db: Pool<Sqlite>) -> Self {
        Self {
            state: AppState::default(),
            screen: CurrentScreen::Welcome,
            editor: Editor {
                title: TextArea::default(),
                body: TextArea::default(),
            },
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
