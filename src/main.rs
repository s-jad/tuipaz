mod db;
mod tui;

use color_eyre::Result;
use db::init_db;
use dotenv::dotenv;
use tui::app::{run, App};

#[tokio::main]
async fn main() -> Result<()> {
    dotenv().ok();
    tui::errors::install_hooks()?;
    let db = init_db::create_db().await?;
    let mut term = tui::utils::init()?;
    let mut app = App::new(db);
    let res = run(&mut app, &mut term).await?;
    tui::utils::restore()?;

    return Ok(res);
}
