mod tui;

use color_eyre::Result;
use tui::app::App;

fn main() -> Result<()> {
    tui::errors::install_hooks()?;
    let mut term = tui::utils::init()?;
    let res = App::default().run(&mut term)?;
    tui::utils::restore()?;

    return Ok(res);
}
