use std::io::{self, stdout, Stdout};

use crossterm::{execute, terminal::*};
use ratatui::{backend::CrosstermBackend, Terminal};

// Terminal type used in this application
pub type Tui = Terminal<CrosstermBackend<Stdout>>;

pub(crate) fn init() -> io::Result<Tui> {
    execute!(stdout(), EnterAlternateScreen)?;
    enable_raw_mode()?;
    Terminal::new(CrosstermBackend::new(stdout()))
}

pub(crate) fn restore() -> io::Result<()> {
    execute!(stdout(), LeaveAlternateScreen)?;
    disable_raw_mode()?;
    Ok(())
}

pub(crate) fn log_format<T: std::fmt::Debug>(data: &T, prefix: &str) -> String {
    let mut s = String::new();
    s.push_str(prefix);
    s.push_str(": ");
    s.push_str(&format!("{:?}", data));
    s
}
