mod db;
mod tui;

use color_eyre::Result;
use db::init_db;
use dotenv::dotenv;
use tui::app::App;

#[tokio::main]
async fn main() -> Result<()> {
    dotenv().ok();
    let db = init_db::create_db().await?;
    tui::errors::install_hooks()?;
    let mut term = tui::utils::init()?;
    let res = App::new(db).run(&mut term)?;
    tui::utils::restore()?;

    return Ok(res);
}
